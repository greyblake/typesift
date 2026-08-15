//! Integration tests for `typesift`, kept outside the published crate.
//!
//! `fixtures` is public because `tests/no_alloc.rs` is a separate test binary that uses it.
//! Everything else is compiled only for tests.

pub mod fixtures;

#[cfg(doctest)]
mod compile_fail;

#[cfg(test)]
mod helpers;

#[cfg(test)]
mod consistency;
#[cfg(test)]
mod control_flow;
#[cfg(test)]
mod derive;
#[cfg(test)]
mod lifetimes;
#[cfg(test)]
mod oracle;
#[cfg(test)]
mod scale;
#[cfg(test)]
mod semantics;
#[cfg(test)]
mod std_impls;
