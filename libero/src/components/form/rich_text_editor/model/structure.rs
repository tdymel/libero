//! Container edits: lists, nesting and quotes.

use super::doc::{Block, BlockKind, NodeKey};
use super::state::EditorState;

impl EditorState {
    /// Bullet (`ordered: false`) or numbered list. Lifts the selection out when it
    /// is in such a list already, switches the kind of a list of the other kind,
    /// and wraps each selected block in an item otherwise.
    pub fn toggle_list(&mut self, ordered: bool) -> bool {
        let (from, to) = self.ordered();
        let from_list = self.nearest(from.block, |kind| matches!(kind, BlockKind::List { .. }));
        let to_list = self.nearest(to.block, |kind| matches!(kind, BlockKind::List { .. }));
        if let Some(list) = from_list.filter(|list| Some(*list) == to_list) {
            let BlockKind::List {
                ordered: current, ..
            } = self.block(list).kind
            else {
                unreachable!("a list")
            };
            if current != ordered {
                let block = self.doc.get_mut(list).expect("the list");
                block.kind = match ordered {
                    true => BlockKind::ordered_list(1),
                    false => BlockKind::bullet_list(),
                };
                return true;
            }
            let items: Vec<NodeKey> = [from.block, to.block]
                .iter()
                .filter_map(|leaf| self.item_in(list, *leaf))
                .collect();
            let path = self.doc.path(list).expect("the list");
            let children = self.doc.at(&path).children();
            let first = children
                .iter()
                .position(|item| item.key == items[0])
                .unwrap_or(0);
            let last = children
                .iter()
                .position(|item| item.key == items[1])
                .unwrap_or(first);
            let lifted: Vec<NodeKey> = children[first..=last].iter().map(|item| item.key).collect();
            for item in lifted.into_iter().rev() {
                self.lift_item(item);
            }
            self.prune();
            self.clamp();
            return true;
        }
        let range = self.sibling_range();
        let Some((parent_path, first, last)) = range else {
            return false;
        };
        let siblings = self.doc.siblings_mut(&parent_path);
        let taken: Vec<Block> = siblings.drain(first..=last).collect();
        let mut items = Vec::new();
        for block in taken {
            match block.kind {
                BlockKind::List { .. } => items.extend(block.into_children()),
                _ => items.push(self.doc.container(BlockKind::list_item(), vec![block])),
            }
        }
        let kind = match ordered {
            true => BlockKind::ordered_list(1),
            false => BlockKind::bullet_list(),
        };
        let list = self.doc.container(kind, items);
        self.doc.siblings_mut(&parent_path).insert(first, list);
        true
    }

    /// Makes the selected list items open tasks, or plain items when all are tasks.
    /// Outside a list it makes a bullet list of tasks.
    pub fn toggle_task_list(&mut self) -> bool {
        if self.selected_items().is_empty() && !self.toggle_list(false) {
            return false;
        }
        let items = self.selected_items();
        let all_tasks = items.iter().all(|item| self.is_task(*item));
        for item in items {
            let block = self.doc.get_mut(item).expect("the item");
            block.kind = match (all_tasks, &block.kind) {
                (true, _) => BlockKind::list_item(),
                (false, BlockKind::ListItem { checked: Some(_) }) => continue,
                (false, _) => BlockKind::task_item(false),
            };
        }
        true
    }

    /// Checks the selected task items, or unchecks them when all are checked already.
    pub fn toggle_task(&mut self) -> bool {
        let tasks: Vec<NodeKey> = self
            .selected_items()
            .into_iter()
            .filter(|item| self.is_task(*item))
            .collect();
        let checked =
            |state: &Self, item: NodeKey| state.block(item).kind == BlockKind::task_item(true);
        let check = !tasks.iter().all(|item| checked(self, *item));
        for item in &tasks {
            self.doc.get_mut(*item).expect("the item").kind = BlockKind::task_item(check);
        }
        !tasks.is_empty()
    }

    /// Whether the caret sits in a task item, and whether that one is checked.
    pub fn task_state(&self) -> Option<bool> {
        let item = self.nearest(self.caret().block, BlockKind::is_list_item)?;
        match self.block(item).kind {
            BlockKind::ListItem { checked } => checked,
            _ => None,
        }
    }

    fn is_task(&self, item: NodeKey) -> bool {
        matches!(
            self.block(item).kind,
            BlockKind::ListItem { checked: Some(_) }
        )
    }

    /// The innermost list item around each selected leaf, in document order.
    fn selected_items(&self) -> Vec<NodeKey> {
        let (from, to) = self.ordered();
        let leaves = self.doc.leaves();
        let range = self.leaf_index(from.block)..=self.leaf_index(to.block);
        let mut items: Vec<NodeKey> = Vec::new();
        for leaf in &leaves[range] {
            if let Some(item) = self.nearest(*leaf, BlockKind::is_list_item)
                && !items.contains(&item)
            {
                items.push(item);
            }
        }
        items
    }

