//! Find every value of a given type inside a nested data structure.
//!
//! ```
//! use typesift::TypeSift;
//!
//! #[derive(Debug, PartialEq, TypeSift)]
//! struct UserId(i32);
//!
//! #[derive(TypeSift)]
//! struct User {
//!     id: UserId,
//!     name: String,
//!     friend_ids: Vec<UserId>,
//! }
//!
//! let user = User {
//!     id: UserId(1),
//!     name: "Alice".to_string(),
//!     friend_ids: vec![UserId(4), UserId(43)],
//! };
//!
//! assert_eq!(user.sift::<UserId>(), [&UserId(1), &UserId(4), &UserId(43)]);
//! assert_eq!(user.sift::<String>(), ["Alice"]);
//! ```
//!
//! Values are matched by comparing [`TypeId`](std::any::TypeId)s. After monomorphization those
//! comparisons are between constants, so optimized builds reduce a traversal to plain field
//! access, with no allocation and no dynamic dispatch. The trade-off is that every traversed type
//! and every searched type must be `'static`.

use std::any::Any;
use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::convert::Infallible;
use std::marker::PhantomData;
use std::num::NonZero;
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

pub use typesift_macros::TypeSift;

/// A type that can be searched for nested values of any `'static` type.
///
/// Only [`visit`](TypeSift::visit) needs to be implemented. [`sift`](TypeSift::sift)
/// and [`sift_each`](TypeSift::sift_each) are built on top of it and should not be
/// overridden.
///
/// Usually implemented with `#[derive(TypeSift)]`. A manual implementation offers `self` to the
/// visitor with [`visit_self`], then visits every field, propagating [`ControlFlow::Break`] with `?`:
///
/// ```
/// use std::ops::ControlFlow;
///
/// use typesift::{TypeSift, visit_self};
///
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// impl TypeSift for Point {
///     fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
///     where
///         F: FnMut(&'a T) -> ControlFlow<B>,
///     {
///         visit_self::<Self, T, B, F>(self, visitor)?;
///         self.x.visit::<T, B, F>(visitor)?;
///         self.y.visit::<T, B, F>(visitor)
///     }
/// }
///
/// assert_eq!(Point { x: 1, y: 2 }.sift::<i32>(), [&1, &2]);
/// ```
pub trait TypeSift: 'static {
    /// Calls `visitor` with every value of type `T` reachable from `self`, `self` included, in
    /// pre-order: a value comes before the values nested inside it.
    ///
    /// The traversal stops as soon as `visitor` returns [`ControlFlow::Break`], and that break is
    /// returned.
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>;

    /// Collects references to every nested value of type `T`.
    fn sift<T: 'static>(&self) -> Vec<&T> {
        let mut values = Vec::new();
        self.sift_each::<T>(|value| values.push(value));
        values
    }

    /// Calls `f` with every nested value of type `T`, without allocating.
    fn sift_each<'a, T: 'static>(&'a self, mut f: impl FnMut(&'a T)) {
        let ControlFlow::Continue(()) = self.visit::<T, Infallible, _>(&mut |value: &'a T| {
            f(value);
            ControlFlow::Continue(())
        });
    }
}

/// Offers `value` itself to `visitor` if its type is `T`.
///
/// This is the first step of every [`TypeSift::visit`] implementation.
pub fn visit_self<'a, S: 'static, T: 'static, B, F>(value: &'a S, visitor: &mut F) -> ControlFlow<B>
where
    F: FnMut(&'a T) -> ControlFlow<B>,
{
    match (value as &dyn Any).downcast_ref::<T>() {
        Some(matched) => visitor(matched),
        None => ControlFlow::Continue(()),
    }
}

macro_rules! impl_type_sift_for_leaf {
    ($($leaf:ty),* $(,)?) => {$(
        impl TypeSift for $leaf {
            fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
            where
                F: FnMut(&'a T) -> ControlFlow<B>,
            {
                visit_self::<Self, T, B, F>(self, visitor)
            }
        }
    )*};
}

impl_type_sift_for_leaf!(
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    f32,
    f64,
    bool,
    char,
    (),
    String,
    PathBuf,
    Duration,
    NonZero<i8>,
    NonZero<i16>,
    NonZero<i32>,
    NonZero<i64>,
    NonZero<i128>,
    NonZero<isize>,
    NonZero<u8>,
    NonZero<u16>,
    NonZero<u32>,
    NonZero<u64>,
    NonZero<u128>,
    NonZero<usize>,
);

/// A `PhantomData` holds no value, so only the marker itself can be found.
impl<X: ?Sized + 'static> TypeSift for PhantomData<X> {
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        visit_self::<Self, T, B, F>(self, visitor)
    }
}

// `str` and slices are unsized, and `T` is always sized, so they are never offered to the visitor
// themselves. They exist so that `&'static str`, `Box<str>`, `Rc<[X]>` and the like work.

