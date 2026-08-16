//! The `rust_decimal` feature: `Decimal` is a leaf.

use rust_decimal::Decimal;
use typesift::TypeSift;

use crate::helpers::{assert_same_refs, check_consistency};

#[derive(Debug, TypeSift)]
struct Invoice {
    total: Decimal,
    lines: Vec<Decimal>,
    discount: Option<Decimal>,
    currency: String,
}

fn invoice() -> Invoice {
    Invoice {
        total: Decimal::new(12_345, 2),
        lines: vec![Decimal::new(10_000, 2), Decimal::new(2_345, 2)],
        discount: None,
        currency: "EUR".to_string(),
    }
}

#[test]
fn decimals_are_found_wherever_they_are_nested() {
    let invoice = invoice();
    assert_same_refs(
        &invoice.sift::<Decimal>(),
        &[&invoice.total, &invoice.lines[0], &invoice.lines[1]],
    );
    assert_eq!(invoice.sift::<String>(), ["EUR"]);
}

#[test]
fn decimal_is_a_leaf() {
    let invoice = invoice();
    // A `Decimal` is made of integers, none of which is searched.
    assert!(invoice.sift::<u32>().is_empty());
    assert!(invoice.sift::<i64>().is_empty());
    assert!(invoice.sift::<u64>().is_empty());
}

#[test]
fn entry_points_agree_for_rust_decimal() {
    check_consistency::<_, Decimal>(&invoice());
    check_consistency::<_, Option<Decimal>>(&invoice());
    check_consistency::<_, Invoice>(&invoice());
}
