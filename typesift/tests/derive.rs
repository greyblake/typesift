use std::collections::{BTreeMap, HashMap};

use typesift::TypeSift;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, TypeSift)]
struct Id(u32);

#[derive(Debug, PartialEq, TypeSift)]
struct Named {
    id: Id,
    label: String,
    tags: Vec<String>,
}

#[derive(Debug, PartialEq, TypeSift)]
struct Tuple(Id, Option<Id>);

#[derive(Debug, PartialEq, TypeSift)]
struct Unit;

#[derive(Debug, PartialEq, TypeSift)]
enum Shape {
    Named { id: Id, label: String },
    Tuple(Id, Id),
    Unit,
}

// The parameter is named `T` on purpose: the derive must not clash with it.
#[derive(Debug, PartialEq, TypeSift)]
struct Wrapper<T, const N: usize>
where
    T: Clone,
{
    items: [T; N],
    extra: Option<T>,
}

#[derive(Debug, PartialEq, TypeSift)]
struct Tree {
    value: u32,
    children: Vec<Tree>,
}

#[derive(TypeSift)]
struct List {
    value: u32,
    next: Option<Box<List>>,
}

#[derive(TypeSift)]
enum Never {}

fn assert_type_sift<C: TypeSift>() {}

#[test]
fn named_struct_visits_self_then_fields_in_order() {
    let named = Named {
        id: Id(1),
        label: "a".to_string(),
        tags: vec!["b".to_string(), "c".to_string()],
    };

    assert_eq!(named.sift::<Id>(), [&Id(1)]);
    assert_eq!(named.sift::<String>(), ["a", "b", "c"]);
    assert_eq!(named.sift::<Named>(), [&named]);
    assert!(named.sift::<bool>().is_empty());
}

#[test]
fn tuple_and_unit_structs() {
    assert_eq!(Tuple(Id(1), Some(Id(2))).sift::<Id>(), [&Id(1), &Id(2)]);
    assert_eq!(Tuple(Id(3), None).sift::<Id>(), [&Id(3)]);
    assert_eq!(Unit.sift::<Unit>(), [&Unit]);
    assert!(Unit.sift::<Id>().is_empty());
}

#[test]
fn enum_variants() {
    let shapes = vec![
        Shape::Named {
            id: Id(1),
            label: "a".to_string(),
        },
        Shape::Tuple(Id(2), Id(3)),
        Shape::Unit,
    ];

    assert_eq!(shapes.sift::<Id>(), [&Id(1), &Id(2), &Id(3)]);
    assert_eq!(shapes.sift::<String>(), ["a"]);
    assert_eq!(shapes.sift::<Shape>().len(), 3);
    assert_eq!(shapes.sift::<Vec<Shape>>(), [&shapes]);
    assert_type_sift::<Never>();
}

#[test]
fn generic_struct() {
    let wrapper = Wrapper {
        items: [Id(1), Id(2)],
        extra: Some(Id(3)),
    };

    assert_eq!(wrapper.sift::<Id>(), [&Id(1), &Id(2), &Id(3)]);
    assert_eq!(wrapper.sift::<[Id; 2]>(), [&[Id(1), Id(2)]]);
}

#[test]
fn recursive_types() {
    let tree = Tree {
        value: 1,
        children: vec![
            Tree {
                value: 2,
                children: vec![],
            },
            Tree {
                value: 3,
                children: vec![Tree {
                    value: 4,
                    children: vec![],
                }],
            },
        ],
    };
    assert_eq!(tree.sift::<u32>(), [&1, &2, &3, &4]);
    assert_eq!(tree.sift::<Tree>().len(), 4);

    let list = List {
        value: 1,
        next: Some(Box::new(List {
            value: 2,
            next: None,
        })),
    };
    assert_eq!(list.sift::<u32>(), [&1, &2]);
}

#[test]
fn std_containers() {
    let map = BTreeMap::from([
        (Id(2), vec!["b".to_string()]),
        (Id(1), vec!["a".to_string()]),
    ]);
    assert_eq!(map.sift::<Id>(), [&Id(1), &Id(2)]);
    assert_eq!(map.sift::<String>(), ["a", "b"]);

    let hash_map = HashMap::from([(Id(1), true)]);
    assert_eq!(hash_map.sift::<bool>(), [&true]);

    let results: [Result<Id, String>; 2] = [Ok(Id(1)), Err("failed".to_string())];
    assert_eq!(results.sift::<Id>(), [&Id(1)]);
    assert_eq!(results.sift::<String>(), ["failed"]);
}
