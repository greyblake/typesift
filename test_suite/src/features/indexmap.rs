//! The `indexmap` feature: both types are walked, in insertion order.

use indexmap::{IndexMap, IndexSet};
use typesift::TypeSift;

use crate::helpers::{assert_same_refs, check_consistency};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypeSift)]
struct Id(u32);

#[derive(Debug, TypeSift)]
struct Registry {
    by_id: IndexMap<Id, String>,
    tags: IndexSet<Id>,
}

/// Keys are inserted out of order, so sorted order and insertion order differ.
fn registry() -> Registry {
    let mut by_id = IndexMap::new();
    by_id.insert(Id(3), "three".to_string());
    by_id.insert(Id(1), "one".to_string());
    by_id.insert(Id(2), "two".to_string());

    let mut tags = IndexSet::new();
    tags.insert(Id(30));
    tags.insert(Id(10));

    Registry { by_id, tags }
}

#[test]
fn both_types_are_walked_in_insertion_order() {
    let registry = registry();
    let ids: Vec<u32> = registry.sift::<Id>().iter().map(|id| id.0).collect();
    assert_eq!(ids, [3, 1, 2, 30, 10]);
    assert_eq!(registry.sift::<String>(), ["three", "one", "two"]);
}

#[test]
fn each_key_comes_before_its_value() {
    let registry = registry();
    let first_key = registry.by_id.keys().next().expect("the map has entries");
    let ids = registry.sift::<Id>();
    assert!(std::ptr::eq(ids[0], first_key));
    assert_same_refs(
        &registry.sift::<String>(),
        &[
            &registry.by_id[&Id(3)],
            &registry.by_id[&Id(1)],
            &registry.by_id[&Id(2)],
        ],
    );
}

#[test]
fn collections_are_found_as_themselves() {
    let registry = registry();
    assert_same_refs(&registry.sift::<IndexMap<Id, String>>(), &[&registry.by_id]);
    assert_same_refs(&registry.sift::<IndexSet<Id>>(), &[&registry.tags]);
}

#[test]
fn entry_points_agree_for_indexmap() {
    check_consistency::<_, Id>(&registry());
    check_consistency::<_, String>(&registry());
    check_consistency::<_, Registry>(&registry());
}
