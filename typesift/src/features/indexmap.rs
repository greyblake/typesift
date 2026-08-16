//! `indexmap`, behind the `indexmap` feature.
//!
//! Both types are walked rather than treated as leaves, in insertion order, so a search over them
//! is deterministic. A map offers each key before its value, like the std maps.

use std::ops::ControlFlow;

use indexmap::{IndexMap, IndexSet};

use crate::{TypeSift, impl_type_sift_for_collection, impl_type_sift_for_map};

impl_type_sift_for_collection!(IndexSet<X, S>);
impl_type_sift_for_map!(IndexMap<K, V, S>);
