//! Compile-fail cases, built only by `cargo test --doc`.
//!
//! Stable rustdoc does not check the error code of a `compile_fail` block, so any error makes it
//! pass. Each failing block is therefore followed by a block that compiles and differs only in
//! the part under test, which shows the failure comes from that part.
//!
//! # UI1: types with lifetime parameters are rejected
//!
//! ```compile_fail
//! #[derive(typesift::TypeSift)]
//! struct Borrowed<'a> {
//!     name: &'a str,
//! }
//! ```
//!
//! ```
//! #[derive(typesift::TypeSift)]
//! struct Borrowed {
//!     name: &'static str,
//! }
//! ```
//!
//! # UI2: unions are rejected
//!
//! ```compile_fail
//! #[derive(typesift::TypeSift)]
//! union Bits {
//!     int: u32,
//!     float: f32,
//! }
//! ```
//!
//! ```
//! #[derive(typesift::TypeSift)]
//! struct Bits {
//!     int: u32,
//!     float: f32,
//! }
//! ```
//!
//! # UI3: every field type needs an impl
//!
//! `Cell` and `RefCell` cannot lend references to their contents for as long as `self` lives.
//!
//! ```compile_fail
//! #[derive(typesift::TypeSift)]
//! struct Counter {
//!     hits: std::cell::Cell<u32>,
//! }
//! ```
//!
//! ```compile_fail
//! #[derive(typesift::TypeSift)]
//! struct Counter {
//!     hits: std::cell::RefCell<u32>,
//! }
//! ```
//!
//! ```
//! #[derive(typesift::TypeSift)]
//! struct Counter {
//!     hits: u32,
//! }
//! ```
//!
//! ```compile_fail
//! struct NotSift;
//!
//! #[derive(typesift::TypeSift)]
//! struct Holder {
//!     value: NotSift,
//! }
//! ```
//!
//! ```
//! #[derive(typesift::TypeSift)]
//! struct NotSift;
//!
//! #[derive(typesift::TypeSift)]
//! struct Holder {
//!     value: NotSift,
//! }
//! ```
//!
//! # UI4: a generic type needs an impl for its arguments
//!
//! ```compile_fail
//! use typesift::TypeSift;
//!
//! #[derive(TypeSift)]
//! struct Wrapper<T> {
//!     value: T,
//! }
//!
//! struct NotSift;
//!
//! Wrapper { value: NotSift }.sift::<u8>();
//! ```
//!
//! ```
//! use typesift::TypeSift;
//!
//! #[derive(TypeSift)]
//! struct Wrapper<T> {
//!     value: T,
//! }
//!
//! struct NotSift;
//!
//! Wrapper { value: 1u8 }.sift::<u8>();
//! ```
//!
//! # UI5: the searched type must be `'static`
//!
//! ```compile_fail
//! use typesift::TypeSift;
//!
//! fn count<'a>(names: &[String], _hint: &'a str) -> usize {
//!     names.sift::<&'a str>().len()
//! }
//! ```
//!
//! ```
//! use typesift::TypeSift;
//!
//! fn count<'a>(names: &[String], _hint: &'a str) -> usize {
//!     names.sift::<&'static str>().len()
//! }
//! ```
//!
//! # UI6: the searched type must be sized
//!
//! ```compile_fail
//! use typesift::TypeSift;
//!
//! let found = "text".to_string().sift::<str>().len();
//! ```
//!
//! ```
//! use typesift::TypeSift;
//!
//! let found = "text".to_string().sift::<String>().len();
//! ```
//!
//! # UI7: the trait cannot be a trait object
//!
//! ```compile_fail
//! fn search(value: &dyn typesift::TypeSift) {}
//! ```
//!
//! ```
//! fn search<S: typesift::TypeSift>(value: &S) {}
//! ```
//!
//! # UI8: results borrow the searched value
//!
//! ```compile_fail
//! use typesift::TypeSift;
//!
//! let mut names = vec!["a".to_string()];
//! let found = names.sift::<String>();
//! names.push("b".to_string());
//! assert_eq!(found.len(), 1);
//! ```
//!
//! ```
//! use typesift::TypeSift;
//!
//! let mut names = vec!["a".to_string()];
//! let found = names.sift::<String>();
//! assert_eq!(found.len(), 1);
//! names.push("b".to_string());
//! ```
//!
//! # UI9: type parameters named like the derive's own
//!
//! The derived `visit` declares `__T`, `__B` and `__F`, so a type parameter with one of those
//! names is declared twice (E0403). Kept as a known limitation.
//!
//! ```compile_fail
//! #[derive(typesift::TypeSift)]
//! struct Clash<__T> {
//!     value: __T,
//! }
//! ```
//!
//! ```compile_fail
//! #[derive(typesift::TypeSift)]
//! struct Clash<__B> {
//!     value: __B,
//! }
//! ```
//!
//! ```compile_fail
//! #[derive(typesift::TypeSift)]
//! struct Clash<__F> {
//!     value: __F,
//! }
//! ```
//!
//! ```
//! #[derive(typesift::TypeSift)]
//! struct Clash<T> {
//!     value: T,
//! }
//! ```
