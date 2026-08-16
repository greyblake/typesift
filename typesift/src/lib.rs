//! Find every value of a given type inside a nested data structure.
//!
//! Derive [`TypeSift`](derive@TypeSift) on your types, then ask any value for all the `T`s it
//! contains: `order.sift::<UserId>()` returns a reference to every `UserId` inside `order`,
//! however deeply it is nested in structs, enums, collections, maps, options or smart pointers.
//! Nothing has to be written per field, so the search stays correct as the types grow.
//!
//! # Usage
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
//! // Collect references, in traversal order.
//! assert_eq!(user.sift::<UserId>(), [&UserId(1), &UserId(4), &UserId(43)]);
//! assert_eq!(user.sift::<String>(), ["Alice"]);
//! ```
//!
//! # Motivation
//!
//! API responses are often graphs of DTOs that refer to other resources by id. Before responding,
//! you want to load those resources, ideally with one batched query instead of one query per
//! reference, and serve them next to the graph. Collecting the ids usually takes a hand-written
//! function that walks every field, and that function silently goes stale when someone adds
//! another field holding an id.
//!
//! With `typesift` the ids are found by their type. Adding, say, `reviewer: Option<UserId>` to
//! `TaskDto` below needs no change to the loading code:
//!
//! ```
//! use std::collections::BTreeSet;
//!
//! use typesift::TypeSift;
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, TypeSift)]
//! struct UserId(u64);
//!
//! #[derive(TypeSift)]
//! struct ProjectDto {
//!     name: String,
//!     owner: UserId,
//!     tasks: Vec<TaskDto>,
//! }
//!
//! #[derive(TypeSift)]
//! struct TaskDto {
//!     title: String,
//!     assignee: Option<UserId>,
//!     comments: Vec<CommentDto>,
//! }
//!
//! #[derive(TypeSift)]
//! enum CommentDto {
//!     Text {
//!         author: UserId,
//!         body: String,
//!         mentions: Vec<UserId>,
//!     },
//!     Deleted,
//! }
//!
//! /// A linked resource, served next to the graph.
//! struct UserDto {
//!     id: UserId,
//!     name: String,
//! }
//!
//! /// The graph, plus every user it refers to.
//! struct Response {
//!     data: ProjectDto,
//!     included: Vec<UserDto>,
//! }
//!
//! /// Stands in for one batched query, such as `SELECT id, name FROM users WHERE id = ANY($1)`.
//! fn load_users(ids: &BTreeSet<UserId>) -> Vec<UserDto> {
//!     ids.iter()
//!         .map(|&id| UserDto {
//!             id,
//!             name: format!("user {}", id.0),
//!         })
//!         .collect()
//! }
//!
//! fn respond(project: ProjectDto) -> Response {
//!     // Every user the graph refers to, wherever it appears, without duplicates.
//!     let user_ids: BTreeSet<UserId> = project.sift::<UserId>().into_iter().copied().collect();
//!     Response {
//!         included: load_users(&user_ids),
//!         data: project,
//!     }
//! }
//!
//! let project = ProjectDto {
//!     name: "typesift".to_string(),
//!     owner: UserId(1),
//!     tasks: vec![
//!         TaskDto {
//!             title: "Write docs".to_string(),
//!             assignee: Some(UserId(2)),
//!             comments: vec![
//!                 CommentDto::Text {
//!                     author: UserId(3),
//!                     body: "Looks good, @owner?".to_string(),
//!                     mentions: vec![UserId(1)],
//!                 },
//!                 CommentDto::Deleted,
//!             ],
//!         },
//!         TaskDto {
//!             title: "Release".to_string(),
//!             assignee: None,
//!             comments: Vec::new(),
//!         },
//!     ],
//! };
//!
//! let response = respond(project);
//! let included: Vec<u64> = response.included.iter().map(|user| user.id.0).collect();
//! assert_eq!(included, [1, 2, 3]);
//! ```
//!
//! The same approach works for any value that is identified by its type: every `Url` to prefetch,
//! every `Email` to validate, or checking that no `Secret` ends up in a log line.
//!
//! # Iterating and stopping early
//!
//! [`sift`](TypeSift::sift) collects references into a `Vec`. [`sift_each`](TypeSift::sift_each)
//! hands each value to a closure without allocating, and [`visit`](TypeSift::visit) lets the
//! closure stop the traversal early. With `user` from the [Usage](#usage) example:
//!
//! ```
//! use std::ops::ControlFlow;
//!
//! # use typesift::TypeSift;
//! #
//! # #[derive(Debug, PartialEq, TypeSift)]
//! # struct UserId(i32);
//! #
//! # #[derive(TypeSift)]
//! # struct User {
//! #     id: UserId,
//! #     name: String,
//! #     friend_ids: Vec<UserId>,
//! # }
//! #
//! # let user = User {
//! #     id: UserId(1),
//! #     name: "Alice".to_string(),
//! #     friend_ids: vec![UserId(4), UserId(43)],
//! # };
//! #
//! // Handle each value without allocating.
//! let mut total = 0;
//! user.sift_each::<UserId>(|id| total += id.0);
//! assert_eq!(total, 48);
//!
//! // Stop at the first match.
//! let first_friend = user.visit::<UserId, &UserId, _>(&mut |id| {
//!     if id.0 == 1 {
//!         ControlFlow::Continue(())
//!     } else {
//!         ControlFlow::Break(id)
//!     }
//! });
//! assert_eq!(first_friend, ControlFlow::Break(&UserId(4)));
//! ```
//!
//! # Skipping fields
//!
//! A field marked `#[typesift(skip)]` is not searched, so its type needs no `TypeSift` impl. Use it
//! for caches, locks, handles and other fields that can't or shouldn't be searched:
//!
//! ```
//! use std::cell::RefCell;
//!
//! use typesift::TypeSift;
//!
//! #[derive(TypeSift)]
//! struct Session {
//!     user: u64,
//!     #[typesift(skip)]
//!     recent: RefCell<Vec<u64>>,
//! }
//!
//! let session = Session {
//!     user: 7,
//!     recent: RefCell::new(vec![1, 2]),
//! };
//! assert_eq!(session.sift::<u64>(), [&7]);
//! ```
//!
//! The attribute works on fields of structs and of enum variants. A type parameter still needs a
//! `TypeSift` impl even when only skipped fields use it.
//!
//! # Types from other crates
//!
//! A type you don't own can't implement `TypeSift`, because both the trait and the type would be
//! foreign to your crate. Mark such a field `#[typesift(leaf)]`: it is offered to the visitor like
//! any other value, but never looked inside, so its type only has to be `'static`.
//!
//! ```
//! use typesift::TypeSift;
//!
//! // Stands in for a type from another crate.
//! #[derive(Debug, PartialEq)]
//! struct Uuid([u8; 16]);
//!
//! #[derive(TypeSift)]
//! struct Invoice {
//!     #[typesift(leaf)]
//!     id: Uuid,
//!     amount: u64,
//! }
//!
//! let invoice = Invoice {
//!     id: Uuid([7; 16]),
//!     amount: 100,
//! };
//!
//! assert_eq!(invoice.sift::<Uuid>(), [&Uuid([7; 16])]);
//! assert_eq!(invoice.sift::<u64>(), [&100]);
//! // The bytes inside `id` are not searched.
//! assert!(invoice.sift::<[u8; 16]>().is_empty());
//! ```
//!
//! `leaf` also works on types that do implement `TypeSift`, when you want the field found but its
//! contents left alone.
//!
//! # Custom traversal
//!
//! When a field needs more than "found" or "ignored", `#[typesift(with = path)]` hands it to a
//! function of yours. The function receives a [`Sifter`], which it uses to [`offer`](Sifter::offer)
//! individual values or to [`walk`](Sifter::walk) values that implement `TypeSift`. Whatever it
//! offers is what the search sees, the field included:
//!
//! ```
//! use std::ops::ControlFlow;
//!
//! use typesift::{Sifter, TypeSift};
//!
//! // Stands in for a type from another crate.
//! struct Headers(Vec<(String, String)>);
//!
//! fn visit_headers<'a, S: Sifter<'a>>(headers: &'a Headers, sift: &mut S) -> ControlFlow<S::Break> {
//!     sift.offer(headers)?;
//!     headers.0.iter().try_for_each(|(name, value)| {
//!         sift.offer(name)?;
//!         sift.offer(value)
//!     })
//! }
//!
//! #[derive(TypeSift)]
//! struct Request {
//!     path: String,
//!     #[typesift(with = visit_headers)]
//!     headers: Headers,
//! }
//!
//! let request = Request {
//!     path: "/orders".to_string(),
//!     headers: Headers(vec![("accept".to_string(), "application/json".to_string())]),
//! };
//!
//! assert_eq!(
//!     request.sift::<String>(),
//!     ["/orders", "accept", "application/json"]
//! );
//! ```
//!
//! The function must be generic over the sifter, so it cannot be a closure. It also decides the
//! traversal order of that field, and whether the field itself is offered at all.
//!
//! # Supported types
//!
//! - Structs and enums with `#[derive(TypeSift)]`, including generic ones. Every type parameter
//!   must implement `TypeSift` itself.
//! - Integers, floats, `bool`, `char`, `()`, `String`, `PathBuf`, `Duration` and `NonZero`
//!   integers. These are matched as a whole; `String` does not expose its `char`s.
//! - `Vec`, `VecDeque`, `LinkedList`, `BTreeSet`, `BinaryHeap`, `HashSet`, arrays and slices.
//! - `BTreeMap` and `HashMap`, visiting each key before its value.
//! - `Option`, `Result`, `Box`, `Rc`, `Arc`, `&'static T`, `Reverse` and tuples of up to 12
//!   elements.
//! - `Cow<'static, T>`, searched as the `T` it dereferences to, whether borrowed or owned.
//! - `PhantomData<T>`, which holds no `T` and only matches itself.
//!
//! Other types can implement the trait by hand, as shown on [`TypeSift`](trait@TypeSift).
//!
//! # Cargo features
//!
//! Types from other crates are supported behind one feature each, all off by default:
//!
//! ```toml
//! typesift = { version = "0.1", features = ["uuid"] }
//! ```
//!
//! | Feature | Types | Treatment |
//! |---|---|---|
//! | `chrono` | `DateTime<Tz>`, `NaiveDate`, `NaiveDateTime`, `NaiveTime`, `TimeDelta` | Leaf |
//! | `time` | `OffsetDateTime`, `PrimitiveDateTime`, `Date`, `Time`, `UtcOffset`, `Duration` | Leaf |
//! | `uuid` | `uuid::Uuid` | Leaf |
//!
//! # Traversal order
//!
//! The traversal is pre-order: a value is offered before the values inside it, fields are visited
//! in declaration order and collections in their iteration order. So a value found inside a field
//! comes before any later field of the same struct. The order inside `HashMap`, `HashSet` and
//! `BinaryHeap` is unspecified.
//!
//! Every occurrence is reported. Equal values are not merged, the same `Rc` target reached twice
//! is reported twice, and when `T` is recursive (a tree node, say) both a node and the nodes
//! inside it are found. The searched value itself is included when it has type `T`.
//!
//! # Limitations
//!
//! - Every traversed type and every searched type must be `'static`, so the derive rejects types
//!   with lifetime parameters. `&'static T` fields are fine.
//! - `Cell`, `RefCell`, `Mutex` and other interior-mutability types are not supported, because
//!   they cannot hand out references to their contents for as long as the outer value is borrowed.
//!   Mark such fields `#[typesift(skip)]`.
//! - A type from another crate cannot implement the trait, because of the orphan rule. Mark such
//!   fields `#[typesift(leaf)]` to have them found without being searched.
//! - The traversal recurses once per nesting level, so very deep values (tens of thousands of
//!   levels in a debug build) can overflow the stack.
//! - The derive cannot be used on a type with a type parameter named `__T`, `__B` or `__F`.
//!
//! # How it works
//!
//! Values are matched by comparing [`TypeId`](std::any::TypeId)s. After monomorphization both
//! sides of every comparison are constants, which the optimizer can fold away.
//! [`visit`](TypeSift::visit) and [`sift_each`](TypeSift::sift_each) allocate nothing and
//! use no dynamic dispatch; only [`sift`](TypeSift::sift) allocates, for the `Vec` it returns.

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

