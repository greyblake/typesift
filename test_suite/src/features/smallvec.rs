//! The `smallvec` feature: a `SmallVec` is walked like a `Vec`.

use smallvec::{SmallVec, smallvec};
use typesift::TypeSift;

use crate::helpers::{assert_same_refs, check_consistency};

#[derive(Debug, Clone, Copy, PartialEq, TypeSift)]
struct Id(u32);

type Ids = SmallVec<[Id; 4]>;

#[derive(Debug, TypeSift)]
struct Batch {
    inline: Ids,
    spilled: Ids,
    label: String,
}

fn batch() -> Batch {
    Batch {
        inline: smallvec![Id(1), Id(2)],
        spilled: smallvec![Id(3), Id(4), Id(5), Id(6), Id(7)],
        label: "batch".to_string(),
    }
}

#[test]
fn items_are_found_inline_and_once_spilled() {
    let batch = batch();
    assert!(!batch.inline.spilled(), "these items fit inline");
    assert!(batch.spilled.spilled(), "these items are on the heap");

    let ids: Vec<u32> = batch.sift::<Id>().iter().map(|id| id.0).collect();
    assert_eq!(ids, [1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(batch.sift::<String>(), ["batch"]);
}

#[test]
fn small_vecs_are_found_as_themselves() {
    let batch = batch();
    assert_same_refs(&batch.sift::<Ids>(), &[&batch.inline, &batch.spilled]);
}

#[test]
fn entry_points_agree_for_smallvec() {
    check_consistency::<_, Id>(&batch());
    check_consistency::<_, Ids>(&batch());
    check_consistency::<_, Batch>(&batch());
}
