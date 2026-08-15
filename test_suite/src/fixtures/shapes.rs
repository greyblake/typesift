//! Every shape the derive accepts, including names that could clash with generated code.

use std::marker::PhantomData;

use typesift::TypeSift;

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
