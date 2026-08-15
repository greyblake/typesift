//! Every shape `#[derive(TypeSift)]` accepts.

use std::collections::HashMap;
use std::marker::PhantomData;

use typesift::TypeSift;

use crate::fixtures::ids::{Marker, NodeId, TaskId, TeamId, UserId};
use crate::fixtures::org::Assignee;
use crate::fixtures::shapes::{
    Discriminants, EmptyBraced, EmptyTuple, HygieneNames, MutualA, MutualB, NamedStruct, Nested,
    Never, Pair, RawIdents, SelfRef, SingleVariant, TupleStruct, Typed, UnitStruct, Variants,
    WithCfg, WithDefault, Wrapper, no_import, shadow,
};
use crate::fixtures::trees::{List, Tree, list, numbered_tree};
use crate::helpers::{assert_same_refs, sorted};

fn assert_type_sift<S: TypeSift>() {}

// D1
#[test]
fn named_struct_visits_self_then_fields_in_declaration_order() {
    let named = NamedStruct {
        first: Marker(1),
        label: "label".to_string(),
        second: Marker(2),
    };
    assert_eq!(named.sift::<Marker>(), [&Marker(1), &Marker(2)]);
    assert_eq!(named.sift::<String>(), ["label"]);
    assert_same_refs(&named.sift::<NamedStruct>(), &[&named]);
}

// D2
#[test]
fn tuple_unit_and_empty_structs() {
    assert_eq!(
        TupleStruct(Marker(1), Some(Marker(2))).sift::<Marker>(),
        [&Marker(1), &Marker(2)]
    );
    assert_eq!(TupleStruct(Marker(3), None).sift::<Marker>(), [&Marker(3)]);

    assert_eq!(UnitStruct.sift::<UnitStruct>(), [&UnitStruct]);
    assert!(UnitStruct.sift::<Marker>().is_empty());
    assert_eq!(EmptyBraced {}.sift::<EmptyBraced>(), [&EmptyBraced {}]);
    assert_eq!(EmptyTuple().sift::<EmptyTuple>(), [&EmptyTuple()]);
}

// D3
#[test]
fn enum_variants_visit_fields_in_declaration_order() {
    let variants = [
        Variants::Unit,
        Variants::Tuple(Marker(1), Marker(2)),
        Variants::Named {
            b: Marker(3),
            a: Marker(4),
        },
    ];
    assert_eq!(
        variants.sift::<Marker>(),
        [&Marker(1), &Marker(2), &Marker(3), &Marker(4)]
    );
    assert_same_refs(
        &variants.sift::<Variants>(),
        &[&variants[0], &variants[1], &variants[2]],
    );
}

// D4
#[test]
fn uninhabited_enum() {
    assert_type_sift::<Never>();
    let none: Option<Never> = None;
    assert!(none.sift::<Marker>().is_empty());
}

// D5
#[test]
fn single_variant_and_explicit_discriminants() {
    assert_eq!(
        SingleVariant::Only(Marker(1)).sift::<Marker>(),
        [&Marker(1)]
    );
    assert_eq!(Discriminants::A(Marker(2)).sift::<Marker>(), [&Marker(2)]);
    assert!(Discriminants::B.sift::<Marker>().is_empty());
    assert_eq!(
        Discriminants::B.sift::<Discriminants>(),
        [&Discriminants::B]
    );
}

// D6
#[test]
fn generic_structs() {
    let wrapper = Wrapper {
        items: [Marker(1), Marker(2)],
        extra: Some(Marker(3)),
    };
    assert_eq!(
        wrapper.sift::<Marker>(),
        [&Marker(1), &Marker(2), &Marker(3)]
    );
    assert_same_refs(&wrapper.sift::<[Marker; 2]>(), &[&wrapper.items]);

    let defaulted: WithDefault = WithDefault { value: Marker(4) };
    assert_eq!(defaulted.sift::<Marker>(), [&Marker(4)]);
    assert_eq!(WithDefault { value: 5u16 }.sift::<u16>(), [&5]);

    let pair = Pair {
        left: Marker(6),
        right: "right".to_string(),
    };
    assert_eq!(pair.sift::<Marker>(), [&Marker(6)]);
    assert_eq!(pair.sift::<String>(), ["right"]);

    let nested = Nested {
        inner: Wrapper {
            items: [numbered_tree(1, 2)],
            extra: None,
        },
    };
    assert_eq!(
        nested.sift::<NodeId>(),
        [&NodeId(1), &NodeId(2), &NodeId(3)]
    );
    assert_same_refs(&nested.sift::<Wrapper<Tree<NodeId>, 1>>(), &[&nested.inner]);
}