mod features;

/// A type that can be searched for nested values of any `'static` type.
///
/// Only [`visit`](TypeSift::visit) needs to be implemented. The other methods are provided and
/// should not be overridden.
///
/// Usually implemented with `#[derive(TypeSift)]`. A manual implementation offers `self` to the
/// visitor with [`visit_self`](TypeSift::visit_self), then visits every field, propagating
/// [`ControlFlow::Break`] with `?`:
///
/// ```
/// use std::ops::ControlFlow;
///
/// use typesift::TypeSift;
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
///         self.visit_self::<T, B, F>(visitor)?;
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

    /// Offers `self` to `visitor` if its type is `T`.
    ///
    /// This is the first step of every [`visit`](TypeSift::visit) implementation. Unsized types
    /// cannot call it, and never need to: `T` is always sized, so an unsized value can never be a
    /// match.
    fn visit_self<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        Self: Sized,
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        match (self as &dyn Any).downcast_ref::<T>() {
            Some(matched) => visitor(matched),
            None => ControlFlow::Continue(()),
        }
    }

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

/// Hands values to the search from a `#[typesift(with = ...)]` function.
///
/// A `with` function is generic over the sifter, which keeps the searched type out of its
/// signature. It cannot be a closure, because closures cannot be generic:
///
/// ```
/// use std::ops::ControlFlow;
///
/// use typesift::Sifter;
///
/// struct Pair(String, String);
///
/// fn visit_pair<'a, S: Sifter<'a>>(pair: &'a Pair, sift: &mut S) -> ControlFlow<S::Break> {
///     sift.offer(pair)?;
///     sift.offer(&pair.0)?;
///     sift.offer(&pair.1)
/// }
/// ```
pub trait Sifter<'a> {
    /// What the search carries when it stops early, the `B` of [`TypeSift::visit`].
    type Break;

    /// Reports `value` if its type is the one being searched for.
    ///
    /// `X` only has to be `'static`, so this works for types that cannot implement [`TypeSift`].
    fn offer<X: 'static>(&mut self, value: &'a X) -> ControlFlow<Self::Break>;

    /// Searches `value` itself and everything inside it.
    fn walk<X: ?Sized + TypeSift>(&mut self, value: &'a X) -> ControlFlow<Self::Break>;
}

