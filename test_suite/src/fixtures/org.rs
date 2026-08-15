//! A realistic domain model with recursion three levels deep in both teams and tasks.

use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use typesift::TypeSift;

use super::ids::{TaskId, TeamId, UserId};

#[derive(Debug, PartialEq, TypeSift)]
pub struct Org {
    pub name: String,
    pub users: BTreeMap<UserId, User>,
    pub teams: Vec<Team>,
    pub tasks: Vec<Task>,
}

#[derive(Debug, PartialEq, TypeSift)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: Option<String>,
    pub manager: Option<UserId>,
    pub roles: BTreeSet<Role>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, TypeSift)]
pub enum Role {
    Admin,
    Member,
    Guest,
}

#[derive(Debug, PartialEq, TypeSift)]
pub struct Team {
    pub id: TeamId,
    pub name: String,
    pub lead: UserId,
    pub members: Vec<UserId>,
    pub sub_teams: Vec<Team>,
}

#[derive(Debug, PartialEq, TypeSift)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub assignee: Assignee,
    pub status: Status,
    pub blocked_by: Vec<TaskId>,
    pub subtasks: Vec<Task>,
    pub estimate: Option<Duration>,
    pub labels: Vec<&'static str>,
}

#[derive(Debug, PartialEq, TypeSift)]
pub enum Assignee {
    Unassigned,
    User(UserId),
    Team {
        team: TeamId,
        reviewer: Option<UserId>,
    },
}

#[derive(Debug, PartialEq, TypeSift)]
pub enum Status {
    Todo,
    InProgress { since: Duration, by: UserId },
    Done(Result<Duration, String>),
}

/// Users 1 Ada, 2 Grace, 3 Linus. Teams 10 Core > 11 Compiler > 12 Backend. Tasks 100 Parser >
/// 101 Lexer > 103 Unicode, then 102 Docs.
///
/// Expected pre-order results:
///
/// | Query | Result |
/// |---|---|
/// | `UserId` | `1,1, 2,2,1, 3,3,1, 1,1,2, 2,2,3, 3,3, 3,3, 2, 1` |
/// | `TeamId` | `10, 11, 12, 11` |
/// | `TaskId` | `100, 101, 103, 102, 100, 101` |
/// | `Task` | `100, 101, 103, 102` |
/// | `Team` | `10, 11, 12` |
/// | `Duration` (secs) | `90, 30, 120, 3600, 600` |
/// | `Role` | `Admin, Member, Member, Guest` |
/// | `Option<UserId>` | `None, Some(1), Some(1), Some(2)` |
/// | `Vec<Task>` | `org.tasks`, then the `subtasks` of 100, 101, 103, 102 |
/// | `Vec<Team>` | `org.teams`, then the `sub_teams` of 10, 11, 12 |
/// | `&'static str` | `"parser", "p1"` |
/// | `Assignee` | `User(3)`, `Team { 11, Some(2) }`, `User(1)`, `Unassigned` |
pub fn sample_org() -> Org {
    let users = BTreeMap::from([
        (
            UserId(1),
            User {
                id: UserId(1),
                name: "Ada".to_string(),
                email: Some("ada@acme.test".to_string()),
                manager: None,
                roles: BTreeSet::from([Role::Admin, Role::Member]),
            },
        ),
        (
            UserId(2),
            User {
                id: UserId(2),
                name: "Grace".to_string(),
                email: Some("grace@acme.test".to_string()),
                manager: Some(UserId(1)),
                roles: BTreeSet::from([Role::Member]),
            },
        ),
        (
            UserId(3),
            User {
                id: UserId(3),
                name: "Linus".to_string(),
                email: None,
                manager: Some(UserId(1)),
                roles: BTreeSet::from([Role::Guest]),
            },
        ),
    ]);

    let backend = Team {
        id: TeamId(12),
        name: "Backend".to_string(),
        lead: UserId(3),
        members: vec![UserId(3)],
        sub_teams: Vec::new(),
    };
    let compiler = Team {
        id: TeamId(11),
        name: "Compiler".to_string(),
        lead: UserId(2),
        members: vec![UserId(2), UserId(3)],
        sub_teams: vec![backend],
    };
    let core = Team {
        id: TeamId(10),
        name: "Core".to_string(),
        lead: UserId(1),
        members: vec![UserId(1), UserId(2)],
        sub_teams: vec![compiler],
    };

    let unicode = Task {
        id: TaskId(103),
        title: "Unicode".to_string(),
        assignee: Assignee::User(UserId(1)),
        status: Status::Done(Err("wontfix".to_string())),
        blocked_by: Vec::new(),
        subtasks: Vec::new(),
        estimate: Some(Duration::from_secs(120)),
        labels: Vec::new(),
    };
    let lexer = Task {
        id: TaskId(101),
        title: "Lexer".to_string(),
        assignee: Assignee::Team {
            team: TeamId(11),
            reviewer: Some(UserId(2)),
        },
        status: Status::Done(Ok(Duration::from_secs(30))),
        blocked_by: Vec::new(),
        subtasks: vec![unicode],
        estimate: None,
        labels: Vec::new(),
    };
    let parser = Task {
        id: TaskId(100),
        title: "Parser".to_string(),
        assignee: Assignee::User(UserId(3)),
        status: Status::InProgress {
            since: Duration::from_secs(90),
            by: UserId(3),
        },
        blocked_by: Vec::new(),
        subtasks: vec![lexer],
        estimate: Some(Duration::from_secs(3600)),
        labels: vec!["parser", "p1"],
    };
    let docs = Task {
        id: TaskId(102),
        title: "Docs".to_string(),
        assignee: Assignee::Unassigned,
        status: Status::Todo,
        blocked_by: vec![TaskId(100), TaskId(101)],
        subtasks: Vec::new(),
        estimate: Some(Duration::from_secs(600)),
        labels: Vec::new(),
    };

    Org {
        name: "Acme".to_string(),
        users,
        teams: vec![core],
        tasks: vec![parser, docs],
    }
}