    /// Tab in a list item: nests it under the item before it.
    pub fn indent(&mut self) -> bool {
        let at = self.caret().block;
        let Some(item) = self.nearest(at, BlockKind::is_list_item) else {
            return false;
        };
        let path = self.doc.path(item).expect("the item");
        let index = *path.last().expect("a path");
        if index == 0 {
            return false;
        }
        let list_kind = self.doc.at(&path[..path.len() - 1]).kind.clone();
        let moved = self.doc.siblings_mut(&path).remove(index);
        let mut previous_path = path.clone();
        *previous_path.last_mut().expect("a path") = index - 1;
        let nested_list = {
            let previous = self.doc.at(&previous_path);
            previous
                .children()
                .last()
                .filter(|last| last.kind == list_kind)
                .map(|last| last.key)
        };
        match nested_list {
            Some(list) => self
                .doc
                .get_mut(list)
                .expect("the list")
                .children_mut()
                .push(moved),
            None => {
                let list = self.doc.container(list_kind, vec![moved]);
                self.doc.at_mut(&previous_path).children_mut().push(list);
            }
        }
        true
    }

    /// Shift+Tab in a list item: moves it one level out, taking the items after
    /// it along as its children. A top-level item leaves the list.
    pub fn outdent(&mut self) -> bool {
        let at = self.caret().block;
        let Some(item) = self.nearest(at, BlockKind::is_list_item) else {
            return false;
        };
        let changed = self.lift_item(item);
        self.prune();
        self.clamp();
        changed
    }

    /// Moves `item` out of its list: into the outer list, or out of lists altogether.
    fn lift_item(&mut self, item: NodeKey) -> bool {
        let path = self.doc.path(item).expect("the item");
        let index = *path.last().expect("a path");
        let list_path = path[..path.len() - 1].to_vec();
        let list_kind = self.doc.at(&list_path).kind.clone();
        let list_key = self.doc.at(&list_path).key;
        let trailing = self
            .doc
            .at_mut(&list_path)
            .children_mut()
            .split_off(index + 1);
        let mut moved = self.doc.at_mut(&list_path).children_mut().remove(index);
        let outer = self.parent(list_key);
        match outer {
            Some((outer_item, BlockKind::ListItem { .. }, _)) => {
                if !trailing.is_empty() {
                    let nested = self.doc.container(list_kind, trailing);
                    moved.children_mut().push(nested);
                }
                self.insert_after(outer_item, false, moved);
            }
            _ => {
                let mut after: Vec<Block> = std::mem::take(moved.children_mut());
                if !trailing.is_empty() {
                    let start = match list_kind {
                        BlockKind::List {
                            ordered: true,
                            start,
                        } => start + index as u64 + 1,
                        _ => 1,
                    };
                    let kind = match list_kind {
                        BlockKind::List { ordered: true, .. } => BlockKind::ordered_list(start),
                        kind => kind,
                    };
                    after.push(self.doc.container(kind, trailing));
                }
                for block in after.into_iter().rev() {
                    self.insert_after(list_key, false, block);
                }
            }
        }
        true
    }

    /// Wraps the selected blocks in a quote, or unwraps the quote around the caret.
    pub fn toggle_quote(&mut self) -> bool {
        let at = self.caret().block;
        if let Some(quote) = self.nearest(at, |kind| *kind == BlockKind::Quote) {
            let path = self.doc.path(quote).expect("the quote");
            let index = *path.last().expect("a path");
            let block = self.doc.siblings_mut(&path).remove(index);
            let siblings = self.doc.siblings_mut(&path);
            for (offset, child) in block.into_children().into_iter().enumerate() {
                siblings.insert(index + offset, child);
            }
            return true;
        }
        let Some((parent_path, first, last)) = self.sibling_range() else {
            return false;
        };
        let taken: Vec<Block> = self
            .doc
            .siblings_mut(&parent_path)
            .drain(first..=last)
            .collect();
        let quote = self.doc.container(BlockKind::Quote, taken);
        self.doc.siblings_mut(&parent_path).insert(first, quote);
        true
    }

    /// Moves a quote's first child out before it (Backspace), any other after it (Enter).
    pub(super) fn lift_out_of_quote(&mut self, leaf: NodeKey) -> bool {
        let Some((quote, BlockKind::Quote, index)) = self.parent(leaf) else {
            return false;
        };
        let block = self.remove(leaf).expect("the leaf");
        self.insert_after(quote, index == 0, block);
        self.prune();
        self.clamp();
        true
    }

    /// The nearest container around `key` whose kind matches.
    pub(super) fn nearest(
        &self,
        key: NodeKey,
        matches: impl Fn(&BlockKind) -> bool,
    ) -> Option<NodeKey> {
        self.ancestors(key)
            .into_iter()
            .find(|(_, kind)| matches(kind))
            .map(|(key, _)| key)
    }

    /// The child of `list` that holds `leaf`.
    fn item_in(&self, list: NodeKey, leaf: NodeKey) -> Option<NodeKey> {
        let list_path = self.doc.path(list)?;
        let leaf_path = self.doc.path(leaf)?;
        let item_path = leaf_path.get(..=list_path.len())?;
        Some(self.doc.at(item_path).key)
    }

    /// The selection's blocks as one sibling range: the deepest container holding
    /// both ends, and the first and last of its children they sit in.
    fn sibling_range(&self) -> Option<(Vec<usize>, usize, usize)> {
        let (from, to) = self.ordered();
        let a = self.doc.path(from.block)?;
        let b = self.doc.path(to.block)?;
        let common = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
        let depth = common.min(a.len() - 1).min(b.len() - 1);
        Some((a[..=depth].to_vec(), a[depth], b[depth]))
    }
}
