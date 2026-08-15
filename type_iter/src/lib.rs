//! Find every value of a given type inside a nested data structure.
//!
//! ```
//! use type_iter::{TypeIter, TypeValues};
//!
//! #[derive(Debug, PartialEq, TypeIter)]
//! struct UserId(i32);
//!
//! #[derive(TypeIter)]
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
//! assert_eq!(user.type_values::<UserId>(), [&UserId(1), &UserId(4), &UserId(43)]);
//! assert_eq!(user.type_values::<String>(), ["Alice"]);
//! assert_eq!(user.find_value::<UserId>(|id| id.0 > 1), Some(&UserId(4)));
//! ```
//!
//! Values are matched by comparing [`TypeId`](std::any::TypeId)s. After monomorphization those
//! comparisons are between constants, so optimized builds reduce a traversal to plain field
//! access, with no allocation and no dynamic dispatch. The trade-off is that every traversed type
//! and every searched type must be `'static`.

use std::any::Any;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::convert::Infallible;
use std::ops::ControlFlow;

pub use type_iter_macros::TypeIter;

/// A type that can be searched for nested values of any `'static` type.
///
/// Usually implemented with `#[derive(TypeIter)]`. A manual implementation offers `self` to the
/// visitor with [`visit_self`], then visits every field, propagating [`ControlFlow::Break`] with `?`:
///
/// ```
/// use std::ops::ControlFlow;
///
/// use type_iter::{TypeIter, TypeValues, visit_self};
///
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// impl TypeIter for Point {
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
/// assert_eq!(Point { x: 1, y: 2 }.type_values::<i32>(), [&1, &2]);
/// ```
pub trait TypeIter: 'static {
    /// Calls `visitor` with every value of type `T` reachable from `self`, `self` included, in
    /// pre-order: a value comes before the values nested inside it.
    ///
    /// The traversal stops as soon as `visitor` returns [`ControlFlow::Break`], and that break is
    /// returned.
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>;
}

/// Offers `value` itself to `visitor` if its type is `T`.
///
/// This is the first step of every [`TypeIter::visit`] implementation.
pub fn visit_self<'a, S: 'static, T: 'static, B, F>(value: &'a S, visitor: &mut F) -> ControlFlow<B>
where
    F: FnMut(&'a T) -> ControlFlow<B>,
{
    match (value as &dyn Any).downcast_ref::<T>() {
        Some(matched) => visitor(matched),
        None => ControlFlow::Continue(()),
    }
}

/// Convenience methods for searching any [`TypeIter`] value.
pub trait TypeValues: TypeIter {
    /// Calls `f` with every nested value of type `T`.
    fn for_each_value<'a, T: 'static>(&'a self, mut f: impl FnMut(&'a T)) {
        let ControlFlow::Continue(()) = self.visit::<T, Infallible, _>(&mut |value: &'a T| {
            f(value);
            ControlFlow::Continue(())
        });
    }

    /// Returns the first nested value of type `T` that satisfies `predicate`, without visiting
    /// the rest.
    fn find_value<'a, T: 'static>(
        &'a self,
        mut predicate: impl FnMut(&T) -> bool,
    ) -> Option<&'a T> {
        self.visit::<T, &'a T, _>(&mut |value: &'a T| {
            if predicate(value) {
                ControlFlow::Break(value)
            } else {
                ControlFlow::Continue(())
            }
        })
        .break_value()
    }

    /// Collects references to every nested value of type `T`.
    fn type_values<T: 'static>(&self) -> Vec<&T> {
        let mut values = Vec::new();
        self.for_each_value::<T>(|value| values.push(value));
        values
    }
}

impl<C: TypeIter> TypeValues for C {}

macro_rules! impl_type_iter_for_leaf {
    ($($leaf:ty),* $(,)?) => {$(
        impl TypeIter for $leaf {
            fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
            where
                F: FnMut(&'a T) -> ControlFlow<B>,
            {
                visit_self::<Self, T, B, F>(self, visitor)
            }
        }
    )*};
}

impl_type_iter_for_leaf!(
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
);

macro_rules! impl_type_iter_for_collection {
    ($($collection:ident<X $(, $param:ident)*>),* $(,)?) => {$(
        impl<X: TypeIter $(, $param: 'static)*> TypeIter for $collection<X $(, $param)*> {
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

impl_type_iter_for_collection!(
    Vec<X>,
    VecDeque<X>,
    LinkedList<X>,
    BTreeSet<X>,
    BinaryHeap<X>,
    HashSet<X, S>,
);

macro_rules! impl_type_iter_for_map {
    ($($map:ident<K, V $(, $param:ident)*>),* $(,)?) => {$(
        impl<K: TypeIter, V: TypeIter $(, $param: 'static)*> TypeIter for $map<K, V $(, $param)*> {
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

impl_type_iter_for_map!(BTreeMap<K, V>, HashMap<K, V, S>);

impl<X: TypeIter, const N: usize> TypeIter for [X; N] {
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        visit_self::<Self, T, B, F>(self, visitor)?;
        self.iter()
            .try_for_each(|item| item.visit::<T, B, F>(visitor))
    }
}

impl<X: TypeIter> TypeIter for Box<X> {
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        visit_self::<Self, T, B, F>(self, visitor)?;
        (**self).visit::<T, B, F>(visitor)
    }
}

impl<X: TypeIter> TypeIter for Option<X> {
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

impl<X: TypeIter, E: TypeIter> TypeIter for Result<X, E> {
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

// TODO: Support tuples!
