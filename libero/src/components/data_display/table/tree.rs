//! Tree data: each row under the row its `parent_key` names.

use std::{collections::HashMap, rc::Rc};

use super::pipeline::VisibleRow;

/// A tree row's place among the shown rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TreeLevel {
    /// 0 for a root.
    pub depth: usize,
    /// Its shown children; 0 for a leaf.
    pub children: usize,
    /// Whether its children show.
    pub open: bool,
    /// 1-based, among its shown siblings.
    pub posinset: usize,
    pub setsize: usize,
}

/// The rows in tree order and each shown row's level, by `data` index.
pub(super) type TreeRows = (Vec<VisibleRow>, Rc<[Option<TreeLevel>]>);

/// Each row's parent by `data` index, and how many parent keys named no row: those
/// rows are roots, as are the rows of a cycle.
pub(super) fn tree_parents(keys: &[String], parents: &[Option<String>]) -> (Vec<usize>, usize) {
    let at: HashMap<&str, usize> = keys
        .iter()
        .enumerate()
        .map(|(index, key)| (key.as_str(), index))
        .collect();
    let mut unknown = 0;
    let mut parent = Vec::with_capacity(parents.len());
    for (index, key) in parents.iter().enumerate() {
        let found = key.as_deref().and_then(|key| at.get(key).copied());
        if key.is_some() && found.is_none() {
            unknown += 1;
        }
        parent.push(found.unwrap_or(index));
    }
    break_cycles(&mut parent);
    (parent, unknown)
}

/// Makes roots of the rows on a cycle. A root is its own parent.
fn break_cycles(parent: &mut [usize]) {
    // 0 unseen, 1 on the current walk up, 2 done.
    let mut state = vec![0u8; parent.len()];
    let mut walk = Vec::new();
    for start in 0..parent.len() {
        let mut at = start;
        while state[at] == 0 {
            state[at] = 1;
            walk.push(at);
            let up = parent[at];
            if up == at {
                break;
            }
            if state[up] == 1 {
                let from = walk.iter().position(|&row| row == up).unwrap_or(0);
                for &row in &walk[from..] {
                    parent[row] = row;
                }
                break;
            }
            at = up;
        }
        for row in walk.drain(..) {
            state[row] = 2;
        }
    }
}

/// The filter's `kept` rows and their ancestors, and the rows open to show a kept descendant.
pub(super) fn with_ancestors(kept: &[bool], parent: &[usize]) -> (Vec<bool>, Vec<bool>) {
    let mut shown = kept.to_vec();
    let mut opened = vec![false; kept.len()];
    for row in (0..kept.len()).filter(|&row| kept[row]) {
        let mut at = row;
        while parent[at] != at && !opened[parent[at]] {
            at = parent[at];
            opened[at] = true;
            shown[at] = true;
        }
    }
    (shown, opened)
}

