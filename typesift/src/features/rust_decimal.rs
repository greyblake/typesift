//! `rust_decimal`, behind the `rust_decimal` feature.
//!
//! A `Decimal` is a leaf: it is found as itself, and the integers it is made of are not searched.

use std::ops::ControlFlow;

use rust_decimal::Decimal;

use crate::{TypeSift, impl_type_sift_for_leaf};

impl_type_sift_for_leaf!(Decimal);
