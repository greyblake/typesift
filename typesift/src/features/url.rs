//! `url`, behind the `url` feature.
//!
//! A `Url` is a leaf: it is found as itself, and the string it is parsed from is not searched.

use std::ops::ControlFlow;

use url::Url;

use crate::{TypeSift, impl_type_sift_for_leaf};

impl_type_sift_for_leaf!(Url);