/// The sorted `order` nested under the parents: each level keeps the sort, an `open`
/// row shows its children after it. A row whose parent is not in `order` is a root.
pub(super) fn tree_rows(
    order: &[usize],
    parent: &[usize],
    open: &dyn Fn(usize) -> bool,
) -> TreeRows {
    let mut shown = vec![false; parent.len()];
    for &row in order {
        shown[row] = true;
    }
    let mut children: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut roots = Vec::new();
    for &row in order {
        match parent[row] {
            up if up != row && shown[up] => children.entry(up).or_default().push(row),
            _ => roots.push(row),
        }
    }
    let none = Vec::new();
    let mut levels = vec![None; parent.len()];
    let mut rows = Vec::with_capacity(order.len());
    // Each open level: its siblings and the next one to show. No recursion: a deep chain.
    let mut stack: Vec<(&[usize], usize)> = vec![(&roots, 0)];
    while let Some((siblings, next)) = stack.last_mut() {
        let Some(&row) = siblings.get(*next) else {
            stack.pop();
            continue;
        };
        *next += 1;
        let (posinset, setsize) = (*next, siblings.len());
        let kids = children.get(&row).unwrap_or(&none);
        let open = !kids.is_empty() && open(row);
        levels[row] = Some(TreeLevel {
            depth: stack.len() - 1,
            children: kids.len(),
            open,
            posinset,
            setsize,
        });
        rows.push(VisibleRow::Data(row));
        if open {
            stack.push((kids, 0));
        }
    }
    (rows, levels.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(keys: &[&str]) -> Vec<String> {
        keys.iter().map(|key| key.to_string()).collect()
    }

    fn parents(parents: &[Option<&str>]) -> Vec<Option<String>> {
        parents.iter().map(|key| key.map(String::from)).collect()
    }

    fn data(rows: &[usize]) -> Vec<VisibleRow> {
        rows.iter().copied().map(VisibleRow::Data).collect()
    }

    #[test]
    fn rows_nest_under_their_parents_keeping_the_sort_per_level() {
        // a > (b > d), c; sorted d, c, b, a.
        let (parent, unknown) = tree_parents(
            &keys(&["a", "b", "c", "d"]),
            &parents(&[None, Some("a"), Some("a"), Some("b")]),
        );
        assert_eq!((parent.as_slice(), unknown), (&[0, 0, 0, 1][..], 0));
        let (rows, levels) = tree_rows(&[3, 2, 1, 0], &parent, &|_| true);
        assert_eq!(rows, data(&[0, 2, 1, 3]));
        let level = levels[1].unwrap();
        assert_eq!(
            (level.depth, level.children, level.posinset, level.setsize),
            (1, 1, 2, 2)
        );
        assert_eq!(levels[3].unwrap().depth, 2);
    }

    #[test]
    fn a_closed_row_hides_its_descendants() {
        let (parent, _) = tree_parents(
            &keys(&["a", "b", "c"]),
            &parents(&[None, Some("a"), Some("b")]),
        );
        let (rows, levels) = tree_rows(&[0, 1, 2], &parent, &|row| row == 1);
        assert_eq!(rows, data(&[0]));
        assert!(!levels[0].unwrap().open && levels[0].unwrap().children == 1);
        assert_eq!(levels[2], None);
    }

    #[test]
    fn an_unknown_parent_and_a_cycle_make_roots() {
        // b names no row; c and d name each other; e hangs under the cycle.
        let (parent, unknown) = tree_parents(
            &keys(&["a", "b", "c", "d", "e"]),
            &parents(&[Some("a"), Some("x"), Some("d"), Some("c"), Some("c")]),
        );
        assert_eq!(unknown, 1);
        assert_eq!(parent, [0, 1, 2, 3, 2]);
        let (rows, _) = tree_rows(&[0, 1, 2, 3, 4], &parent, &|_| true);
        assert_eq!(rows, data(&[0, 1, 2, 4, 3]));
    }

    #[test]
    fn a_long_chain_flattens_without_recursion() {
        let count: usize = 50_000;
        let parent: Vec<usize> = (0..count).map(|row| row.saturating_sub(1)).collect();
        let order: Vec<usize> = (0..count).collect();
        let (rows, levels) = tree_rows(&order, &parent, &|_| true);
        assert_eq!(rows.len(), count);
        assert_eq!(levels[count - 1].unwrap().depth, count - 1);
    }

    #[test]
    fn the_filter_keeps_and_opens_the_ancestors_of_a_kept_row() {
        // a > b > c, d: only c matches.
        let parent = [0, 0, 1, 3];
        let (shown, opened) = with_ancestors(&[false, false, true, false], &parent);
        assert_eq!(shown, [true, true, true, false]);
        assert_eq!(opened, [true, true, false, false]);
        let order: Vec<usize> = (0..4).filter(|&row| shown[row]).collect();
        let (rows, _) = tree_rows(&order, &parent, &|row| opened[row]);
        assert_eq!(rows, data(&[0, 1, 2]));
    }
}
