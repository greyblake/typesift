//! `bytes`, behind the `bytes` feature.
//!
//! Both types are leaves: a buffer is found as itself, and the bytes it holds are not searched.

use std::ops::ControlFlow;

use bytes::{Bytes, BytesMut};

use crate::{TypeSift, impl_type_sift_for_leaf};

impl_type_sift_for_leaf!(Bytes, BytesMut);
