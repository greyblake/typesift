//! `sift`, `sift_each` and `visit` must agree on every fixture.

use std::collections::BTreeMap;
use std::rc::Rc;
use std::time::Duration;

use crate::fixtures::containers::{AllContainers, Leaves, all_containers};
use crate::fixtures::ids::{Absent, Marker, NodeId, TaskId, UserId};
use crate::fixtures::json::{Json, sample_json};
use crate::fixtures::org::{Org, Task, Team, sample_org};
use crate::fixtures::shapes::{MutualA, MutualB, SelfRef, Variants};
use crate::fixtures::trees::{Entry, Folder, List, Tree, list, numbered_tree, shared_folders};
use crate::helpers::{check_consistency, count};

macro_rules! check {
    ($root:expr => $($ty:ty),+ $(,)?) => {$(
        check_consistency::<_, $ty>($root);
    )+};
}

// C1, C2, C3
#[test]
fn org() {
    check!(&sample_org() =>
        Org, UserId, TaskId, Task, Team, String, Duration, Option<UserId>, Vec<Task>, Absent);
}

#[test]
fn containers() {
    check!(&all_containers() =>
        AllContainers, Marker, Option<Marker>, (), Vec<Marker>, Leaves, u8, String, Absent);
}

#[test]
fn trees() {
    check!(&numbered_tree(3, 2) => Tree<NodeId>, NodeId, Vec<Tree<NodeId>>, Absent);
    check!(&list(50) => List, u32, Option<Box<List>>, Box<List>);
    check!(&shared_folders() => Folder, Entry, String, u64, Rc<Folder>);
}

#[test]
fn json() {
    check!(&sample_json() => Json, String, i64, bool, BTreeMap<String, Json>);
}

#[test]
fn shapes() {
    let variants = [
        Variants::Unit,
        Variants::Tuple(Marker(1), Marker(2)),
        Variants::Named {
            b: Marker(3),
            a: Marker(4),
        },
    ];
    check!(&variants => Variants, Marker);

    let self_ref = SelfRef {
        marker: Marker(1),
        children: vec![SelfRef {
            marker: Marker(2),
            children: Vec::new(),
        }],
    };
    check!(&self_ref => SelfRef, Marker, Vec<SelfRef>);

    let mutual = MutualA {
        marker: Marker(1),
        bs: vec![MutualB {
            marker: Marker(2),
            a: Some(Box::new(MutualA {
                marker: Marker(3),
                bs: Vec::new(),
            })),
        }],
    };
    check!(&mutual => MutualA, MutualB, Marker);
}

// C4
#[test]
fn a_struct_count_is_the_root_plus_its_fields() {
    let org = sample_org();
    let fields = |org: &Org| {
        count::<_, UserId>(&org.name)
            + count::<_, UserId>(&org.users)
            + count::<_, UserId>(&org.teams)
            + count::<_, UserId>(&org.tasks)
    };
    assert_eq!(count::<_, UserId>(&org), fields(&org));

    let task = &org.tasks[0];
    let task_fields = count::<_, Task>(&task.id)
        + count::<_, Task>(&task.title)
        + count::<_, Task>(&task.assignee)
        + count::<_, Task>(&task.status)
        + count::<_, Task>(&task.blocked_by)
        + count::<_, Task>(&task.subtasks)
        + count::<_, Task>(&task.estimate)
        + count::<_, Task>(&task.labels);
    assert_eq!(count::<_, Task>(task), 1 + task_fields);
}
