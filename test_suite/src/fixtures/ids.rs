use typesift::TypeSift;

macro_rules! id_types {
    ($($name:ident),* $(,)?) => {$(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, TypeSift)]
        pub struct $name(pub u32);
    )*};
}

id_types!(UserId, TeamId, TaskId, NodeId, Marker);

/// Not contained in any fixture.
#[derive(Debug, PartialEq, TypeSift)]
pub struct Absent;
