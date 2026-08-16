//! `uuid`, behind the `uuid` feature.
//!
//! A `Uuid` is a leaf: it is found as itself, and the bytes inside it are never searched.

use std::ops::ControlFlow;

use uuid::Uuid;

use crate::{TypeSift, impl_type_sift_for_leaf};

impl_type_sift_for_leaf!(Uuid);
