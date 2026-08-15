use std::collections::{BTreeMap, HashMap};

use type_iter::{TypeIter, TypeValues};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, TypeIter)]
struct Id(u32);

#[derive(Debug, PartialEq, TypeIter)]
struct Named {
    id: Id,
    label: String,
    tags: Vec<String>,
}

#[derive(Debug, PartialEq, TypeIter)]
struct Tuple(Id, Option<Id>);

#[derive(Debug, PartialEq, TypeIter)]
struct Unit;

#[derive(Debug, PartialEq, TypeIter)]
enum Shape {
    Named { id: Id, label: String },
    Tuple(Id, Id),
    Unit,
}

// The parameter is named `T` on purpose: the derive must not clash with it.
#[derive(Debug, PartialEq, TypeIter)]
struct Wrapper<T, const N: usize>
where
    T: Clone,
{
    items: [T; N],
    extra: Option<T>,
}

#[derive(Debug, PartialEq, TypeIter)]
struct Tree {
    value: u32,
    children: Vec<Tree>,
}

#[derive(TypeIter)]
struct List {
    value: u32,
    next: Option<Box<List>>,
}

#[derive(TypeIter)]
enum Never {}

fn assert_type_iter<C: TypeIter>() {}

#[test]
fn named_struct_visits_self_then_fields_in_order() {
    let named = Named {
        id: Id(1),
        label: "a".to_string(),
        tags: vec!["b".to_string(), "c".to_string()],
    };

    assert_eq!(named.type_values::<Id>(), [&Id(1)]);
    assert_eq!(named.type_values::<String>(), ["a", "b", "c"]);
    assert_eq!(named.type_values::<Named>(), [&named]);
    assert!(named.type_values::<bool>().is_empty());
}

#[test]
fn tuple_and_unit_structs() {
    assert_eq!(
        Tuple(Id(1), Some(Id(2))).type_values::<Id>(),
        [&Id(1), &Id(2)]
    );
    assert_eq!(Tuple(Id(3), None).type_values::<Id>(), [&Id(3)]);
    assert_eq!(Unit.type_values::<Unit>(), [&Unit]);
    assert!(Unit.type_values::<Id>().is_empty());
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

    assert_eq!(shapes.type_values::<Id>(), [&Id(1), &Id(2), &Id(3)]);
    assert_eq!(shapes.type_values::<String>(), ["a"]);
    assert_eq!(shapes.type_values::<Shape>().len(), 3);
    assert_eq!(shapes.type_values::<Vec<Shape>>(), [&shapes]);
    assert_type_iter::<Never>();
}

#[test]
fn generic_struct() {
    let wrapper = Wrapper {
        items: [Id(1), Id(2)],
        extra: Some(Id(3)),
    };

    assert_eq!(wrapper.type_values::<Id>(), [&Id(1), &Id(2), &Id(3)]);
    assert_eq!(wrapper.type_values::<[Id; 2]>(), [&[Id(1), Id(2)]]);
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
    assert_eq!(tree.type_values::<u32>(), [&1, &2, &3, &4]);
    assert_eq!(tree.type_values::<Tree>().len(), 4);

    let list = List {
        value: 1,
        next: Some(Box::new(List {
            value: 2,
            next: None,
        })),
    };
    assert_eq!(list.type_values::<u32>(), [&1, &2]);
}

#[test]
fn std_containers() {
    let map = BTreeMap::from([
        (Id(2), vec!["b".to_string()]),
        (Id(1), vec!["a".to_string()]),
    ]);
    assert_eq!(map.type_values::<Id>(), [&Id(1), &Id(2)]);
    assert_eq!(map.type_values::<String>(), ["a", "b"]);

    let hash_map = HashMap::from([(Id(1), true)]);
    assert_eq!(hash_map.type_values::<bool>(), [&true]);

    let results: [Result<Id, String>; 2] = [Ok(Id(1)), Err("failed".to_string())];
    assert_eq!(results.type_values::<Id>(), [&Id(1)]);
    assert_eq!(results.type_values::<String>(), ["failed"]);
}

#[test]
fn find_value_stops_at_first_match() {
    let ids: Vec<Id> = (0..10).map(Id).collect();
    let mut visited = 0;

    let found = ids.find_value::<Id>(|id| {
        visited += 1;
        id.0 == 3
    });

    assert_eq!(found, Some(&Id(3)));
    assert_eq!(visited, 4);
    assert_eq!(ids.find_value::<Id>(|id| id.0 == 42), None);
}
