//! `chrono`, behind the `chrono` feature.
//!
//! Every type is a leaf: a `DateTime` is found as itself, and the naive date and time it is built
//! from are not searched.

use std::ops::ControlFlow;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone};

use crate::{TypeSift, impl_type_sift_for_leaf};

impl_type_sift_for_leaf!(NaiveDate, NaiveDateTime, NaiveTime, TimeDelta);

impl<Tz: TimeZone + 'static> TypeSift for DateTime<Tz>
where
    Tz::Offset: 'static,
{
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        self.visit_self::<T, B, F>(visitor)
    }
}
