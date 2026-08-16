//! The `http` feature: `HeaderMap` is walked, the rest are leaves.

use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri};
use typesift::TypeSift;

use crate::helpers::{assert_same_refs, check_consistency};

#[derive(Debug, TypeSift)]
struct Exchange {
    method: Method,
    uri: Uri,
    status: StatusCode,
    headers: HeaderMap,
    note: String,
}

fn exchange() -> Exchange {
    let mut headers = HeaderMap::new();
    headers.insert("accept", HeaderValue::from_static("application/json"));
    headers.insert("x-trace", HeaderValue::from_static("abc"));

    Exchange {
        method: Method::POST,
        uri: "https://example.test/orders".parse().expect("a valid uri"),
        status: StatusCode::CREATED,
        headers,
        note: "created".to_string(),
    }
}

#[test]
fn header_names_and_values_are_found() {
    let exchange = exchange();
    // A `HeaderMap` iterates in an unspecified order, so compare as a multiset.
    let mut names: Vec<&str> = exchange
        .sift::<HeaderName>()
        .iter()
        .map(|name| name.as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(names, ["accept", "x-trace"]);
    assert_eq!(exchange.sift::<HeaderValue>().len(), 2);
    assert_same_refs(&exchange.sift::<HeaderMap>(), &[&exchange.headers]);
}

#[test]
fn each_name_is_offered_with_its_value() {
    let exchange = exchange();
    let names = exchange.sift::<HeaderName>();
    let values = exchange.sift::<HeaderValue>();
    assert_eq!(names.len(), values.len());
    for (name, value) in names.iter().zip(&values) {
        assert_eq!(exchange.headers.get(*name), Some(*value));
    }
}

#[test]
fn the_small_types_are_leaves() {
    let exchange = exchange();
    assert_same_refs(&exchange.sift::<Method>(), &[&exchange.method]);
    assert_same_refs(&exchange.sift::<Uri>(), &[&exchange.uri]);
    assert_same_refs(&exchange.sift::<StatusCode>(), &[&exchange.status]);
    // A `Uri` holds strings and a `StatusCode` a number. Neither is searched.
    assert_eq!(exchange.sift::<String>(), ["created"]);
    assert!(exchange.sift::<u16>().is_empty());
}

#[test]
fn entry_points_agree_for_http() {
    check_consistency::<_, HeaderName>(&exchange());
    check_consistency::<_, HeaderValue>(&exchange());
    check_consistency::<_, HeaderMap>(&exchange());
    check_consistency::<_, Exchange>(&exchange());
}
