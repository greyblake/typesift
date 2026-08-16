//! The `time` feature: every type is a leaf.

use time::{Date, Duration, Month, OffsetDateTime, PrimitiveDateTime, Time};
use typesift::TypeSift;

use crate::helpers::{assert_same_refs, check_consistency};

#[derive(Debug, TypeSift)]
struct Booking {
    at: OffsetDateTime,
    day: Date,
    every: Option<Duration>,
    name: String,
}

fn booking() -> Booking {
    Booking {
        at: OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("a valid timestamp"),
        day: Date::from_calendar_date(2026, Month::September, 16).expect("a valid date"),
        every: Some(Duration::hours(1)),
        name: "table".to_string(),
    }
}

#[test]
fn time_types_are_found_wherever_they_are_nested() {
    let booking = booking();
    assert_same_refs(&booking.sift::<OffsetDateTime>(), &[&booking.at]);
    assert_same_refs(&booking.sift::<Date>(), &[&booking.day]);
    assert_eq!(booking.sift::<Duration>().len(), 1);
    assert_eq!(booking.sift::<String>(), ["table"]);
}

#[test]
fn time_types_are_leaves() {
    let booking = booking();
    // An `OffsetDateTime` is built from a date, a time and an offset. None of them is searched.
    assert!(booking.sift::<PrimitiveDateTime>().is_empty());
    assert!(booking.sift::<Time>().is_empty());
    assert!(booking.sift::<i64>().is_empty());
    assert!(booking.sift::<i32>().is_empty());
}

#[test]
fn entry_points_agree_for_time() {
    check_consistency::<_, OffsetDateTime>(&booking());
    check_consistency::<_, Date>(&booking());
    check_consistency::<_, Duration>(&booking());
    check_consistency::<_, Booking>(&booking());
}
