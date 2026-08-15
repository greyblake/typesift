use std::collections::HashMap;

use typesift::TypeSift;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypeSift)]
struct UserId(i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypeSift)]
struct GroupId(i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypeSift)]
struct TaskId(i32);

#[derive(Clone, Debug, TypeSift)]
struct User {
    id: UserId,
    name: String,
    friend_ids: Vec<UserId>,
}

#[derive(Debug, TypeSift)]
enum Assignment {
    User(UserId),
    Group(GroupId),
}

#[derive(Debug, TypeSift)]
struct Task {
    id: TaskId,
    name: String,
}

#[derive(Debug, TypeSift)]
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

    assert_eq!(UserId(32).sift::<UserId>(), [&UserId(32)]);
    assert!(true.sift::<UserId>().is_empty());
    assert_eq!(user.sift::<UserId>(), [&UserId(1), &UserId(4), &UserId(43)]);
    assert_eq!(
        users.sift::<UserId>(),
        [&UserId(1), &UserId(4), &UserId(43), &UserId(2), &UserId(3)]
    );
    assert_eq!(id123.sift::<UserId>(), [&UserId(1), &UserId(2), &UserId(3)]);
    assert_eq!(user.sift::<String>(), ["Alice"]);
    assert_eq!(graph.sift::<GroupId>(), [&GroupId(1)]);

    // `HashMap` iteration order is unspecified, so sort before comparing.
    let mut user_ids: Vec<i32> = graph.sift::<UserId>().into_iter().map(|id| id.0).collect();
    user_ids.sort_unstable();
    assert_eq!(user_ids, [1, 2, 3, 4, 5, 43]);

    println!("task ids: {:?}", graph.sift::<TaskId>());
}
