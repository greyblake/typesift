//! A JSON-like value, a seeded generator for it and traversals written without the library.

use std::collections::BTreeMap;

use typesift::TypeSift;

#[derive(Debug, Clone, PartialEq, TypeSift)]
pub enum Json {
    Null,
    Bool(bool),
    Int(i64),
    Text(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

/// Four levels of `Json` deep: object > object > array > int.
///
/// `sift::<String>()` gives `name, typesift, nested, deep, tags, rust` (keys are sorted).
/// `sift::<i64>()` gives `2, 1`. `sift::<Json>()` gives 10 nodes.
pub fn sample_json() -> Json {
    Json::Object(BTreeMap::from([
        ("name".to_string(), Json::Text("typesift".to_string())),
        (
            "tags".to_string(),
            Json::Array(vec![
                Json::Text("rust".to_string()),
                Json::Int(1),
                Json::Null,
            ]),
        ),
        (
            "nested".to_string(),
            Json::Object(BTreeMap::from([(
                "deep".to_string(),
                Json::Array(vec![Json::Bool(true), Json::Int(2)]),
            )])),
        ),
    ]))
}

/// A deterministic pseudo-random document, at most `max_depth` containers deep and `max_width` wide.
pub fn random_json(seed: u64, max_depth: u32, max_width: u64) -> Json {
    generate(&mut XorShift(seed.max(1)), max_depth, max_width)
}

struct XorShift(u64);

impl XorShift {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next_u64() % bound
    }
}

fn generate(rng: &mut XorShift, depth: u32, width: u64) -> Json {
    let kinds = if depth == 0 { 4 } else { 6 };
    match rng.below(kinds) {
        0 => Json::Null,
        1 => Json::Bool(rng.below(2) == 1),
        2 => Json::Int(rng.below(1_000) as i64 - 500),
        3 => Json::Text(format!("text-{}", rng.below(1_000))),
        4 => Json::Array(
            (0..rng.below(width + 1))
                .map(|_| generate(rng, depth - 1, width))
                .collect(),
        ),
        _ => Json::Object(
            (0..rng.below(width + 1))
                .map(|index| (format!("key-{index}"), generate(rng, depth - 1, width)))
                .collect(),
        ),
    }
}

/// Every node in pre-order.
pub fn reference_nodes(json: &Json) -> Vec<&Json> {
    fn collect<'a>(json: &'a Json, out: &mut Vec<&'a Json>) {
        out.push(json);
        match json {
            Json::Array(items) => items.iter().for_each(|item| collect(item, out)),
            Json::Object(fields) => fields.values().for_each(|value| collect(value, out)),
            Json::Null | Json::Bool(_) | Json::Int(_) | Json::Text(_) => {}
        }
    }

    let mut out = Vec::new();
    collect(json, &mut out);
    out
}

/// Every string in pre-order, with each object key before its value.
pub fn reference_strings(json: &Json) -> Vec<&String> {
    fn collect<'a>(json: &'a Json, out: &mut Vec<&'a String>) {
        match json {
            Json::Text(text) => out.push(text),
            Json::Array(items) => items.iter().for_each(|item| collect(item, out)),
            Json::Object(fields) => {
                for (key, value) in fields {
                    out.push(key);
                    collect(value, out);
                }
            }
            Json::Null | Json::Bool(_) | Json::Int(_) => {}
        }
    }

    let mut out = Vec::new();
    collect(json, &mut out);
    out
}

/// Every integer in pre-order.
pub fn reference_ints(json: &Json) -> Vec<&i64> {
    reference_nodes(json)
        .into_iter()
        .filter_map(|node| match node {
            Json::Int(value) => Some(value),
            _ => None,
        })
        .collect()
}
