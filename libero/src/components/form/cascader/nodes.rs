//! Reading an erased tree the two ways a cascader needs it: as columns walked
//! by an index path, and as a flat list of full root-to-node paths.
//!
//! Every function here is pure and takes `&[TreeNodeErased]`, which is what
//! makes the cursor arithmetic testable without rendering anything.

use crate::components::navigation::TreeNodeErased;

/// One full root-to-node path. `Paths` draws one row per entry, and search
/// matches against the joined [`labels`](Self::labels).
#[derive(Clone, Debug, PartialEq)]
pub(super) struct FlatPath {
    /// One index per level - the same shape as the cursor.
    pub indices: Vec<usize>,
    pub ids: Vec<String>,
    pub labels: Vec<String>,
    /// The node's own `disabled`, or'd with every ancestor's. Mantine's
    /// `flattenCascaderPaths` rule: a node under a disabled ancestor is
    /// disabled.
    pub disabled: bool,
}

/// The nodes one level below `indices` - the roots for an empty path.
pub(super) fn children_at<'a>(
    nodes: &'a [TreeNodeErased],
    indices: &[usize],
) -> &'a [TreeNodeErased] {
    let mut level = nodes;
    for index in indices {
        match level.get(*index) {
            Some(node) => level = &node.children,
            None => return &[],
        }
    }
    level
}

pub(super) fn node_at<'a>(
    nodes: &'a [TreeNodeErased],
    indices: &[usize],
) -> Option<&'a TreeNodeErased> {
    let (last, parents) = indices.split_last()?;
    children_at(nodes, parents).get(*last)
}

/// Whether the node at `indices` is unreachable - its own `disabled`, or any
/// ancestor's. A path that does not resolve counts as disabled, so a stale
/// cursor can never be committed.
pub(super) fn disabled_at(nodes: &[TreeNodeErased], indices: &[usize]) -> bool {
    let mut level = nodes;
    for index in indices {
        let Some(node) = level.get(*index) else {
            return true;
        };
        if node.disabled {
            return true;
        }
        level = &node.children;
    }
    false
}

pub(super) fn ids_at(nodes: &[TreeNodeErased], indices: &[usize]) -> Vec<String> {
    let mut ids = Vec::with_capacity(indices.len());
    let mut level = nodes;
    for index in indices {
        let Some(node) = level.get(*index) else {
            break;
        };
        ids.push(node.id.clone());
        level = &node.children;
    }
    ids
}

/// The cursor for a path of ids. `None` when any id is not where the path says
/// it is, which is what makes a `value` that no longer matches `data` visible
/// rather than silently half-applied.
pub(super) fn indices_for_ids(nodes: &[TreeNodeErased], ids: &[String]) -> Option<Vec<usize>> {
    if ids.is_empty() {
        return None;
    }
    let mut indices = Vec::with_capacity(ids.len());
    let mut level = nodes;
    for id in ids {
        let index = level.iter().position(|node| &node.id == id)?;
        indices.push(index);
        level = &level[index].children;
    }
    Some(indices)
}

/// Every path, depth first. With `any_level` that is one entry per node;
/// without it, one per leaf - which is the same rule the keyboard commits by,
/// so `Paths` never offers a row Enter would refuse.
pub(super) fn flatten_paths(nodes: &[TreeNodeErased], any_level: bool) -> Vec<FlatPath> {
    let mut out = Vec::new();
    let mut prefix = FlatPath {
        indices: Vec::new(),
        ids: Vec::new(),
        labels: Vec::new(),
        disabled: false,
    };
    push_paths(nodes, any_level, &mut prefix, &mut out);
    out
}

fn push_paths(
    nodes: &[TreeNodeErased],
    any_level: bool,
    prefix: &mut FlatPath,
    out: &mut Vec<FlatPath>,
) {
    for (index, node) in nodes.iter().enumerate() {
        let inherited = prefix.disabled;
        prefix.indices.push(index);
        prefix.ids.push(node.id.clone());
        prefix.labels.push(node.label.clone());
        prefix.disabled = inherited || node.disabled;

        if any_level || !node.has_children() {
            out.push(prefix.clone());
        }
        push_paths(&node.children, any_level, prefix, out);

        prefix.disabled = inherited;
        prefix.indices.pop();
        prefix.ids.pop();
        prefix.labels.pop();
    }
}

pub(super) fn join_labels(labels: &[String], separator: &str) -> String {
    labels.join(separator)
}

/// The next enabled row in one direction, or `None` when there is none - which
/// leaves the cursor where it is rather than wrapping. Clamping, not wrapping,
/// is what `ComboboxCore`'s arrows already do.
///
/// From no cursor at all, forward lands on the first enabled row and backward
/// on the last, so one press arms the list instead of moving inside it.
pub(super) fn step(
    len: usize,
    disabled: impl Fn(usize) -> bool,
    from: Option<usize>,
    forward: bool,
) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let range: Vec<usize> = match (from, forward) {
        (Some(index), true) => (index + 1..len).collect(),
        (Some(index), false) => (0..index).rev().collect(),
        (None, true) => (0..len).collect(),
        (None, false) => (0..len).rev().collect(),
    };
    range.into_iter().find(|index| !disabled(*index))
}

