//! The `uuid` feature: `Uuid` is a leaf.

use std::collections::HashMap;

use typesift::TypeSift;
use uuid::Uuid;

use crate::helpers::{assert_same_refs, check_consistency};

#[derive(Debug, TypeSift)]
struct Account {
    id: Uuid,
    owner: Option<Uuid>,
    sessions: Vec<Uuid>,
    label: String,
}

fn account() -> Account {
    Account {
        id: Uuid::from_u128(1),
        owner: Some(Uuid::from_u128(2)),
        sessions: vec![Uuid::from_u128(3), Uuid::from_u128(4)],
        label: "main".to_string(),
    }
}

#[test]
fn uuid_is_found_wherever_it_is_nested() {
    let account = account();
    assert_same_refs(
        &account.sift::<Uuid>(),
        &[
            &account.id,
            account.owner.as_ref().expect("the owner is set"),
            &account.sessions[0],
            &account.sessions[1],
        ],
    );
    assert_eq!(account.sift::<String>(), ["main"]);
}

#[test]
fn uuid_is_a_leaf() {
    let account = account();
    // A `Uuid` is offered as itself; what it holds is never searched.
    assert!(account.sift::<[u8; 16]>().is_empty());
    assert!(account.sift::<u128>().is_empty());
    assert!(account.sift::<u8>().is_empty());
}

#[test]
fn uuid_inside_std_containers() {
    let map: HashMap<Uuid, Vec<Uuid>> =
        HashMap::from([(Uuid::from_u128(1), vec![Uuid::from_u128(2)])]);
    assert_eq!(map.sift::<Uuid>().len(), 2);
    check_consistency::<_, Uuid>(&map);
}

#[test]
fn entry_points_agree_for_uuid() {
    check_consistency::<_, Uuid>(&account());
    check_consistency::<_, String>(&account());
    check_consistency::<_, Account>(&account());
}
