//! Which values are found, in what order, and what is never found.

use std::borrow::Cow;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use typesift::TypeSift;

use crate::fixtures::containers::{all_containers, leaves};
use crate::fixtures::ids::{Absent, Marker, NodeId, TaskId, TeamId, UserId};
use crate::fixtures::org::{Assignee, Org, Role, Task, Team, sample_org};
use crate::fixtures::shapes::{Pair, Typed, UnitStruct, Wrapper};
use crate::fixtures::trees::{Folder, Tree, numbered_tree, shared_folders, tree_size};
use crate::helpers::assert_same_refs;

// S1
#[test]
fn root_is_found_when_it_has_the_searched_type() {
    let org = sample_org();
    assert_same_refs(&org.sift::<Org>(), &[&org]);

    let tree = numbered_tree(2, 2);
    assert!(std::ptr::eq(tree.sift::<Tree<NodeId>>()[0], &tree));
}

// S2
#[test]
fn values_are_found_in_pre_order() {
    let org = sample_org();

    let user_ids: Vec<u32> = org.sift::<UserId>().iter().map(|id| id.0).collect();
    assert_eq!(
        user_ids,
        [1, 1, 2, 2, 1, 3, 3, 1, 1, 1, 2, 2, 2, 3, 3, 3, 3, 3, 2, 1]
    );

    let team_ids: Vec<u32> = org.sift::<TeamId>().iter().map(|id| id.0).collect();
    assert_eq!(team_ids, [10, 11, 12, 11]);

    let task_ids: Vec<u32> = org.sift::<TaskId>().iter().map(|id| id.0).collect();
    assert_eq!(task_ids, [100, 101, 103, 102, 100, 101]);

    assert_eq!(
        org.sift::<Role>(),
        [&Role::Admin, &Role::Member, &Role::Member, &Role::Guest]
    );
    assert_eq!(
        org.sift::<String>(),
        [
            "Acme",
            "Ada",
            "ada@acme.test",
            "Grace",
            "grace@acme.test",
            "Linus",
            "Core",
            "Compiler",
            "Backend",
            "Parser",
            "Lexer",
            "Unicode",
            "wontfix",
            "Docs",
        ]
    );
    assert_eq!(org.sift::<&'static str>(), [&"parser", &"p1"]);
    assert_eq!(
        org.sift::<Assignee>(),
        [
            &Assignee::User(UserId(3)),
            &Assignee::Team {
                team: TeamId(11),
                reviewer: Some(UserId(2)),
            },
            &Assignee::User(UserId(1)),
            &Assignee::Unassigned,
        ]
    );

    let node_ids: Vec<u32> = numbered_tree(3, 3)
        .sift::<NodeId>()
        .iter()
        .map(|id| id.0)
        .collect();
    assert_eq!(node_ids, (1..=tree_size(3, 3)).collect::<Vec<_>>());
}

// S3
#[test]
fn nested_values_come_before_later_fields_of_their_parent() {
    let seconds: Vec<u64> = sample_org()
        .sift::<Duration>()
        .iter()
        .map(|duration| duration.as_secs())
        .collect();
    assert_eq!(seconds, [90, 30, 120, 3600, 600]);
}

// S4, S11: three levels of recursion in both teams and tasks, checked by address.
#[test]
fn overlapping_matches_point_into_the_original_value() {
    let org = sample_org();

    let core = &org.teams[0];
    let compiler = &core.sub_teams[0];
    let backend = &compiler.sub_teams[0];
    assert_same_refs(&org.sift::<Team>(), &[core, compiler, backend]);
    assert_same_refs(
        &org.sift::<Vec<Team>>(),
        &[
            &org.teams,
            &core.sub_teams,
            &compiler.sub_teams,
            &backend.sub_teams,
        ],
    );

    let parser = &org.tasks[0];
    let lexer = &parser.subtasks[0];
    let unicode = &lexer.subtasks[0];
    let docs = &org.tasks[1];
    assert_same_refs(&org.sift::<Task>(), &[parser, lexer, unicode, docs]);
    assert_same_refs(
        &org.sift::<Vec<Task>>(),
        &[
            &org.tasks,
            &parser.subtasks,
            &lexer.subtasks,
            &unicode.subtasks,
            &docs.subtasks,
        ],
    );

    let user_ids = org.sift::<UserId>();
    let first_key = org.users.keys().next().expect("the org has users");
    assert!(std::ptr::eq(user_ids[0], first_key));
    assert!(std::ptr::eq(user_ids[1], &org.users[&UserId(1)].id));
    let last = &unicode.assignee;
    let Assignee::User(unicode_assignee) = last else {
        panic!("Unicode is assigned to a user");
    };
    assert!(std::ptr::eq(user_ids[19], unicode_assignee));

    let tree = numbered_tree(3, 2);
    let subtrees = tree.sift::<Tree<NodeId>>();
    assert_eq!(subtrees.len() as u32, tree_size(3, 2));
    assert!(std::ptr::eq(subtrees[1], &tree.children[0]));
    assert!(std::ptr::eq(
        subtrees[3],
        &tree.children[0].children[0].children[0]
    ));
}

// S5
#[test]
fn nothing_is_found_for_an_absent_type() {
    let org = sample_org();
    assert!(org.sift::<Absent>().is_empty());

    let mut called = false;
    org.sift_each::<Absent>(|_| called = true);
    assert!(!called);
}

// S6, S7, S8
#[test]
fn containers_are_found_as_whole_values_by_exact_type() {
    let all = all_containers();

    assert_same_refs(&all.sift::<Vec<Marker>>(), &[&all.vec, &all.empty_vec]);
    assert_same_refs(&all.sift::<[Marker; 2]>(), &[&all.array]);
    assert_same_refs(&all.sift::<[Marker; 0]>(), &[&all.empty_array]);
    assert_same_refs(&all.sift::<&'static Marker>(), &[&all.static_ref]);
    assert_same_refs(&all.sift::<&'static [Marker]>(), &[&all.static_slice]);
    assert_same_refs(&all.sift::<Rc<Marker>>(), &[&all.rc]);
    assert_same_refs(&all.sift::<Arc<Marker>>(), &[&all.arc]);
    assert_same_refs(
        &all.sift::<Cow<'static, [Marker]>>(),
        &[&all.cow_borrowed, &all.cow_owned],
    );
    assert_same_refs(
        &all.sift::<Option<Marker>>(),
        &[&all.some, &all.none, &***all.layered],
    );
    assert_same_refs(&all.sift::<Result<Marker, Marker>>(), &[&all.ok, &all.err]);
    assert_same_refs(&all.sift::<(Marker,)>(), &[&all.tuple.1]);
    assert_same_refs(&all.sift::<PhantomData<Marker>>(), &[&all.phantom]);
    assert_eq!(all.sift::<()>().len(), 2);

    assert!(all.sift::<Vec<Vec<Marker>>>().is_empty());
    assert!(all.sift::<[Marker; 3]>().is_empty());
    assert!(all.sift::<Box<[Marker; 1]>>().is_empty());
}

// S9
#[test]
fn different_instantiations_of_a_generic_type_are_distinct() {
    let pair = Pair {
        left: Wrapper {
            items: [Marker(1), Marker(2)],
            extra: None,
        },
        right: Wrapper {
            items: [Marker(3), Marker(4), Marker(5)],
            extra: Some(Marker(6)),
        },
    };
    assert_same_refs(&pair.sift::<Wrapper<Marker, 2>>(), &[&pair.left]);
    assert_same_refs(&pair.sift::<Wrapper<Marker, 3>>(), &[&pair.right]);

    let trees = (
        Tree {
            value: 1u32,
            children: Vec::new(),
        },
        Tree {
            value: 2u64,
            children: Vec::new(),
        },
    );
    assert_same_refs(&trees.sift::<Tree<u32>>(), &[&trees.0]);
    assert_same_refs(&trees.sift::<Tree<u64>>(), &[&trees.1]);
}

// S10
#[test]
fn type_aliases_match_the_aliased_type() {
    type Members = Vec<UserId>;

    let org = sample_org();
    let core = &org.teams[0];
    let compiler = &core.sub_teams[0];
    let backend = &compiler.sub_teams[0];
    assert_same_refs(
        &org.sift::<Members>(),
        &[&core.members, &compiler.members, &backend.members],
    );
}

// S12
#[test]
fn shared_and_equal_values_are_not_deduplicated() {
    let root = shared_folders();
    let folders = root.sift::<Folder>();
    let names: Vec<&str> = folders.iter().map(|folder| folder.name.as_str()).collect();
    assert_eq!(names, ["root", "shared", "docs", "shared"]);
    assert!(std::ptr::eq(folders[1], folders[3]));

    let sizes: Vec<u64> = root.sift::<u64>().into_iter().copied().collect();
    assert_eq!(sizes, [10, 10, 20]);

    let equal = [Marker(7), Marker(7)];
    assert_same_refs(&equal.sift::<Marker>(), &[&equal[0], &equal[1]]);
}

// S13
#[test]
fn zero_sized_values_are_found_once_per_occurrence() {
    let values = ((), Ok::<(), Marker>(()), UnitStruct, [(); 2]);
    assert_eq!(values.sift::<()>().len(), 4);
    assert_eq!(values.sift::<UnitStruct>(), [&UnitStruct]);
}

// S14
#[test]
fn floats_are_matched_by_type_not_value() {
    let floats = [f64::NAN, -0.0, 0.0];
    let found = floats.sift::<f64>();
    assert_eq!(found.len(), 3);
    assert!(found[0].is_nan());
    assert!(found[1].is_sign_negative());
    assert!(found[2].is_sign_positive());
}

// S15
#[test]
fn leaves_do_not_expose_what_is_inside_them() {
    let leaves = leaves();
    // `String`, `str`, `Box<str>`, `Rc<str>`, `Arc<str>`, `Cow<str>` and `PathBuf` hold bytes and
    // chars, `Duration` holds a `u64`, `NonZero<u32>` holds a `u32`.
    assert_same_refs(&leaves.sift::<u8>(), &[&leaves.u8]);
    assert_same_refs(&leaves.sift::<char>(), &[&leaves.char]);
    assert_same_refs(&leaves.sift::<u32>(), &[&leaves.u32]);
    assert_same_refs(&leaves.sift::<u64>(), &[&leaves.u64]);
    assert_same_refs(&leaves.sift::<String>(), &[&leaves.string]);
    assert!(leaves.sift::<std::ffi::OsString>().is_empty());
}

// S16
#[test]
fn any_supported_type_can_be_the_root() {
    assert_eq!(
        Vec::from([Marker(1), Marker(2)]).sift::<Marker>(),
        [&Marker(1), &Marker(2)]
    );
    assert_eq!((Marker(1), "a".to_string()).sift::<String>(), ["a"]);
    assert_eq!(Some(Marker(1)).sift::<Option<Marker>>().len(), 1);
    assert_eq!([Marker(1); 3].sift::<Marker>().len(), 3);
    assert_eq!(Box::new(Marker(1)).sift::<Box<Marker>>().len(), 1);
    assert_eq!(Marker(5).sift::<u32>(), [&5]);
    assert_eq!(7u8.sift::<u8>(), [&7]);
    assert!("abc".sift::<char>().is_empty());
}

// S17
#[test]
fn phantom_data_never_yields_its_parameter() {
    let typed = Typed::<Marker> {
        raw: 7,
        marker: PhantomData,
    };
    assert!(typed.sift::<Marker>().is_empty());
    assert_same_refs(&typed.sift::<PhantomData<Marker>>(), &[&typed.marker]);
    assert_eq!(typed.sift::<u64>(), [&7]);
}

// S18
#[test]
fn cow_is_searched_through_deref() {
    let all = all_containers();
    assert_eq!(all.cow_borrowed.sift::<Marker>(), [&Marker(27)]);
    assert_eq!(all.cow_owned.sift::<Marker>(), [&Marker(28)]);

    let owned: Cow<'static, str> = Cow::Owned("text".to_string());
    assert!(owned.sift::<String>().is_empty());
    assert_same_refs(&owned.sift::<Cow<'static, str>>(), &[&owned]);
}