pub(super) fn first_enabled(column: &[TreeNodeErased]) -> Option<usize> {
    step(column.len(), |index| column[index].disabled, None, true)
}

pub(super) fn last_enabled(column: &[TreeNodeErased]) -> Option<usize> {
    step(column.len(), |index| column[index].disabled, None, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::TreeNode;
    use crate::components::navigation::{TreeNodeErased, erase_nodes};

    fn tree() -> Vec<TreeNodeErased> {
        erase_nodes(&[
            TreeNode::new("food", "Food").children(vec![
                TreeNode::new("fruit", "Fruit").children(vec![
                    TreeNode::new("apple", "Apple"),
                    TreeNode::new("pear", "Pear").disabled(true),
                ]),
                TreeNode::new("veg", "Veg").children(vec![TreeNode::new("leek", "Leek")]),
            ]),
            TreeNode::new("drink", "Drink")
                .disabled(true)
                .children(vec![TreeNode::new("tea", "Tea")]),
            TreeNode::new("other", "Other"),
        ])
    }

    #[test]
    fn children_at_walks_the_index_path() {
        let tree = tree();
        assert_eq!(children_at(&tree, &[]).len(), 3);
        assert_eq!(children_at(&tree, &[0]).len(), 2);
        assert_eq!(children_at(&tree, &[0, 0]).len(), 2);
        // A leaf has no column under it, and neither does a path off the tree.
        assert!(children_at(&tree, &[2]).is_empty());
        assert!(children_at(&tree, &[9, 9]).is_empty());
    }

    #[test]
    fn node_at_finds_the_node_and_the_root() {
        let tree = tree();
        assert_eq!(node_at(&tree, &[0, 0, 1]).unwrap().id, "pear");
        assert_eq!(node_at(&tree, &[1]).unwrap().id, "drink");
        assert!(node_at(&tree, &[]).is_none());
    }

    #[test]
    fn disabled_is_inherited_from_every_ancestor() {
        let tree = tree();
        assert!(!disabled_at(&tree, &[0, 0, 0]));
        assert!(disabled_at(&tree, &[0, 0, 1]));
        // `tea` is enabled itself and sits under a disabled `drink`.
        assert!(!node_at(&tree, &[1, 0]).unwrap().disabled);
        assert!(disabled_at(&tree, &[1, 0]));
        // A path that no longer resolves is never committable.
        assert!(disabled_at(&tree, &[7]));
    }

    #[test]
    fn leaf_paths_are_the_default_flattening() {
        let paths = flatten_paths(&tree(), false);
        let joined: Vec<String> = paths
            .iter()
            .map(|path| join_labels(&path.labels, " / "))
            .collect();
        assert_eq!(
            joined,
            vec![
                "Food / Fruit / Apple",
                "Food / Fruit / Pear",
                "Food / Veg / Leek",
                "Drink / Tea",
                "Other",
            ]
        );
        // Inherited, not just its own flag.
        assert_eq!(
            paths.iter().map(|path| path.disabled).collect::<Vec<_>>(),
            vec![false, true, false, true, false]
        );
    }

    #[test]
    fn any_level_flattens_every_node() {
        let paths = flatten_paths(&tree(), true);
        assert_eq!(paths.len(), 9);
        assert_eq!(paths[0].ids, vec!["food"]);
        assert_eq!(paths[1].ids, vec!["food", "fruit"]);
        assert_eq!(paths[2].ids, vec!["food", "fruit", "apple"]);
    }

    #[test]
    fn ids_round_trip_through_indices() {
        let tree = tree();
        let ids = vec!["food".to_string(), "veg".to_string(), "leek".to_string()];
        let indices = indices_for_ids(&tree, &ids).unwrap();
        assert_eq!(indices, vec![0, 1, 0]);
        assert_eq!(ids_at(&tree, &indices), ids);
    }

    #[test]
    fn an_id_that_is_not_there_resolves_to_nothing() {
        let tree = tree();
        assert!(indices_for_ids(&tree, &["nope".to_string()]).is_none());
        // Right ids, wrong nesting.
        assert!(indices_for_ids(&tree, &["apple".to_string()]).is_none());
        assert!(indices_for_ids(&tree, &[]).is_none());
    }

    #[test]
    fn stepping_skips_disabled_and_clamps_at_the_ends() {
        let tree = tree();
        let roots = children_at(&tree, &[]);
        let disabled = |index: usize| roots[index].disabled;
        // `drink` is disabled, so forward from `food` is `other`.
        assert_eq!(step(roots.len(), disabled, Some(0), true), Some(2));
        assert_eq!(step(roots.len(), disabled, Some(2), false), Some(0));
        // Past the last enabled row is no move at all, not a wrap.
        assert_eq!(step(roots.len(), disabled, Some(2), true), None);
        assert_eq!(step(roots.len(), disabled, Some(0), false), None);
        assert_eq!(first_enabled(roots), Some(0));
        assert_eq!(last_enabled(roots), Some(2));
    }

    #[test]
    fn a_column_of_nothing_enabled_has_no_cursor() {
        let all_disabled = erase_nodes(&[TreeNode::new("a", "A").disabled(true)]);
        assert_eq!(first_enabled(&all_disabled), None);
        assert_eq!(last_enabled(&all_disabled), None);
        assert_eq!(step(0, |_| false, None, true), None);
    }
}
