//! Wide and deep values.
//!
//! The traversal recurses once per nesting level, so depth is bounded by the thread's stack. In a
//! debug build on the 2 MiB test thread a list of 10 000 nodes works and 30 000 overflows, so
//! these tests stay well below that.

use typesift::TypeSift;

use crate::fixtures::ids::{Marker, NodeId};
use crate::fixtures::trees::{List, list, numbered_tree};

// X1
#[test]
fn wide_vec() {
    let wide: Vec<Marker> = (0..1_000_000).map(Marker).collect();
    let found = wide.sift::<Marker>();
    assert_eq!(found.len(), 1_000_000);
    assert!(std::ptr::eq(found[0], &wide[0]));
    assert!(std::ptr::eq(found[999_999], &wide[999_999]));
}

// X2
#[test]
fn deep_list_and_tree() {
    let list = list(1_000);
    let values: Vec<u32> = list.sift::<u32>().into_iter().copied().collect();
    assert_eq!(values, (1..=1_000).collect::<Vec<_>>());
    assert_eq!(list.sift::<List>().len(), 1_000);

    let chain = numbered_tree(999, 1);
    assert_eq!(chain.sift::<NodeId>().len(), 1_000);
    assert_eq!(chain.sift::<NodeId>()[999], &NodeId(1_000));
}

// X3
#[test]
fn many_zero_sized_values() {
    let units: Vec<()> = std::iter::repeat_n((), 1_000_000).collect();
    assert_eq!(units.sift::<()>().len(), 1_000_000);
}