/// The [`Sifter`] that `#[derive(TypeSift)]` passes to a `with` function.
#[doc(hidden)]
pub struct Sift<'v, T, B, F> {
    visitor: &'v mut F,
    marker: PhantomData<fn(&T) -> B>,
}

#[doc(hidden)]
impl<'v, T, B, F> Sift<'v, T, B, F> {
    pub fn new(visitor: &'v mut F) -> Self {
        Self {
            visitor,
            marker: PhantomData,
        }
    }
}

impl<'a, T: 'static, B, F> Sifter<'a> for Sift<'_, T, B, F>
where
    F: FnMut(&'a T) -> ControlFlow<B>,
{
    type Break = B;

    fn offer<X: 'static>(&mut self, value: &'a X) -> ControlFlow<B> {
        match (value as &dyn Any).downcast_ref::<T>() {
            Some(matched) => (self.visitor)(matched),
            None => ControlFlow::Continue(()),
        }
    }

    fn walk<X: ?Sized + TypeSift>(&mut self, value: &'a X) -> ControlFlow<B> {
        value.visit::<T, B, F>(self.visitor)
    }
}

macro_rules! impl_type_sift_for_leaf {
    ($($leaf:ty),* $(,)?) => {$(
        impl TypeSift for $leaf {
            fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
            where
                F: FnMut(&'a T) -> ControlFlow<B>,
            {
                self.visit_self::<T, B, F>(visitor)
            }
        }
    )*};
}

// So that the `features` modules can use it, wherever they sit in the file. Nothing uses it when
// every feature is off.
#[allow(unused_imports)]
pub(crate) use impl_type_sift_for_leaf;

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
        self.visit_self::<T, B, F>(visitor)
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
                self.visit_self::<T, B, F>(visitor)?;
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
                self.visit_self::<T, B, F>(visitor)?;
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
        self.visit_self::<T, B, F>(visitor)?;
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
                self.visit_self::<T, B, F>(visitor)?;
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
        self.visit_self::<T, B, F>(visitor)?;
        (**self).visit::<T, B, F>(visitor)
    }
}

impl<X: TypeSift> TypeSift for Reverse<X> {
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        self.visit_self::<T, B, F>(visitor)?;
        self.0.visit::<T, B, F>(visitor)
    }
}

impl<X: TypeSift> TypeSift for Option<X> {
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        self.visit_self::<T, B, F>(visitor)?;
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
        self.visit_self::<T, B, F>(visitor)?;
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
                self.visit_self::<T, B, F>(visitor)?;
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