// D7
#[test]
fn parameter_used_only_in_phantom_data() {
    let typed = Typed::<UserId> {
        raw: 1,
        marker: PhantomData,
    };
    assert_same_refs(&typed.sift::<PhantomData<UserId>>(), &[&typed.marker]);
    assert!(typed.sift::<UserId>().is_empty());
}

// D8, D9, D10, D11, D12
#[test]
fn names_that_could_clash_with_generated_code() {
    let names = HygieneNames {
        visitor: Marker(1),
        value: 2u8,
        f: "f".to_string(),
    };
    assert_eq!(names.sift::<Marker>(), [&Marker(1)]);
    assert_eq!(names.sift::<u8>(), [&2]);
    assert_eq!(names.sift::<String>(), ["f"]);

    let raw = RawIdents {
        r#type: Marker(1),
        r#match: Marker(2),
    };
    assert_eq!(raw.sift::<Marker>(), [&Marker(1), &Marker(2)]);

    assert_eq!(WithCfg { a: Marker(1) }.sift::<Marker>(), [&Marker(1)]);
    assert_eq!(
        shadow::Shadowed { id: Marker(1) }.sift::<Marker>(),
        [&Marker(1)]
    );
    assert_eq!(no_import::NotImported(3).sift::<u8>(), [&3]);
}

// D13
#[test]
fn recursive_types() {
    let self_ref = SelfRef {
        marker: Marker(1),
        children: vec![
            SelfRef {
                marker: Marker(2),
                children: vec![SelfRef {
                    marker: Marker(3),
                    children: Vec::new(),
                }],
            },
            SelfRef {
                marker: Marker(4),
                children: Vec::new(),
            },
        ],
    };
    let markers: Vec<u32> = self_ref.sift::<Marker>().iter().map(|m| m.0).collect();
    assert_eq!(markers, [1, 2, 3, 4]);
    assert_eq!(self_ref.sift::<SelfRef>().len(), 4);

    let mutual = MutualA {
        marker: Marker(1),
        bs: vec![
            MutualB {
                marker: Marker(2),
                a: Some(Box::new(MutualA {
                    marker: Marker(3),
                    bs: Vec::new(),
                })),
            },
            MutualB {
                marker: Marker(4),
                a: None,
            },
        ],
    };
    let markers: Vec<u32> = mutual.sift::<Marker>().iter().map(|m| m.0).collect();
    assert_eq!(markers, [1, 2, 3, 4]);
    assert_eq!(mutual.sift::<MutualA>().len(), 2);
    assert_eq!(mutual.sift::<MutualB>().len(), 2);

    let list = list(5);
    assert_eq!(list.sift::<u32>(), [&1, &2, &3, &4, &5]);
    assert_eq!(list.sift::<List>().len(), 5);
}

// D14
#[test]
fn derived_types_and_std_containers_nest_both_ways() {
    let assignments = HashMap::from([
        (TaskId(1), Assignee::User(UserId(5))),
        (
            TaskId(2),
            Assignee::Team {
                team: TeamId(1),
                reviewer: None,
            },
        ),
    ]);
    assert_eq!(sorted(assignments.sift::<TaskId>()), [TaskId(1), TaskId(2)]);
    assert_eq!(assignments.sift::<UserId>(), [&UserId(5)]);
    assert_eq!(assignments.sift::<TeamId>(), [&TeamId(1)]);
    assert_eq!(assignments.sift::<Option<UserId>>(), [&None]);
}

// D15
#[test]
fn types_declared_inside_a_function() {
    #[derive(TypeSift)]
    struct Local {
        hidden: Marker,
    }

    assert_eq!(Local { hidden: Marker(9) }.sift::<Marker>(), [&Marker(9)]);
}
