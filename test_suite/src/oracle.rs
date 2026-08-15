//! Results compared against independent reference traversals.

use typesift::TypeSift;

use crate::fixtures::ids::NodeId;
use crate::fixtures::json::{
    Json, random_json, reference_ints, reference_nodes, reference_strings, sample_json,
};
use crate::fixtures::trees::{Tree, numbered_tree, tree_size};
use crate::helpers::{assert_same_refs, check_consistency};

#[test]
fn sample_json_matches_reference_traversal() {
    let json = sample_json();
    assert_eq!(
        json.sift::<String>(),
        ["name", "typesift", "nested", "deep", "tags", "rust"]
    );
    assert_eq!(json.sift::<i64>(), [&2, &1]);
    assert_same_refs(&json.sift::<String>(), &reference_strings(&json));
    assert_same_refs(&json.sift::<Json>(), &reference_nodes(&json));
}

// O1, O3
#[test]
fn random_json_matches_reference_traversal() {
    for seed in 1..=300 {
        let json = random_json(seed, 5, 4);
        let nodes = reference_nodes(&json);
        assert_same_refs(&json.sift::<Json>(), &nodes);
        assert_same_refs(&json.sift::<String>(), &reference_strings(&json));
        assert_same_refs(&json.sift::<i64>(), &reference_ints(&json));

        if nodes.len() <= 100 {
            check_consistency::<_, Json>(&json);
            check_consistency::<_, String>(&json);
        }
    }
}

// O2
#[test]
fn numbered_trees_yield_ids_in_pre_order() {
    for depth in 0..=5 {
        for fanout in 1..=4 {
            let tree = numbered_tree(depth, fanout);
            let size = tree_size(depth, fanout);

            let ids: Vec<u32> = tree.sift::<NodeId>().iter().map(|id| id.0).collect();
            assert_eq!(
                ids,
                (1..=size).collect::<Vec<_>>(),
                "depth {depth}, fanout {fanout}"
            );

            for (position, subtree) in (1..).zip(tree.sift::<Tree<NodeId>>()) {
                assert_eq!(subtree.value, NodeId(position));
            }
        }
    }
}