impl TypeSift for str {
    fn visit<'a, T: 'static, B, F>(&'a self, _visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        ControlFlow::Continue(())
    }
}

impl<X: TypeSift> TypeSift for [X] {
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        self.iter()
            .try_for_each(|item| item.visit::<T, B, F>(visitor))
    }
}

macro_rules! impl_type_sift_for_collection {
    ($($collection:ident<X $(, $param:ident)*>),* $(,)?) => {$(
        impl<X: TypeSift $(, $param: 'static)*> TypeSift for $collection<X $(, $param)*> {
            fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
            where
                F: FnMut(&'a T) -> ControlFlow<B>,
            {
                visit_self::<Self, T, B, F>(self, visitor)?;
                self.iter().try_for_each(|item| item.visit::<T, B, F>(visitor))
            }
        }
    )*};
}

impl_type_sift_for_collection!(
    Vec<X>,
    VecDeque<X>,
    LinkedList<X>,
    BTreeSet<X>,
    BinaryHeap<X>,
    HashSet<X, S>,
);

macro_rules! impl_type_sift_for_map {
    ($($map:ident<K, V $(, $param:ident)*>),* $(,)?) => {$(
        impl<K: TypeSift, V: TypeSift $(, $param: 'static)*> TypeSift for $map<K, V $(, $param)*> {
            fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
            where
                F: FnMut(&'a T) -> ControlFlow<B>,
            {
                visit_self::<Self, T, B, F>(self, visitor)?;
                self.iter().try_for_each(|(key, value)| {
                    key.visit::<T, B, F>(visitor)?;
                    value.visit::<T, B, F>(visitor)
                })
            }
        }
    )*};
}

impl_type_sift_for_map!(BTreeMap<K, V>, HashMap<K, V, S>);

impl<X: TypeSift, const N: usize> TypeSift for [X; N] {
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        visit_self::<Self, T, B, F>(self, visitor)?;
        self.iter()
            .try_for_each(|item| item.visit::<T, B, F>(visitor))
    }
}

macro_rules! impl_type_sift_for_pointer {
    ($($pointer:ty),* $(,)?) => {$(
        impl<X: ?Sized + TypeSift> TypeSift for $pointer {
            fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
            where
                F: FnMut(&'a T) -> ControlFlow<B>,
            {
                visit_self::<Self, T, B, F>(self, visitor)?;
                (**self).visit::<T, B, F>(visitor)
            }
        }
    )*};
}

impl_type_sift_for_pointer!(&'static X, Box<X>, Rc<X>, Arc<X>);

/// Visited through `Deref`: both variants are searched as `X`, so `Cow::<str>::Owned` yields no
/// `String`.
impl<X: ?Sized + ToOwned + TypeSift> TypeSift for Cow<'static, X> {
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        visit_self::<Self, T, B, F>(self, visitor)?;
        (**self).visit::<T, B, F>(visitor)
    }
}

impl<X: TypeSift> TypeSift for Reverse<X> {
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        visit_self::<Self, T, B, F>(self, visitor)?;
        self.0.visit::<T, B, F>(visitor)
    }
}

impl<X: TypeSift> TypeSift for Option<X> {
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        visit_self::<Self, T, B, F>(self, visitor)?;
        match self {
            Some(value) => value.visit::<T, B, F>(visitor),
            None => ControlFlow::Continue(()),
        }
    }
}

impl<X: TypeSift, E: TypeSift> TypeSift for Result<X, E> {
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        visit_self::<Self, T, B, F>(self, visitor)?;
        match self {
            Ok(value) => value.visit::<T, B, F>(visitor),
            Err(error) => error.visit::<T, B, F>(visitor),
        }
    }
}

/// Implements `TypeSift` for the tuple of all given elements, then recurses on all but the first.
macro_rules! impl_type_sift_for_tuples {
    () => {};
    ($param:ident $binding:ident $(, $rest_param:ident $rest_binding:ident)*) => {
        impl<$param: TypeSift $(, $rest_param: TypeSift)*> TypeSift for ($param, $($rest_param,)*) {
            fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
            where
                F: FnMut(&'a T) -> ControlFlow<B>,
            {
                visit_self::<Self, T, B, F>(self, visitor)?;
                let ($binding, $($rest_binding,)*) = self;
                $binding.visit::<T, B, F>(visitor)?;
                $($rest_binding.visit::<T, B, F>(visitor)?;)*
                ControlFlow::Continue(())
            }
        }

        impl_type_sift_for_tuples!($($rest_param $rest_binding),*);
    };
}

impl_type_sift_for_tuples!(
    X0 x0, X1 x1, X2 x2, X3 x3, X4 x4, X5 x5, X6 x6, X7 x7, X8 x8, X9 x9, X10 x10, X11 x11
);
