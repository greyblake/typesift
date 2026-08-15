//! Recursive structures: a generic tree, a linked list and folders that share a subtree.

use std::rc::Rc;

use typesift::TypeSift;

use super::ids::NodeId;

#[derive(Debug, Clone, PartialEq, TypeSift)]
pub struct Tree<T> {
    pub value: T,
    pub children: Vec<Tree<T>>,
}

/// A complete tree whose nodes are numbered `1..=tree_size(depth, fanout)` in pre-order.
pub fn numbered_tree(depth: u32, fanout: u32) -> Tree<NodeId> {
    fn build(depth: u32, fanout: u32, next: &mut u32) -> Tree<NodeId> {
        *next += 1;
        let value = NodeId(*next);
        let children = if depth == 0 {
            Vec::new()
        } else {
            (0..fanout)
                .map(|_| build(depth - 1, fanout, next))
                .collect()
        };
        Tree { value, children }
    }

    build(depth, fanout, &mut 0)
}

/// The number of nodes in `numbered_tree(depth, fanout)`.
pub fn tree_size(depth: u32, fanout: u32) -> u32 {
    let mut level = 1;
    let mut size = 1;
    for _ in 0..depth {
        level *= fanout;
        size += level;
    }
    size
}

#[derive(Debug, TypeSift)]
pub struct List {
    pub value: u32,
    pub next: Option<Box<List>>,
}

impl Drop for List {
    // The generated drop glue recurses once per node; unlink iteratively instead.
    fn drop(&mut self) {
        let mut next = self.next.take();
        while let Some(mut node) = next {
            next = node.next.take();
        }
    }
}

/// A list holding `1..=len`, in order.
pub fn list(len: u32) -> List {
    assert!(len > 0, "a list has at least one node");
    let mut head = List {
        value: len,
        next: None,
    };
    for value in (1..len).rev() {
        head = List {
            value,
            next: Some(Box::new(head)),
        };
    }
    head
}

#[derive(Debug, TypeSift)]
pub struct Folder {
    pub name: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug, TypeSift)]
pub enum Entry {
    File { name: String, size: u64 },
    Folder(Box<Folder>),
    Shared(Rc<Folder>),
}

/// `root` holds `shared`, then `docs` (which holds `shared` again), then the file `notes`.
///
/// `sift::<Folder>()` gives `root, shared, docs, shared`, where both `shared` are one allocation.
/// `sift::<u64>()` gives `10, 10, 20`.
pub fn shared_folders() -> Folder {
    let shared = Rc::new(Folder {
        name: "shared".to_string(),
        entries: vec![Entry::File {
            name: "readme".to_string(),
            size: 10,
        }],
    });
    Folder {
        name: "root".to_string(),
        entries: vec![
            Entry::Shared(Rc::clone(&shared)),
            Entry::Folder(Box::new(Folder {
                name: "docs".to_string(),
                entries: vec![Entry::Shared(shared)],
            })),
            Entry::File {
                name: "notes".to_string(),
                size: 20,
            },
        ],
    }
}
