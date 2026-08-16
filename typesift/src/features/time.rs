//! `time`, behind the `time` feature.
//!
//! Every type is a leaf: an `OffsetDateTime` is found as itself, and the date, time and offset it
//! is built from are not searched.

use std::ops::ControlFlow;

use time::{Date, Duration, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

use crate::{TypeSift, impl_type_sift_for_leaf};

impl_type_sift_for_leaf!(
    Date,
    Duration,
    OffsetDateTime,
    PrimitiveDateTime,
    Time,
    UtcOffset,
);
