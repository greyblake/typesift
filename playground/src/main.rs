use std::collections::HashMap;

use type_iter::{TypeIter, TypeValues};

#[derive(Debug, Clone, Copy, PartialEq, TypeIter, Eq, Hash)]
#[type_iter(String)]
struct UserId(i32);

#[derive(Debug, Clone, Copy, PartialEq, TypeIter, Eq)]
#[type_iter(String)]
struct GroupId(i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypeIter)]
#[type_iter(String)]
struct TaskId(i32);

#[derive(Clone, Debug, TypeIter)]
#[type_iter(UserId, String)]
struct User {
    id: UserId,
    name: String,
    // #[ignore]
    friend_ids: Vec<UserId>,
}

// #[derive(Debug, TypeIter)]
// #[type_iter(UserId)]
// enum Assignment {
//     User(UserId),
//     Group(GroupId),
// }
//
// #[derive(Debug, TypeIter)]
// struct Task {
//     id: TaskId,
//     name: String,
// }
//
// #[derive(Debug, TypeIter)]
// #[type_iter(UserId)]
// struct Graph {
//     users: Vec<User>,
//     assignments: HashMap<TaskId, Assignment>,
// }

fn main() {
    let users = vec![
        User {
            id: UserId(1),
            name: "Alice".to_string(),
            friend_ids: vec![UserId(4), UserId(43)],
        },
        User {
            id: UserId(2),
            name: "Bob".to_string(),
            friend_ids: vec![],
        },
        User {
            id: UserId(3),
            name: "Carol".to_string(),
            friend_ids: vec![],
        },
    ];

    let mut map = HashMap::new();
    map.insert(UserId(1), users.clone());

    /*
    let task1 = Task {
        id: TaskId(1),
        name: "Task 1".to_string(),
    };
    let task2 = Task {
        id: TaskId(2),
        name: "Task 2".to_string(),
    };

    let assignments = vec![
        (TaskId(1), Assignment::User(UserId(5))),
        (TaskId(2), Assignment::Group(GroupId(1))),
    ];

    let graph = Graph {
        users: users.clone(),
        assignments: assignments.into_iter().collect(),
    };
    */

    let id1 = UserId(1);
    let id2 = UserId(2);
    let id3 = UserId(3);

    let id123 = [id1, id2, id3];

    let user = users[0].clone();

    assert_eq!(
        UserId(32).type_values::<UserId>().collect::<Vec<_>>(),
        vec![&UserId(32)]
    );

    assert_eq!(
        true.type_values::<UserId>().collect::<Vec<_>>(),
        vec![] as Vec<&UserId>
    );

    assert_eq!(
        user.type_values::<UserId>().collect::<Vec<_>>(),
        vec![&UserId(1), &UserId(4), &UserId(43)]
    );

    assert_eq!(
        users.type_values::<UserId>().collect::<Vec<_>>(),
        vec![&UserId(1), &UserId(4), &UserId(43), &UserId(2), &UserId(3)]
    );

    assert_eq!(
        id123.type_values::<UserId>().collect::<Vec<_>>(),
        vec![&UserId(1), &UserId(2), &UserId(3)]
    );

    assert_eq!(
        user.type_values::<String>().collect::<Vec<_>>(),
        vec![&"Alice".to_string()]
    );

    // graph.type_values::<UserId>().collect::<Vec<_>>();
    let ids = map.type_values::<UserId>().collect::<Vec<_>>();
    println!("{:?}", ids);
}
