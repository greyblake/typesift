//! Stopping a traversal early with `ControlFlow::Break`.

use std::ops::ControlFlow;

use typesift::TypeSift;

use crate::fixtures::containers::{MARKER_COUNT, all_containers};
use crate::fixtures::ids::{Marker, NodeId, TaskId, UserId};
use crate::fixtures::org::{Org, Task, sample_org};
use crate::fixtures::shapes::{MutualA, MutualB, Nested, Variants, Wrapper};
use crate::fixtures::trees::{list, numbered_tree, tree_size};
use crate::helpers::break_at;

// CF1
#[test]
fn break_is_returned_from_every_position_through_every_impl() {
    let all = all_containers();
    for k in 1..=MARKER_COUNT {
        let (flow, calls) = break_at::<_, Marker>(&all, k as usize);
        assert_eq!(flow, ControlFlow::Break(&Marker(k)), "break at {k}");
        assert_eq!(calls, k as usize, "calls before breaking at {k}");
    }

    let (flow, calls) = break_at::<_, Marker>(&all, MARKER_COUNT as usize + 1);
    assert_eq!(flow, ControlFlow::Continue(()));
    assert_eq!(calls, MARKER_COUNT as usize);
}

fn find_user<'a>(org: &'a Org, wanted: UserId, calls: &mut usize) -> Option<&'a UserId> {
    org.visit::<UserId, &'a UserId, _>(&mut |id: &'a UserId| {
        *calls += 1;
        if *id == wanted {
            ControlFlow::Break(id)
        } else {
            ControlFlow::Continue(())
        }
    })
    .break_value()
}

// CF2, CF7
#[test]
fn break_on_a_condition_returns_a_reference_into_the_value() {
    let org = sample_org();

    let mut calls = 0;
    let found = find_user(&org, UserId(3), &mut calls).expect("user 3 exists");
    assert_eq!(calls, 6);
    assert!(std::ptr::eq(
        found,
        org.users.keys().nth(2).expect("three users")
    ));

    let mut calls = 0;
    assert_eq!(find_user(&org, UserId(99), &mut calls), None);
    assert_eq!(calls, 20);
}

// CF3
#[test]
fn break_deep_inside_recursion() {
    let tree = numbered_tree(4, 3);
    let size = tree_size(4, 3) as usize;
    for k in [1, 2, 5, 40, size] {
        let (flow, calls) = break_at::<_, NodeId>(&tree, k);
        assert_eq!(flow, ControlFlow::Break(&NodeId(k as u32)));
        assert_eq!(calls, k);
    }

    let list = list(1_000);
    let (flow, calls) = break_at::<_, u32>(&list, 999);
    assert_eq!(flow, ControlFlow::Break(&999));
    assert_eq!(calls, 999);
}

// CF4
#[test]
fn break_leaves_later_fields_and_variants_unvisited() {
    let variants = [
        Variants::Named {
            b: Marker(1),
            a: Marker(2),
        },
        Variants::Tuple(Marker(3), Marker(4)),
    ];
    assert_eq!(
        break_at::<_, Marker>(&variants, 2),
        (ControlFlow::Break(&Marker(2)), 2)
    );

    let nested = Nested {
        inner: Wrapper {
            items: [numbered_tree(1, 2)],
            extra: None,
        },
    };
    assert_eq!(
        break_at::<_, NodeId>(&nested, 2),
        (ControlFlow::Break(&NodeId(2)), 2)
    );

    let mutual = MutualA {
        marker: Marker(1),
        bs: vec![
            MutualB {
                marker: Marker(2),
                a: Some(Box::new(MutualA {
                    marker: Marker(3),
                    bs: Vec::new(),
                })),
            },
            MutualB {
                marker: Marker(4),
                a: None,
            },
        ],
    };
    assert_eq!(
        break_at::<_, Marker>(&mutual, 3),
        (ControlFlow::Break(&Marker(3)), 3)
    );
}

// CF5
#[test]
fn break_can_carry_any_value() {
    let org = sample_org();

    let mut position = 0;
    let flow = org.visit::<TaskId, usize, _>(&mut |id: &TaskId| {
        position += 1;
        if id.0 == 102 {
            ControlFlow::Break(position)
        } else {
            ControlFlow::Continue(())
        }
    });
    assert_eq!(flow, ControlFlow::Break(4));

    let flow = org.visit::<Task, String, _>(&mut |task: &Task| {
        if task.labels.is_empty() {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(task.title.clone())
        }
    });
    assert_eq!(flow, ControlFlow::Break("Parser".to_string()));
}

// CF6
#[test]
fn continue_everywhere_visits_everything() {
    let org = sample_org();
    let mut calls = 0;
    let flow = org.visit::<UserId, (), _>(&mut |_: &UserId| {
        calls += 1;
        ControlFlow::Continue(())
    });
    assert_eq!(flow, ControlFlow::Continue(()));
    assert_eq!(calls, 20);
}

// CF8
#[test]
fn visitor_can_keep_state() {
    let all = all_containers();
    let mut sum = 0;
    let mut max = 0;
    all.sift_each::<Marker>(|marker| {
        sum += marker.0;
        max = max.max(marker.0);
    });
    assert_eq!(sum, (1..=MARKER_COUNT).sum::<u32>());
    assert_eq!(max, MARKER_COUNT);
}
