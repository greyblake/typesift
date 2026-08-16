//! The `serde_json` feature: a `Value` is walked, a `Number` is a leaf.

use serde_json::{Map, Number, Value, json};
use typesift::TypeSift;

use crate::helpers::{assert_same_refs, check_consistency};

/// Object keys are sorted, because a `Map` is a `BTreeMap` unless `preserve_order` is on.
fn document() -> Value {
    json!({
        "name": "typesift",
        "tags": ["rust", "search"],
        "meta": { "version": 1, "stars": 2 },
        "ok": true,
    })
}

#[test]
fn a_value_is_walked_in_key_order() {
    let document = document();
    assert_eq!(
        document.sift::<String>(),
        [
            "meta", "stars", "version", "name", "typesift", "ok", "tags", "rust", "search",
        ]
    );
    assert_eq!(document.sift::<Value>().len(), 9);
    assert_eq!(document.sift::<Map<String, Value>>().len(), 2);
    assert_eq!(document.sift::<Vec<Value>>().len(), 1);
    assert_eq!(document.sift::<bool>(), [&true]);
}

#[test]
fn numbers_are_leaves() {
    let document = document();
    let numbers: Vec<u64> = document
        .sift::<Number>()
        .iter()
        .filter_map(|number| number.as_u64())
        .collect();
    assert_eq!(numbers, [2, 1]);
    // What a `Number` holds is not searched.
    assert!(document.sift::<u64>().is_empty());
    assert!(document.sift::<i64>().is_empty());
    assert!(document.sift::<f64>().is_empty());
}

#[test]
fn the_document_itself_comes_first() {
    let document = document();
    assert_same_refs(&document.sift::<Value>()[..1], &[&document]);
}

#[test]
fn entry_points_agree_for_serde_json() {
    check_consistency::<_, Value>(&document());
    check_consistency::<_, String>(&document());
    check_consistency::<_, Number>(&document());
    check_consistency::<_, Map<String, Value>>(&document());
}
