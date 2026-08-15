//! Behaviour of individual std impls that a single-element fixture cannot show.

use std::any::Any;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::marker::PhantomData;
use std::num::NonZero;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use typesift::TypeSift;

use crate::fixtures::containers::{FixedHasher, MARKER_COUNT, all_containers, leaves};
use crate::fixtures::ids::Marker;
use crate::helpers::{assert_same_refs, sorted};

#[test]
fn every_container_yields_its_markers_in_order() {
    let markers: Vec<u32> = all_containers()
        .sift::<Marker>()
        .iter()
        .map(|marker| marker.0)
        .collect();
    assert_eq!(markers, (1..=MARKER_COUNT).collect::<Vec<_>>());
}

// STD1
#[test]
fn vec_deque_yields_logical_order() {
    let mut deque = VecDeque::with_capacity(4);
    deque.push_back(Marker(3));
    deque.push_back(Marker(4));
    deque.push_front(Marker(2));
    deque.push_front(Marker(1));
    assert_eq!(
        deque.sift::<Marker>(),
        [&Marker(1), &Marker(2), &Marker(3), &Marker(4)]
    );
}

// STD2
#[test]
fn btree_collections_yield_sorted_order() {
    let set = BTreeSet::from([Marker(3), Marker(1), Marker(2)]);
    assert_eq!(set.sift::<Marker>(), [&Marker(1), &Marker(2), &Marker(3)]);

    let map = BTreeMap::from([
        (Marker(30), Marker(31)),
        (Marker(10), Marker(11)),
        (Marker(20), Marker(21)),
    ]);
    let markers: Vec<u32> = map.sift::<Marker>().iter().map(|marker| marker.0).collect();
    assert_eq!(markers, [10, 11, 20, 21, 30, 31]);
}

// STD3
#[test]
fn hash_map_yields_each_key_right_before_its_value() {
    let map: HashMap<Marker, Marker> = (1..=100).map(|n| (Marker(n), Marker(n * 1000))).collect();
    let found = map.sift::<Marker>();
    assert_eq!(found.len(), 200);
    for entry in found.chunks(2) {
        assert_eq!(entry[1].0, entry[0].0 * 1000);
        assert!(std::ptr::eq(entry[1], &map[entry[0]]));
    }
    let mut keys: Vec<u32> = found.chunks(2).map(|entry| entry[0].0).collect();
    keys.sort_unstable();
    assert_eq!(keys, (1..=100).collect::<Vec<_>>());
}

// STD4
#[test]
fn unordered_collections_yield_every_element() {
    let expected: Vec<Marker> = (1..=50).map(Marker).collect();

    let set: HashSet<Marker> = expected.iter().copied().collect();
    assert_eq!(sorted(set.sift::<Marker>()), expected);

    let heap: BinaryHeap<Marker> = expected.iter().copied().collect();
    assert_eq!(sorted(heap.sift::<Marker>()), expected);
}

// STD5
#[test]
fn hasher_parameter_is_part_of_the_type() {
    let mut fixed_map: HashMap<Marker, u8, FixedHasher> = HashMap::default();
    fixed_map.insert(Marker(1), 1);
    let mut fixed_set: HashSet<Marker, FixedHasher> = HashSet::default();
    fixed_set.insert(Marker(2));
    let values = (
        fixed_map,
        HashMap::from([(Marker(3), 3u8)]),
        fixed_set,
        HashSet::from([Marker(4)]),
    );

    assert_same_refs(
        &values.sift::<HashMap<Marker, u8, FixedHasher>>(),
        &[&values.0],
    );
    assert_same_refs(&values.sift::<HashMap<Marker, u8>>(), &[&values.1]);
    assert_same_refs(&values.sift::<HashSet<Marker, FixedHasher>>(), &[&values.2]);
    assert_same_refs(&values.sift::<HashSet<Marker>>(), &[&values.3]);
}

