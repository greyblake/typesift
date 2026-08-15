//! One value per std impl, and one value per leaf type.

use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::hash::{BuildHasherDefault, DefaultHasher};
use std::marker::PhantomData;
use std::num::NonZero;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use typesift::TypeSift;

use super::ids::Marker;

pub type FixedHasher = BuildHasherDefault<DefaultHasher>;

/// Holds `Marker(1)..=Marker(MARKER_COUNT)` in the order a pre-order walk reaches them.
///
/// Unordered containers hold a single element, so the whole sequence is deterministic.
#[derive(TypeSift)]
pub struct AllContainers {
    pub plain: Marker,                                  // 1
    pub vec: Vec<Marker>,                               // 2, 3
    pub vec_deque: VecDeque<Marker>,                    // 4, 5
    pub linked_list: LinkedList<Marker>,                // 6
    pub btree_set: BTreeSet<Marker>,                    // 7, 8
    pub binary_heap: BinaryHeap<Marker>,                // 9
    pub hash_set: HashSet<Marker, FixedHasher>,         // 10
    pub btree_map: BTreeMap<Marker, Marker>,            // 11 -> 12, 13 -> 14
    pub hash_map: HashMap<Marker, Marker, FixedHasher>, // 15 -> 16
    pub array: [Marker; 2],                             // 17, 18
    pub empty_array: [Marker; 0],
    pub empty_vec: Vec<Marker>,
    pub static_ref: &'static Marker,          // 19
    pub static_slice: &'static [Marker],      // 20, 21
    pub boxed: Box<Marker>,                   // 22
    pub boxed_slice: Box<[Marker]>,           // 23
    pub rc: Rc<Marker>,                       // 24
    pub rc_slice: Rc<[Marker]>,               // 25
    pub arc: Arc<Marker>,                     // 26
    pub cow_borrowed: Cow<'static, [Marker]>, // 27
    pub cow_owned: Cow<'static, [Marker]>,    // 28
    pub some: Option<Marker>,                 // 29
    pub none: Option<Marker>,
    pub ok: Result<Marker, Marker>,     // 30
    pub err: Result<Marker, Marker>,    // 31
    pub reverse: Reverse<Marker>,       // 32
    pub tuple: (Marker, (Marker,), ()), // 33, 34
    #[allow(clippy::redundant_allocation)]
    pub layered: Box<Rc<Arc<Option<Marker>>>>, // 35
    pub phantom: PhantomData<Marker>,
    pub leaves: Leaves,
}

pub const MARKER_COUNT: u32 = 35;

pub fn all_containers() -> AllContainers {
    // `push_front` after `push_back` makes the ring buffer wrap around.
    let mut vec_deque = VecDeque::new();
    vec_deque.push_back(Marker(5));
    vec_deque.push_front(Marker(4));

    let mut hash_set = HashSet::default();
    hash_set.insert(Marker(10));

    let mut hash_map = HashMap::default();
    hash_map.insert(Marker(15), Marker(16));

    AllContainers {
        plain: Marker(1),
        vec: vec![Marker(2), Marker(3)],
        vec_deque,
        linked_list: LinkedList::from([Marker(6)]),
        btree_set: BTreeSet::from([Marker(8), Marker(7)]),
        binary_heap: BinaryHeap::from([Marker(9)]),
        hash_set,
        btree_map: BTreeMap::from([(Marker(13), Marker(14)), (Marker(11), Marker(12))]),
        hash_map,
        array: [Marker(17), Marker(18)],
        empty_array: [],
        empty_vec: Vec::new(),
        static_ref: &Marker(19),
        static_slice: &[Marker(20), Marker(21)],
        boxed: Box::new(Marker(22)),
        boxed_slice: Box::new([Marker(23)]),
        rc: Rc::new(Marker(24)),
        rc_slice: Rc::new([Marker(25)]),
        arc: Arc::new(Marker(26)),
        cow_borrowed: Cow::Borrowed(&[Marker(27)]),
        cow_owned: Cow::Owned(vec![Marker(28)]),
        some: Some(Marker(29)),
        none: None,
        ok: Ok(Marker(30)),
        err: Err(Marker(31)),
        reverse: Reverse(Marker(32)),
        tuple: (Marker(33), (Marker(34),), ()),
        layered: Box::new(Rc::new(Arc::new(Some(Marker(35))))),
        phantom: PhantomData,
        leaves: leaves(),
    }
}

/// Exactly one field of every leaf type, and no `Marker`.
#[derive(TypeSift)]
pub struct Leaves {
    pub i8: i8,
    pub i16: i16,
    pub i32: i32,
    pub i64: i64,
    pub i128: i128,
    pub isize: isize,
    pub u8: u8,
    pub u16: u16,
    pub u32: u32,
    pub u64: u64,
    pub u128: u128,
    pub usize: usize,
    pub f32: f32,
    pub f64: f64,
    pub bool: bool,
    pub char: char,
    pub unit: (),
    pub string: String,
    pub path: PathBuf,
    pub duration: Duration,
    pub static_str: &'static str,
    pub boxed_str: Box<str>,
    pub rc_str: Rc<str>,
    pub arc_str: Arc<str>,
    pub cow_str: Cow<'static, str>,
    pub non_zero_i8: NonZero<i8>,
    pub non_zero_i16: NonZero<i16>,
    pub non_zero_i32: NonZero<i32>,
    pub non_zero_i64: NonZero<i64>,
    pub non_zero_i128: NonZero<i128>,
    pub non_zero_isize: NonZero<isize>,
    pub non_zero_u8: NonZero<u8>,
    pub non_zero_u16: NonZero<u16>,
    pub non_zero_u32: NonZero<u32>,
    pub non_zero_u64: NonZero<u64>,
    pub non_zero_u128: NonZero<u128>,
    pub non_zero_usize: NonZero<usize>,
}

pub fn leaves() -> Leaves {
    Leaves {
        i8: -8,
        i16: -16,
        i32: -32,
        i64: -64,
        i128: -128,
        isize: -1,
        u8: 8,
        u16: 16,
        u32: 32,
        u64: 64,
        u128: 128,
        usize: 1,
        f32: 3.5,
        f64: 6.5,
        bool: true,
        char: 'c',
        unit: (),
        string: "string".to_string(),
        path: PathBuf::from("/leaves"),
        duration: Duration::from_secs(7),
        static_str: "static",
        boxed_str: "boxed".into(),
        rc_str: "rc".into(),
        arc_str: "arc".into(),
        cow_str: Cow::Borrowed("cow"),
        non_zero_i8: NonZero::<i8>::MAX,
        non_zero_i16: NonZero::<i16>::MAX,
        non_zero_i32: NonZero::<i32>::MAX,
        non_zero_i64: NonZero::<i64>::MAX,
        non_zero_i128: NonZero::<i128>::MAX,
        non_zero_isize: NonZero::<isize>::MAX,
        non_zero_u8: NonZero::<u8>::MAX,
        non_zero_u16: NonZero::<u16>::MAX,
        non_zero_u32: NonZero::<u32>::MAX,
        non_zero_u64: NonZero::<u64>::MAX,
        non_zero_u128: NonZero::<u128>::MAX,
        non_zero_usize: NonZero::<usize>::MAX,
    }
}
