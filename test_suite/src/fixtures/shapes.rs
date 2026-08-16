//! Every shape the derive accepts, including names that could clash with generated code.

use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::ops::ControlFlow;
use std::rc::Weak;
use std::sync::Mutex;

use typesift::{Sifter, TypeSift};

use super::ids::Marker;
use super::trees::Tree;

#[derive(Debug, PartialEq, TypeSift)]
pub struct NamedStruct {
    pub first: Marker,
    pub label: String,
    pub second: Marker,
}

#[derive(Debug, PartialEq, TypeSift)]
pub struct TupleStruct(pub Marker, pub Option<Marker>);

#[derive(Debug, PartialEq, TypeSift)]
pub struct UnitStruct;

#[derive(Debug, PartialEq, TypeSift)]
pub struct EmptyBraced {}

#[derive(Debug, PartialEq, TypeSift)]
pub struct EmptyTuple();

/// The named variant declares `b` before `a`, so declaration order differs from alphabetical order.
#[derive(Debug, PartialEq, TypeSift)]
pub enum Variants {
    Unit,
    Tuple(Marker, Marker),
    Named { b: Marker, a: Marker },
}

#[derive(Debug, PartialEq, TypeSift)]
pub enum SingleVariant {
    Only(Marker),
}

#[derive(TypeSift)]
pub enum Never {}

#[derive(Debug, PartialEq, TypeSift)]
#[repr(u8)]
pub enum Discriminants {
    A(Marker) = 1,
    B = 5,
}

#[derive(Debug, PartialEq, TypeSift)]
pub struct Wrapper<T, const N: usize>
where
    T: Clone,
{
    pub items: [T; N],
    pub extra: Option<T>,
}

#[derive(Debug, PartialEq, TypeSift)]
pub struct WithDefault<T = Marker> {
    pub value: T,
}

#[derive(Debug, PartialEq, TypeSift)]
pub struct Pair<A, B> {
    pub left: A,
    pub right: B,
}

#[derive(Debug, PartialEq, TypeSift)]
pub struct Nested<T: Clone> {
    pub inner: Wrapper<Tree<T>, 1>,
}

/// `M` appears only in `PhantomData`.
#[derive(Debug, PartialEq, TypeSift)]
pub struct Typed<M> {
    pub raw: u64,
    pub marker: PhantomData<M>,
}

/// Parameter and field names that match the short names the derive could have used.
#[derive(Debug, PartialEq, TypeSift)]
pub struct HygieneNames<T, B, F> {
    pub visitor: T,
    pub value: B,
    pub f: F,
}

#[derive(Debug, PartialEq, TypeSift)]
pub struct RawIdents {
    pub r#type: Marker,
    pub r#match: Marker,
}

/// `b` is removed by `#[cfg]` before the derive runs, so its type needs no `TypeSift` impl.
#[derive(Debug, TypeSift)]
pub struct WithCfg {
    pub a: Marker,
    #[cfg(any())]
    pub b: std::cell::RefCell<u32>,
}

/// `cache` has no `TypeSift` impl. `hidden` has one, but is skipped anyway.
#[derive(Debug, TypeSift)]
pub struct WithSkipped {
    pub visible: Marker,
    /// Other attributes, doc comments included, can sit next to `skip`.
    #[typesift(skip)]
    pub cache: RefCell<Vec<Marker>>,
    #[typesift(skip)]
    pub hidden: Marker,
    pub also_visible: Marker,
}