// STD6
#[test]
fn result_with_the_same_type_on_both_sides() {
    let results = [Ok::<Marker, Marker>(Marker(1)), Err(Marker(2))];
    assert_eq!(results.sift::<Marker>(), [&Marker(1), &Marker(2)]);
}

// STD7
#[test]
fn nested_options() {
    let options = [Some(Some(Marker(1))), Some(None), None];
    assert_eq!(options.sift::<Option<Option<Marker>>>().len(), 3);
    assert_eq!(options.sift::<Option<Marker>>(), [&Some(Marker(1)), &None]);
    assert_eq!(options.sift::<Marker>(), [&Marker(1)]);
}

// STD8
#[test]
#[allow(clippy::redundant_allocation)]
fn every_pointer_layer_is_found() {
    let layered: Box<Rc<Arc<Marker>>> = Box::new(Rc::new(Arc::new(Marker(1))));
    assert_same_refs(&layered.sift::<Box<Rc<Arc<Marker>>>>(), &[&layered]);
    assert_same_refs(&layered.sift::<Rc<Arc<Marker>>>(), &[&*layered]);
    assert_same_refs(&layered.sift::<Arc<Marker>>(), &[&**layered]);
    assert_same_refs(&layered.sift::<Marker>(), &[&***layered]);
}

// STD9
#[test]
fn unsized_contents_yield_items_but_never_themselves() {
    let slice: &'static [Marker] = &[Marker(4)];
    let values = (
        Box::<[Marker]>::from([Marker(1), Marker(2)]),
        Rc::<[Marker]>::from([Marker(3)]),
        slice,
        Cow::<'static, [Marker]>::Owned(vec![Marker(5)]),
        Arc::<str>::from("arc"),
    );
    let markers: Vec<u32> = values.sift::<Marker>().iter().map(|m| m.0).collect();
    assert_eq!(markers, [1, 2, 3, 4, 5]);
    assert!(values.sift::<[Marker; 2]>().is_empty());
    assert!(values.sift::<Vec<Marker>>().is_empty());
    assert!(values.sift::<String>().is_empty());
    assert_same_refs(&values.sift::<Arc<str>>(), &[&values.4]);
}

macro_rules! assert_tuple_order {
    ($($n:literal),+) => {{
        let tuple = ($(Marker($n),)+);
        let found: Vec<u32> = tuple.sift::<Marker>().iter().map(|marker| marker.0).collect();
        assert_eq!(found, [$($n),+]);
    }};
}

// STD10
#[test]
fn tuples_of_every_arity() {
    assert_tuple_order!(1);
    assert_tuple_order!(1, 2);
    assert_tuple_order!(1, 2, 3);
    assert_tuple_order!(1, 2, 3, 4);
    assert_tuple_order!(1, 2, 3, 4, 5);
    assert_tuple_order!(1, 2, 3, 4, 5, 6);
    assert_tuple_order!(1, 2, 3, 4, 5, 6, 7);
    assert_tuple_order!(1, 2, 3, 4, 5, 6, 7, 8);
    assert_tuple_order!(1, 2, 3, 4, 5, 6, 7, 8, 9);
    assert_tuple_order!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
    assert_tuple_order!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
    assert_tuple_order!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);

    let nested = ((Marker(1),), (Marker(2), ((), (Marker(3),))), Marker(4));
    let markers: Vec<u32> = nested.sift::<Marker>().iter().map(|m| m.0).collect();
    assert_eq!(markers, [1, 2, 3, 4]);
    assert_same_refs(&nested.sift::<(Marker,)>(), &[&nested.0, &nested.1.1.1]);
    assert_eq!(nested.sift::<()>().len(), 1);
}

macro_rules! assert_found_once {
    ($root:ident: $($field:ident: $ty:ty),+ $(,)?) => {$(
        assert_same_refs(&$root.sift::<$ty>(), &[&$root.$field]);
    )+};
}

