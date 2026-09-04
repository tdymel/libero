//! Reading the option tree the two ways a cascader needs it: as columns walked
//! by an index path, and as a flat list of full root-to-option paths.
//!
//! Every function here is pure and takes `&[CascaderNode]` - the tree without
//! its values - which is what keeps it out of the per-`T` code and makes the
//! cursor arithmetic testable without rendering anything.

use super::option::CascaderNode;

/// One full root-to-node path. `Paths` draws one row per entry, and search
/// matches against the joined [`labels`](Self::labels).
#[derive(Clone, Debug, PartialEq)]
pub(super) struct FlatPath {
    /// One index per level - the same shape as the cursor.
    pub indices: Vec<usize>,
    pub labels: Vec<String>,
    /// The node's own `disabled`, or'd with every ancestor's. Mantine's
    /// `flattenCascaderPaths` rule: a node under a disabled ancestor is
    /// disabled.
    pub disabled: bool,
}

/// The nodes one level below `indices` - the roots for an empty path.
pub(super) fn children_at<'a>(nodes: &'a [CascaderNode], indices: &[usize]) -> &'a [CascaderNode] {
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
    nodes: &'a [CascaderNode],
    indices: &[usize],
) -> Option<&'a CascaderNode> {
    let (last, parents) = indices.split_last()?;
    children_at(nodes, parents).get(*last)
}

/// Whether the node at `indices` is unreachable - its own `disabled`, or any
/// ancestor's. A path that does not resolve counts as disabled, so a stale
/// cursor can never be committed.
pub(super) fn disabled_at(nodes: &[CascaderNode], indices: &[usize]) -> bool {
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

/// Every path, depth first. With `any_level` that is one entry per node;
/// without it, one per leaf - which is the same rule the keyboard commits by,
/// so `Paths` never offers a row Enter would refuse.
pub(super) fn flatten_paths(nodes: &[CascaderNode], any_level: bool) -> Vec<FlatPath> {
    let mut out = Vec::new();
    let mut prefix = FlatPath {
        indices: Vec::new(),
        labels: Vec::new(),
        disabled: false,
    };
    push_paths(nodes, any_level, &mut prefix, &mut out);
    out
}

fn push_paths(
    nodes: &[CascaderNode],
    any_level: bool,
    prefix: &mut FlatPath,
    out: &mut Vec<FlatPath>,
) {
    for (index, node) in nodes.iter().enumerate() {
        let inherited = prefix.disabled;
        prefix.indices.push(index);
        prefix.labels.push(node.label.clone());
        prefix.disabled = inherited || node.disabled;

        if any_level || !node.has_children() {
            out.push(prefix.clone());
        }
        push_paths(&node.children, any_level, prefix, out);

        prefix.disabled = inherited;
        prefix.indices.pop();
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

pub(super) fn first_enabled(column: &[CascaderNode]) -> Option<usize> {
    step(column.len(), |index| column[index].disabled, None, true)
}

pub(super) fn last_enabled(column: &[CascaderNode]) -> Option<usize> {
    step(column.len(), |index| column[index].disabled, None, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(label: &str, children: Vec<CascaderNode>) -> CascaderNode {
        CascaderNode {
            label: label.to_string(),
            disabled: false,
            children,
        }
    }

    fn off(mut node: CascaderNode) -> CascaderNode {
        node.disabled = true;
        node
    }

    fn tree() -> Vec<CascaderNode> {
        vec![
            node(
                "Food",
                vec![
                    node(
                        "Fruit",
                        vec![node("Apple", vec![]), off(node("Pear", vec![]))],
                    ),
                    node("Veg", vec![node("Leek", vec![])]),
                ],
            ),
            off(node("Drink", vec![node("Tea", vec![])])),
            node("Other", vec![]),
        ]
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
        assert_eq!(node_at(&tree, &[0, 0, 1]).unwrap().label, "Pear");
        assert_eq!(node_at(&tree, &[1]).unwrap().label, "Drink");
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
        assert_eq!(paths[0].labels, vec!["Food"]);
        assert_eq!(paths[1].labels, vec!["Food", "Fruit"]);
        assert_eq!(paths[2].indices, vec![0, 0, 0]);
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
        let all_disabled = vec![off(node("A", vec![]))];
        assert_eq!(first_enabled(&all_disabled), None);
        assert_eq!(last_enabled(&all_disabled), None);
        assert_eq!(step(0, |_| false, None, true), None);
    }
}
