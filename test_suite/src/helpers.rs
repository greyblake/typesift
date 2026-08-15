use std::any::Any;
use std::convert::Infallible;
use std::ops::ControlFlow;

use typesift::TypeSift;

/// Clones and sorts found values, for comparing results from unordered containers.
pub fn sorted<T: Ord + Clone>(found: Vec<&T>) -> Vec<T> {
    let mut values: Vec<T> = found.into_iter().cloned().collect();
    values.sort();
    values
}

/// Asserts that both slices hold references to the same places, in the same order.
#[track_caller]
pub fn assert_same_refs<T>(found: &[&T], expected: &[&T]) {
    assert_eq!(found.len(), expected.len(), "number of references");
    for (index, (found, expected)) in found.iter().zip(expected).enumerate() {
        assert!(
            std::ptr::eq(*found, *expected),
            "reference {index} points somewhere else"
        );
    }
}

pub fn count<S: TypeSift, T: 'static>(root: &S) -> usize {
    root.sift::<T>().len()
}

pub fn via_visit<'a, S: TypeSift, T: 'static>(root: &'a S) -> Vec<&'a T> {
    let mut found = Vec::new();
    let ControlFlow::Continue(()) = root.visit::<T, Infallible, _>(&mut |value: &'a T| {
        found.push(value);
        ControlFlow::Continue(())
    });
    found
}

pub fn via_sift_each<S: TypeSift, T: 'static>(root: &S) -> Vec<&T> {
    let mut found = Vec::new();
    root.sift_each::<T>(|value| found.push(value));
    found
}

/// Breaks on the `k`-th value (counting from 1) and returns the result with the number of calls.
pub fn break_at<'a, S: TypeSift, T: 'static>(root: &'a S, k: usize) -> (ControlFlow<&'a T>, usize) {
    let mut calls = 0;
    let flow = root.visit::<T, &'a T, _>(&mut |value: &'a T| {
        calls += 1;
        if calls == k {
            ControlFlow::Break(value)
        } else {
            ControlFlow::Continue(())
        }
    });
    (flow, calls)
}

/// Checks that `sift`, `sift_each` and `visit` agree, including a break at every position.
///
/// Quadratic in the number of matches, so keep inputs small.
#[track_caller]
pub fn check_consistency<S: TypeSift, T: 'static>(root: &S) {
    let expected = root.sift::<T>();
    assert_same_refs(&via_sift_each::<S, T>(root), &expected);
    assert_same_refs(&via_visit::<S, T>(root), &expected);

    for (index, value) in expected.iter().enumerate() {
        let (flow, calls) = break_at::<S, T>(root, index + 1);
        assert_eq!(calls, index + 1, "calls before breaking at {}", index + 1);
        match flow {
            ControlFlow::Break(found) => assert!(std::ptr::eq(found, *value)),
            ControlFlow::Continue(()) => panic!("no break at {}", index + 1),
        }
    }

    let (flow, calls) = break_at::<S, T>(root, expected.len() + 1);
    assert!(flow.is_continue());
    assert_eq!(calls, expected.len());

    if let Some(root) = (root as &dyn Any).downcast_ref::<T>() {
        assert!(std::ptr::eq(expected[0], root), "the root comes first");
    }
}