// STD11
#[test]
fn every_leaf_type_is_found_as_itself() {
    let leaves = leaves();
    assert_found_once!(leaves:
        i8: i8, i16: i16, i32: i32, i64: i64, i128: i128, isize: isize,
        u8: u8, u16: u16, u32: u32, u64: u64, u128: u128, usize: usize,
        f32: f32, f64: f64, bool: bool, char: char,
        string: String, path: PathBuf, duration: Duration,
        static_str: &'static str, boxed_str: Box<str>, rc_str: Rc<str>, arc_str: Arc<str>,
        cow_str: Cow<'static, str>,
        non_zero_i8: NonZero<i8>, non_zero_i16: NonZero<i16>, non_zero_i32: NonZero<i32>,
        non_zero_i64: NonZero<i64>, non_zero_i128: NonZero<i128>, non_zero_isize: NonZero<isize>,
        non_zero_u8: NonZero<u8>, non_zero_u16: NonZero<u16>, non_zero_u32: NonZero<u32>,
        non_zero_u64: NonZero<u64>, non_zero_u128: NonZero<u128>, non_zero_usize: NonZero<usize>,
    );
    assert_eq!(leaves.sift::<()>().len(), 1);
}

// STD12
#[test]
fn phantom_data_needs_no_impl_for_its_parameter() {
    let phantoms = (
        PhantomData::<fn()>,
        PhantomData::<dyn Any>,
        PhantomData::<*const u8>,
        PhantomData::<RefCell<u8>>,
    );
    assert_eq!(phantoms.sift::<PhantomData<fn()>>().len(), 1);
    assert_eq!(phantoms.sift::<PhantomData<dyn Any>>().len(), 1);
    assert_eq!(phantoms.sift::<PhantomData<*const u8>>().len(), 1);
    assert_eq!(phantoms.sift::<PhantomData<RefCell<u8>>>().len(), 1);
}

// STD13
#[test]
fn empty_collections_are_found_but_hold_nothing() {
    let empty_array: [Marker; 0] = [];
    let empties = (
        Vec::<Marker>::new(),
        VecDeque::<Marker>::new(),
        LinkedList::<Marker>::new(),
        BTreeSet::<Marker>::new(),
        BinaryHeap::<Marker>::new(),
        HashSet::<Marker>::new(),
        BTreeMap::<Marker, Marker>::new(),
        HashMap::<Marker, Marker>::new(),
        empty_array,
        None::<Marker>,
    );
    assert!(empties.sift::<Marker>().is_empty());
    assert_eq!(empties.sift::<Vec<Marker>>().len(), 1);
    assert_eq!(empties.sift::<VecDeque<Marker>>().len(), 1);
    assert_eq!(empties.sift::<LinkedList<Marker>>().len(), 1);
    assert_eq!(empties.sift::<BTreeSet<Marker>>().len(), 1);
    assert_eq!(empties.sift::<BinaryHeap<Marker>>().len(), 1);
    assert_eq!(empties.sift::<HashSet<Marker>>().len(), 1);
    assert_eq!(empties.sift::<BTreeMap<Marker, Marker>>().len(), 1);
    assert_eq!(empties.sift::<HashMap<Marker, Marker>>().len(), 1);
    assert_eq!(empties.sift::<[Marker; 0]>().len(), 1);
    assert_eq!(empties.sift::<Option<Marker>>().len(), 1);
}

#[test]
#[allow(
    clippy::type_complexity,
    reason = "the nesting is what this test checks"
)]
fn six_levels_of_std_nesting() {
    let nested: Vec<Option<Box<(Result<[Marker; 2], String>,)>>> = vec![
        Some(Box::new((Ok([Marker(1), Marker(2)]),))),
        None,
        Some(Box::new((Err("failed".to_string()),))),
    ];
    assert_eq!(nested.sift::<Marker>(), [&Marker(1), &Marker(2)]);
    assert_eq!(nested.sift::<String>(), ["failed"]);
    assert_eq!(nested.sift::<Result<[Marker; 2], String>>().len(), 2);
    assert_eq!(
        nested
            .sift::<Option<Box<(Result<[Marker; 2], String>,)>>>()
            .len(),
        3
    );
}
