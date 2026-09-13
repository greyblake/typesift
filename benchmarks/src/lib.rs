//! A DTO graph and two ways to collect the `UserId`s in it.
//!
//! - [`Company::user_ids`] returns `impl Iterator`, which is what most people reach for. The
//!   caller pulls values out with `next`, through five levels of `Chain` and `FlatMap`.
//! - `sift` from `typesift`.
//!
//! Both must produce the same references in the same order, which `manual_and_typesift_agree`
//! checks. Without that, the comparison means nothing.
//!
//! The graph nests five levels deep: [`Company`] > [`Department`] > [`Team`] > [`Project`] >
//! [`Task`], with ids reached through struct fields, `Vec`s, an `Option` and an enum variant.

use typesift::TypeSift;

const DEPARTMENTS: usize = 4;
const TEAMS_PER_DEPARTMENT: usize = 4;
const PROJECTS_PER_TEAM: usize = 4;
const TASKS_PER_PROJECT: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TypeSift)]
pub struct UserId(pub u64);

// Every `user_ids` below chains its fields in declaration order, because that is the order
// `typesift` visits them in. Deviating would make the two traversals return different sequences.

#[derive(TypeSift)]
pub struct Company {
    pub name: String,
    pub owner: UserId,
    pub departments: Vec<Department>,
}

impl Company {
    pub fn user_ids(&self) -> impl Iterator<Item = &UserId> {
        std::iter::once(&self.owner).chain(
            self.departments
                .iter()
                .flat_map(|department| department.user_ids()),
        )
    }
}

#[derive(TypeSift)]
pub struct Department {
    pub name: String,
    pub head: UserId,
    pub teams: Vec<Team>,
}

impl Department {
    pub fn user_ids(&self) -> impl Iterator<Item = &UserId> {
        std::iter::once(&self.head).chain(self.teams.iter().flat_map(|team| team.user_ids()))
    }
}

#[derive(TypeSift)]
pub struct Team {
    pub name: String,
    pub lead: UserId,
    pub members: Vec<UserId>,
    pub projects: Vec<Project>,
}

impl Team {
    pub fn user_ids(&self) -> impl Iterator<Item = &UserId> {
        std::iter::once(&self.lead)
            .chain(self.members.iter())
            .chain(self.projects.iter().flat_map(|project| project.user_ids()))
    }
}

#[derive(TypeSift)]
pub struct Project {
    pub name: String,
    pub owner: UserId,
    pub reviewers: Vec<UserId>,
    pub status: Status,
    pub tasks: Vec<Task>,
}

impl Project {
    pub fn user_ids(&self) -> impl Iterator<Item = &UserId> {
        std::iter::once(&self.owner)
            .chain(self.reviewers.iter())
            .chain(self.status.user_ids())
            .chain(self.tasks.iter().flat_map(|task| task.user_ids()))
    }
}

#[derive(TypeSift)]
pub enum Status {
    Active,
    Blocked { by: UserId },
}

impl Status {
    pub fn user_ids(&self) -> impl Iterator<Item = &UserId> {
        match self {
            Status::Active => None,
            Status::Blocked { by } => Some(by),
        }
        .into_iter()
    }
}

#[derive(TypeSift)]
pub struct Task {
    pub title: String,
    pub assignee: Option<UserId>,
    pub watchers: Vec<UserId>,
}

impl Task {
    pub fn user_ids(&self) -> impl Iterator<Item = &UserId> {
        self.assignee.iter().chain(self.watchers.iter())
    }
}

/// The graph both traversals run over. Deterministic, so runs are comparable.
pub fn sample_company() -> Company {
    let mut next_id = 0;
    let mut id = || {
        next_id += 1;
        UserId(next_id)
    };

    let departments = (0..DEPARTMENTS)
        .map(|department| {
            let teams = (0..TEAMS_PER_DEPARTMENT)
                .map(|team| {
                    let projects = (0..PROJECTS_PER_TEAM)
                        .map(|project| {
                            let tasks = (0..TASKS_PER_PROJECT)
                                .map(|task| Task {
                                    title: format!("task {task}"),
                                    assignee: Some(id()),
                                    watchers: vec![id(), id()],
                                })
                                .collect();
                            Project {
                                name: format!("project {project}"),
                                owner: id(),
                                reviewers: vec![id(), id(), id()],
                                // Half the projects carry an id inside an enum variant.
                                status: if project % 2 == 0 {
                                    Status::Active
                                } else {
                                    Status::Blocked { by: id() }
                                },
                                tasks,
                            }
                        })
                        .collect();
                    Team {
                        name: format!("team {team}"),
                        lead: id(),
                        members: vec![id(), id(), id(), id()],
                        projects,
                    }
                })
                .collect();
            Department {
                name: format!("department {department}"),
                head: id(),
                teams,
            }
        })
        .collect();

    Company {
        name: "Acme".to_string(),
        owner: id(),
        departments,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The benchmark is only meaningful if both traversals do the same work.
    #[test]
    fn manual_and_typesift_agree() {
        let company = sample_company();
        let manual: Vec<&UserId> = company.user_ids().collect();
        let sifted = company.sift::<UserId>();
        assert_eq!(manual, sifted);
    }

    #[test]
    fn graph_is_big_enough_to_measure() {
        let company = sample_company();
        assert!(company.user_ids().count() > 3000);
    }
}
