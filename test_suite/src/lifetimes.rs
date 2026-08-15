//! Borrowing from results, and using the trait from generic code.

use std::ops::ControlFlow;
use std::rc::Rc;

use typesift::TypeSift;

use crate::fixtures::containers::{MARKER_COUNT, all_containers};
use crate::fixtures::ids::{Marker, NodeId, TaskId, UserId};
use crate::fixtures::json::{Json, sample_json};
use crate::fixtures::org::{Org, Team, sample_org};
use crate::fixtures::trees::numbered_tree;
use crate::helpers::{assert_same_refs, count};

fn user_ids(org: &Org) -> Vec<&UserId> {
    org.sift::<UserId>()
}

fn find_all<S: TypeSift, T: 'static>(root: &S) -> Vec<&T> {
    root.sift::<T>()
}

/// Searches for a container, then returns a reference into it.
fn first_blocker<'a>(org: &'a Org) -> Option<&'a TaskId> {
    org.visit::<Vec<TaskId>, &'a TaskId, _>(&mut |blocked_by: &'a Vec<TaskId>| {
        blocked_by
            .first()
            .map_or(ControlFlow::Continue(()), ControlFlow::Break)
    })
    .break_value()
}

// L1
#[test]
fn results_borrow_only_from_the_searched_value() {
    let org = sample_org();
    let ids = {
        let scratch = sample_org();
        assert_eq!(user_ids(&scratch).len(), 20);
        user_ids(&org)
    };
    assert_eq!(ids.len(), 20);

    let blocker = first_blocker(&org).expect("Docs is blocked");
    assert!(std::ptr::eq(blocker, &org.tasks[1].blocked_by[0]));
}

// L2
#[test]
fn generic_callers() {
    assert_eq!(count::<_, UserId>(&sample_org()), 20);
    assert_eq!(count::<_, Marker>(&all_containers()), MARKER_COUNT as usize);
    assert_eq!(count::<_, NodeId>(&numbered_tree(2, 3)), 13);
    assert_eq!(count::<_, Json>(&sample_json()), 10);

    let org = sample_org();
    assert_same_refs(&find_all::<_, Team>(&org), &org.sift::<Team>());
}

// L3
#[test]
fn called_through_references_and_smart_pointers() {
    let org = sample_org();
    let expected = org.sift::<Team>();

    let by_ref = &org;
    assert_same_refs(&by_ref.sift::<Team>(), &expected);

    let boxed = Box::new(sample_org());
    assert_eq!(boxed.sift::<UserId>().len(), 20);
    assert_same_refs(&boxed.sift::<Org>(), &[&*boxed]);

    let shared = Rc::new(sample_org());
    assert_same_refs(&shared.sift::<Org>(), &[&*shared]);
    assert_same_refs(&shared.sift::<Rc<Org>>(), &[&shared]);
}

// L4
#[test]
fn searched_from_several_threads() {
    let org = sample_org();
    let expected = org.sift::<UserId>();
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..4)
            .map(|_| scope.spawn(|| org.sift::<UserId>()))
            .collect();
        for handle in handles {
            let found = handle.join().expect("search thread panicked");
            assert_same_refs(&found, &expected);
        }
    });
}