#[derive(Debug, TypeSift)]
pub struct TupleWithSkipped(pub Marker, #[typesift(skip)] pub Marker, pub Marker);

#[derive(Debug, TypeSift)]
pub enum VariantsWithSkipped {
    Named {
        visible: Marker,
        #[typesift(skip)]
        hidden: Marker,
        #[typesift(skip)]
        lock: Mutex<Marker>,
    },
    Tuple(#[typesift(skip)] Marker, Marker),
    AllSkipped(#[typesift(skip)] Cell<u32>),
}

/// `T` still needs `TypeSift`, although the only other use of it is in a skipped field.
#[derive(Debug, TypeSift)]
pub struct GenericWithSkipped<T> {
    pub value: T,
    #[typesift(skip)]
    pub weak: Weak<T>,
}

/// Stands in for a type from another crate: it has no `TypeSift` impl and cannot get one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Foreign(pub u32);

/// `foreign` has no impl. `markers` has one, but `leaf` stops the walk at it anyway.
#[derive(Debug, TypeSift)]
pub struct WithLeaf {
    pub visible: Marker,
    #[typesift(leaf)]
    pub foreign: Foreign,
    #[typesift(leaf)]
    pub markers: Vec<Marker>,
}

#[derive(Debug, TypeSift)]
pub struct TupleWithLeaf(#[typesift(leaf)] pub Foreign, pub Marker);

#[derive(Debug, TypeSift)]
pub enum VariantsWithLeaf {
    Named {
        #[typesift(leaf)]
        foreign: Foreign,
        visible: Marker,
    },
    Tuple(#[typesift(leaf)] Vec<Marker>, Marker),
}

/// Both arguments in one type.
#[derive(Debug, TypeSift)]
pub struct SkipAndLeaf {
    #[typesift(skip)]
    pub lock: Mutex<Marker>,
    #[typesift(leaf)]
    pub foreign: Foreign,
    pub visible: Marker,
}

/// A foreign type whose parts are worth searching.
#[derive(Debug)]
pub struct ForeignPair(pub Marker, pub Marker);

/// Offers the pair itself and both halves. `offer` reports one value without looking inside it.
pub fn visit_foreign_pair<'a, S: Sifter<'a>>(
    pair: &'a ForeignPair,
    sift: &mut S,
) -> ControlFlow<S::Break> {
    sift.offer(pair)?;
    sift.offer(&pair.0)?;
    sift.offer(&pair.1)
}

/// Offers only the second half: a `with` function decides what the search sees.
pub fn visit_second_only<'a, S: Sifter<'a>>(
    pair: &'a ForeignPair,
    sift: &mut S,
) -> ControlFlow<S::Break> {
    sift.offer(&pair.1)
}

/// Walks a value that does implement `TypeSift`. A slice never offers itself, so the `Vec` around
/// it is not found.
pub fn walk_markers<'a, S: Sifter<'a>>(
    markers: &'a [Marker],
    sift: &mut S,
) -> ControlFlow<S::Break> {
    sift.walk(markers)
}

#[derive(Debug, TypeSift)]
pub struct WithCustom {
    pub visible: Marker,
    #[typesift(with = visit_foreign_pair)]
    pub pair: ForeignPair,
    #[typesift(with = walk_markers)]
    pub markers: Vec<Marker>,
}

#[derive(Debug, TypeSift)]
pub enum VariantsWithCustom {
    Pair(#[typesift(with = visit_second_only)] ForeignPair, Marker),
    Named {
        #[typesift(with = visit_foreign_pair)]
        pair: ForeignPair,
    },
}

#[derive(Debug, TypeSift)]
pub struct SelfRef {
    pub marker: Marker,
    pub children: Vec<Self>,
}

#[derive(Debug, TypeSift)]
pub struct MutualA {
    pub marker: Marker,
    pub bs: Vec<MutualB>,
}

#[derive(Debug, TypeSift)]
pub struct MutualB {
    pub marker: Marker,
    pub a: Option<Box<MutualA>>,
}

/// Local items that shadow names the generated code must not rely on.
pub mod shadow {
    use typesift::TypeSift;

    pub struct ControlFlow;
    pub struct Continue;
    pub struct Break;
    pub struct FnMut;

    #[derive(Debug, TypeSift)]
    pub struct Shadowed {
        pub id: super::Marker,
    }
}

/// The derive is used by path, without importing `TypeSift`.
pub mod no_import {
    #[derive(Debug, typesift::TypeSift)]
    pub struct NotImported(pub u8);
}
