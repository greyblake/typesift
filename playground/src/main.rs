use std::collections::HashMap;

use type_iter::TypeIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypeIter)]
struct UserId(i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypeIter)]
struct GroupId(i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypeIter)]
struct TaskId(i32);

#[derive(Clone, Debug, TypeIter)]
struct User {
    id: UserId,
    name: String,
    friend_ids: Vec<UserId>,
}

#[derive(Debug, TypeIter)]
enum Assignment {
    User(UserId),
    Group(GroupId),
}

#[derive(Debug, TypeIter)]
struct Task {
    id: TaskId,
    name: String,
}

#[derive(Debug, TypeIter)]
struct Graph {
    users: Vec<User>,
    tasks: Vec<Task>,
    assignments: HashMap<TaskId, Assignment>,
}

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

    let tasks = vec![
        Task {
            id: TaskId(1),
            name: "Task 1".to_string(),
        },
        Task {
            id: TaskId(2),
            name: "Task 2".to_string(),
        },
    ];

    let assignments = HashMap::from([
        (TaskId(1), Assignment::User(UserId(5))),
        (TaskId(2), Assignment::Group(GroupId(1))),
    ]);

    let graph = Graph {
        users: users.clone(),
        tasks,
        assignments,
    };

    let user = &users[0];
    let id123 = [UserId(1), UserId(2), UserId(3)];

    assert_eq!(UserId(32).type_values::<UserId>(), [&UserId(32)]);
    assert!(true.type_values::<UserId>().is_empty());
    assert_eq!(
        user.type_values::<UserId>(),
        [&UserId(1), &UserId(4), &UserId(43)]
    );
    assert_eq!(
        users.type_values::<UserId>(),
        [&UserId(1), &UserId(4), &UserId(43), &UserId(2), &UserId(3)]
    );
    assert_eq!(
        id123.type_values::<UserId>(),
        [&UserId(1), &UserId(2), &UserId(3)]
    );
    assert_eq!(user.type_values::<String>(), ["Alice"]);
    assert_eq!(graph.type_values::<GroupId>(), [&GroupId(1)]);

    // `HashMap` iteration order is unspecified, so sort before comparing.
    let mut user_ids: Vec<i32> = graph
        .type_values::<UserId>()
        .into_iter()
        .map(|id| id.0)
        .collect();
    user_ids.sort_unstable();
    assert_eq!(user_ids, [1, 2, 3, 4, 5, 43]);

    println!("task ids: {:?}", graph.type_values::<TaskId>());
}
