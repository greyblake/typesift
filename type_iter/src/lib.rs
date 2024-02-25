#![feature(specialization)]

/// If __Container__ implements __TypeIter<T>__, then __Container__ can be traversed over all nested __T__
/// that it contains.
///
/// Note: It would be great to get rid of `Box` someday, but the moment it does not work well with `#![feature(specialization)]`.
pub trait TypeIter<T> {
    fn type_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a T> + 'a>;
}

/// This crate exist only to facilitate usage of TypeIter implementation.
/// It allows a consumer to explicitly specify `T` type used  in `TypeIter<T>` implementation.
pub trait TypeValues {
    fn type_values<'a, T>(&'a self) -> impl Iterator<Item = &'a T>
    where
        Self: TypeIter<T>,
        T: 'a;
}

impl<C> TypeValues for C {
    fn type_values<'a, T>(&'a self) -> impl Iterator<Item = &'a T>
    where
        Self: TypeIter<T>,
        T: 'a,
    {
        self.type_iter()
    }
}

macro_rules! impl_type_iter_for_primitive {
    ($($primitive_type:ty),*) => {
        $(
            impl<T> TypeIter<T> for $primitive_type {
                default fn type_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a T> + 'a> {
                    Box::new(std::iter::empty())
                }
            }
            impl TypeIter<$primitive_type> for $primitive_type {
                fn type_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a $primitive_type> + 'a> {
                    Box::new(std::iter::once(self))
                }
            }
        )*
    };
}

impl_type_iter_for_primitive!(i8);
impl_type_iter_for_primitive!(i16);
impl_type_iter_for_primitive!(i32);
impl_type_iter_for_primitive!(i64);
impl_type_iter_for_primitive!(i128);
impl_type_iter_for_primitive!(u8);
impl_type_iter_for_primitive!(u16);
impl_type_iter_for_primitive!(u32);
impl_type_iter_for_primitive!(u64);
impl_type_iter_for_primitive!(u128);
impl_type_iter_for_primitive!(f32);
impl_type_iter_for_primitive!(f64);
impl_type_iter_for_primitive!(bool);
impl_type_iter_for_primitive!(char);

macro_rules! impl_type_iter_for_collection {
    ($($collection_type:ident),*) => {
        $(
            impl<T, Id> TypeIter<Id> for $collection_type <T>
            where
                T: TypeIter<Id>,
            {
                default fn type_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Id> + 'a> {
                    Box::new(self.iter().flat_map(|x| x.type_iter()))
                }
            }
        )*
    };
}

macro_rules! impl_type_iter_for_kv_collection {
    ($($collection_type:ident),*) => {
        $(
            impl<K, V, Id> TypeIter<Id> for $collection_type <K, V>
            where
                K: TypeIter<Id>,
                V: TypeIter<Id>,
            {
                default fn type_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Id> + 'a> {
                    let keys_iter = self.keys().flat_map(|x| x.type_iter());
                    let values_iter = self.values().flat_map(|x| x.type_iter());
                    let iter = keys_iter.chain(values_iter);
                    Box::new(iter)
                }
            }
        )*
    };
}

use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};

impl_type_iter_for_collection!(Vec);
impl_type_iter_for_collection!(VecDeque);
impl_type_iter_for_collection!(LinkedList);
impl_type_iter_for_collection!(HashSet);
impl_type_iter_for_collection!(BTreeSet);
impl_type_iter_for_collection!(BinaryHeap);
impl_type_iter_for_kv_collection!(HashMap);
impl_type_iter_for_kv_collection!(BTreeMap);
