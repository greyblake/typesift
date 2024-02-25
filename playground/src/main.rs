use type_iter::{TypeIter, TypeValues};

#[derive(Debug, Clone, Copy, PartialEq, TypeIter)]
#[type_iter(String)]
struct UserId(i32);


#[derive(Clone, TypeIter)]
#[type_iter(UserId, String)]
struct User {
    id: UserId,
    name: String,
    friend_ids: Vec<UserId>,
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
}
