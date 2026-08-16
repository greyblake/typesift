//! Impls for types from other crates, one module per crate, each behind a cargo feature of the
//! same name.
//!
//! Every module documents whether its types are leaves, found as themselves and never searched, or
//! walked like a container. That choice is part of the API: changing it later changes what a search
//! returns.

#[cfg(feature = "chrono")]
mod chrono;
#[cfg(feature = "rust_decimal")]
mod rust_decimal;
#[cfg(feature = "time")]
mod time;
#[cfg(feature = "uuid")]
mod uuid;
