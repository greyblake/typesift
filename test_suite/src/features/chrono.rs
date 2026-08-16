//! The `chrono` feature: every type is a leaf.

use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeDelta, Utc};
use typesift::TypeSift;

use crate::helpers::{assert_same_refs, check_consistency};

#[derive(Debug, TypeSift)]
struct Event {
    at: DateTime<Utc>,
    day: NaiveDate,
    every: Option<TimeDelta>,
    name: String,
}

fn event() -> Event {
    Event {
        at: DateTime::from_timestamp(1_700_000_000, 0).expect("a valid timestamp"),
        day: NaiveDate::from_ymd_opt(2026, 9, 16).expect("a valid date"),
        every: Some(TimeDelta::hours(1)),
        name: "launch".to_string(),
    }
}

#[test]
fn chrono_types_are_found_wherever_they_are_nested() {
    let event = event();
    assert_same_refs(&event.sift::<DateTime<Utc>>(), &[&event.at]);
    assert_same_refs(&event.sift::<NaiveDate>(), &[&event.day]);
    assert_eq!(event.sift::<TimeDelta>().len(), 1);
    assert_eq!(event.sift::<String>(), ["launch"]);
}

#[test]
fn chrono_types_are_leaves() {
    let event = event();
    // A `DateTime` holds a naive date and time, and a `TimeDelta` holds seconds and nanoseconds.
    // Neither is searched.
    assert!(event.sift::<NaiveDateTime>().is_empty());
    assert!(event.sift::<i64>().is_empty());
    assert!(event.sift::<i32>().is_empty());
    assert!(event.sift::<u32>().is_empty());
}

#[test]
fn entry_points_agree_for_chrono() {
    check_consistency::<_, DateTime<Utc>>(&event());
    check_consistency::<_, NaiveDate>(&event());
    check_consistency::<_, TimeDelta>(&event());
    check_consistency::<_, Event>(&event());
}
