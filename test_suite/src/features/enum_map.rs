//! The `enum-map` feature: values are walked, keys are never offered.

use enum_map::{Enum, EnumMap, enum_map};
use typesift::TypeSift;

use crate::helpers::{assert_same_refs, check_consistency};

/// Deliberately without a `TypeSift` impl: a key type does not need one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
enum Slot {
    First,
    Second,
    Third,
}

#[derive(Debug, Clone, Copy, PartialEq, TypeSift)]
struct Id(u32);

#[derive(Debug, TypeSift)]
struct Board {
    slots: EnumMap<Slot, Id>,
    label: String,
}

fn board() -> Board {
    Board {
        slots: enum_map! {
            Slot::First => Id(1),
            Slot::Second => Id(2),
            Slot::Third => Id(3),
        },
        label: "board".to_string(),
    }
}

#[test]
fn values_are_walked_in_variant_order() {
    let board = board();
    assert_same_refs(
        &board.sift::<Id>(),
        &[
            &board.slots[Slot::First],
            &board.slots[Slot::Second],
            &board.slots[Slot::Third],
        ],
    );
    assert_eq!(board.sift::<u32>(), [&1, &2, &3]);
    assert_eq!(board.sift::<String>(), ["board"]);
}

#[test]
fn keys_are_never_offered() {
    let board = board();
    // An `EnumMap` is an array indexed by the key's position, so it holds no keys to offer.
    assert!(board.sift::<Slot>().is_empty());
}

#[test]
fn the_map_is_found_as_itself() {
    let board = board();
    assert_same_refs(&board.sift::<EnumMap<Slot, Id>>(), &[&board.slots]);
}

#[test]
fn entry_points_agree_for_enum_map() {
    check_consistency::<_, Id>(&board());
    check_consistency::<_, EnumMap<Slot, Id>>(&board());
    check_consistency::<_, Board>(&board());
}
