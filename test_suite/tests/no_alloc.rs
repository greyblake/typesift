//! `sift_each` and `visit` must not allocate.
//!
//! A separate test binary, because it installs a counting global allocator.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::convert::Infallible;
use std::ops::ControlFlow;

use typesift::TypeSift;
use typesift_test_suite::fixtures::containers::{MARKER_COUNT, all_containers};
use typesift_test_suite::fixtures::ids::{Absent, Marker, UserId};
use typesift_test_suite::fixtures::org::sample_org;

thread_local! {
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

/// Counts allocations per thread, so tests running in parallel don't disturb each other.
struct CountingAllocator;

fn record_allocation() {
    // `try_with` fails only while the thread is being torn down.
    let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record_allocation();
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// Runs `f` and returns its result with the number of allocations it made on this thread.
fn allocations_during<R>(f: impl FnOnce() -> R) -> (R, usize) {
    let before = ALLOCATIONS.with(Cell::get);
    let result = f();
    (result, ALLOCATIONS.with(Cell::get) - before)
}

// A1
#[test]
fn sift_each_does_not_allocate() {
    let org = sample_org();
    let all = all_containers();

    let mut seen = 0;
    let (_, allocations) = allocations_during(|| {
        org.sift_each::<UserId>(|_| seen += 1);
        all.sift_each::<Marker>(|_| seen += 1);
    });
    assert_eq!(allocations, 0);
    assert_eq!(seen, 20 + MARKER_COUNT);
}

// A2
#[test]
fn visit_does_not_allocate() {
    let org = sample_org();
    let all = all_containers();

    let (flow, allocations) = allocations_during(|| {
        all.visit::<Marker, u32, _>(&mut |marker: &Marker| {
            if marker.0 == 20 {
                ControlFlow::Break(marker.0)
            } else {
                ControlFlow::Continue(())
            }
        })
    });
    assert_eq!(allocations, 0);
    assert_eq!(flow, ControlFlow::Break(20));

    let (flow, allocations) = allocations_during(|| {
        org.visit::<UserId, Infallible, _>(&mut |_: &UserId| ControlFlow::Continue(()))
    });
    assert_eq!(allocations, 0);
    assert!(flow.is_continue());
}

// A3
#[test]
fn sift_without_matches_does_not_allocate() {
    let org = sample_org();
    let (found, allocations) = allocations_during(|| org.sift::<Absent>());
    assert_eq!(allocations, 0);
    assert!(found.is_empty());
}

// A4: proves the counter works.
#[test]
fn sift_with_matches_allocates() {
    let org = sample_org();
    let (found, allocations) = allocations_during(|| org.sift::<UserId>());
    assert!(allocations > 0);
    assert_eq!(found.len(), 20);
}
