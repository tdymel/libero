//! [`UndoHistory`], a snapshot undo/redo stack without dioxus, so it runs anywhere.

use std::collections::VecDeque;
use std::rc::Rc;

/// Snapshots to step back and forth through. Each entry is an `Rc`, so a
/// clone of the history or of a snapshot is cheap.
///
/// ```rust
/// use libero::hooks::UndoHistory;
///
/// let mut history = UndoHistory::new(String::new());
/// history.merge("h".to_string());
/// history.merge("hi".to_string());
/// history.seal();
/// history.push("hi!".to_string());
///
/// history.undo();
/// assert_eq!(**history.present(), "hi");
/// history.undo();
/// assert_eq!(**history.present(), "");
/// history.redo();
/// assert_eq!(**history.present(), "hi");
/// ```
pub struct UndoHistory<T> {
    past: VecDeque<Rc<T>>,
    present: Rc<T>,
    future: Vec<Rc<T>>,
    cap: usize,
    group_max: usize,
    /// Changes merged into the present entry's open group; 0 when none is open.
    group: usize,
}

impl<T> Clone for UndoHistory<T> {
    fn clone(&self) -> Self {
        Self {
            past: self.past.clone(),
            present: self.present.clone(),
            future: self.future.clone(),
            cap: self.cap,
            group_max: self.group_max,
            group: self.group,
        }
    }
}

impl<T: PartialEq> PartialEq for UndoHistory<T> {
    fn eq(&self, other: &Self) -> bool {
        self.past == other.past
            && self.present == other.present
            && self.future == other.future
            && self.cap == other.cap
            && self.group_max == other.group_max
            && self.group == other.group
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for UndoHistory<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UndoHistory")
            .field("past", &self.past)
            .field("present", &self.present)
            .field("future", &self.future)
            .finish_non_exhaustive()
    }
}

impl<T> UndoHistory<T> {
    /// Undo steps kept by default.
    pub const DEFAULT_CAP: usize = 200;
    /// Changes one group merges by default before the next starts a new entry.
    pub const DEFAULT_GROUP_MAX: usize = 50;

    /// A history at `initial`, nothing to undo, [`Self::DEFAULT_CAP`] undo steps.
    pub fn new(initial: T) -> Self {
        Self {
            past: VecDeque::new(),
            present: Rc::new(initial),
            future: Vec::new(),
            cap: Self::DEFAULT_CAP,
            group_max: Self::DEFAULT_GROUP_MAX,
            group: 0,
        }
    }

    /// Keeps at most `cap` undo steps (at least 1); the oldest drop first.
    pub fn with_cap(mut self, cap: usize) -> Self {
        self.cap = cap.max(1);
        self.trim();
        self
    }

    /// Lets one group merge at most `max` changes (at least 1).
    pub fn with_group_max(mut self, max: usize) -> Self {
        self.group_max = max.max(1);
        self
    }

    /// The current snapshot.
    pub fn present(&self) -> &Rc<T> {
        &self.present
    }

    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    /// Records `value` as a new entry of its own. Clears the redo steps.
    pub fn push(&mut self, value: T) {
        self.seal();
        self.advance(value);
    }

    /// Records `value` into the open group, replacing its snapshot, so one undo
    /// steps over the whole group. Opens a new group when none is open or the
    /// open one is full. Clears the redo steps.
    pub fn merge(&mut self, value: T) {
        match self.group > 0 && self.group < self.group_max {
            true => {
                self.present = Rc::new(value);
                self.group += 1;
            }
            false => {
                self.advance(value);
                self.group = 1;
            }
        }
    }

    /// Closes the open group: the next [`merge`](Self::merge) starts a new entry.
    pub fn seal(&mut self) {
        self.group = 0;
    }

    /// Steps back one entry. `false` when there is nothing to undo.
    pub fn undo(&mut self) -> bool {
        self.seal();
        let Some(previous) = self.past.pop_back() else {
            return false;
        };
        let current = std::mem::replace(&mut self.present, previous);
        self.future.push(current);
        true
    }

    /// Steps forward one undone entry. `false` when there is nothing to redo.
    pub fn redo(&mut self) -> bool {
        self.seal();
        let Some(next) = self.future.pop() else {
            return false;
        };
        let current = std::mem::replace(&mut self.present, next);
        self.past.push_back(current);
        true
    }

    /// Starts over at `value`, nothing to undo or redo.
    pub fn reset(&mut self, value: T) {
        self.past.clear();
        self.future.clear();
        self.present = Rc::new(value);
        self.group = 0;
    }

    fn advance(&mut self, value: T) {
        let current = std::mem::replace(&mut self.present, Rc::new(value));
        self.past.push_back(current);
        self.future.clear();
        self.trim();
    }

    fn trim(&mut self) {
        while self.past.len() > self.cap {
            self.past.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(history: &UndoHistory<i32>) -> i32 {
        **history.present()
    }

    #[test]
    fn undo_and_redo_walk_the_pushed_entries() {
        let mut history = UndoHistory::new(0);
        assert!(!history.can_undo() && !history.can_redo());
        history.push(1);
        history.push(2);

        assert!(history.undo());
        assert_eq!(at(&history), 1);
        assert!(history.undo());
        assert_eq!(at(&history), 0);
        assert!(!history.undo(), "undid past the start");
        assert_eq!(at(&history), 0);

        assert!(history.redo());
        assert!(history.redo());
        assert_eq!(at(&history), 2);
        assert!(!history.redo(), "redid past the end");
    }

    #[test]
    fn a_push_after_an_undo_drops_the_redo_steps() {
        let mut history = UndoHistory::new(0);
        history.push(1);
        history.push(2);
        history.undo();
        history.push(3);

        assert!(!history.can_redo());
        history.undo();
        assert_eq!(at(&history), 1);
    }

    #[test]
    fn the_cap_drops_the_oldest_steps() {
        let mut history = UndoHistory::new(0).with_cap(2);
        for value in 1..=5 {
            history.push(value);
        }
        assert!(history.undo() && history.undo());
        assert_eq!(at(&history), 3);
        assert!(!history.can_undo());
    }

    #[test]
    fn the_default_cap_is_two_hundred() {
        let mut history = UndoHistory::new(0);
        for value in 1..=250 {
            history.push(value);
        }
        let mut steps = 0;
        while history.undo() {
            steps += 1;
        }
        assert_eq!(steps, UndoHistory::<i32>::DEFAULT_CAP);
        assert_eq!(at(&history), 50);
    }

    #[test]
    fn merged_changes_undo_as_one_step() {
        let mut history = UndoHistory::new(0);
        history.merge(1);
        history.merge(2);
        history.merge(3);
        assert_eq!(at(&history), 3);

        history.undo();
        assert_eq!(at(&history), 0);
        history.redo();
        assert_eq!(at(&history), 3);
    }

    #[test]
    fn seal_starts_a_new_group() {
        let mut history = UndoHistory::new(0);
        history.merge(1);
        history.merge(2);
        history.seal();
        history.merge(3);
        history.merge(4);

        history.undo();
        assert_eq!(at(&history), 2);
        history.undo();
        assert_eq!(at(&history), 0);
    }

    #[test]
    fn a_full_group_starts_a_new_one() {
        let mut history = UndoHistory::new(0).with_group_max(2);
        for value in 1..=5 {
            history.merge(value);
        }
        // Groups: [1, 2], [3, 4], [5].
        history.undo();
        assert_eq!(at(&history), 4);
        history.undo();
        assert_eq!(at(&history), 2);
        history.undo();
        assert_eq!(at(&history), 0);
    }

    #[test]
    fn a_push_closes_the_group_and_a_merge_after_it_opens_another() {
        let mut history = UndoHistory::new(0);
        history.merge(1);
        history.push(2);
        history.merge(3);

        history.undo();
        assert_eq!(at(&history), 2);
        history.undo();
        assert_eq!(at(&history), 1);
    }

    #[test]
    fn a_merge_after_an_undo_does_not_overwrite_the_restored_entry() {
        let mut history = UndoHistory::new(0);
        history.merge(1);
        history.undo();
        history.merge(5);

        assert!(!history.can_redo());
        history.undo();
        assert_eq!(at(&history), 0);
    }

    #[test]
    fn undo_seals_the_open_group() {
        let mut history = UndoHistory::new(0);
        history.merge(1);
        history.merge(2);
        history.undo();
        history.redo();
        history.merge(3);

        history.undo();
        assert_eq!(at(&history), 2, "the merge after redo joined the old group");
    }

    #[test]
    fn reset_forgets_every_step() {
        let mut history = UndoHistory::new(0);
        history.push(1);
        history.push(2);
        history.undo();
        history.reset(9);

        assert_eq!(at(&history), 9);
        assert!(!history.can_undo() && !history.can_redo());
    }

    #[test]
    fn a_clone_shares_the_snapshots() {
        let mut history = UndoHistory::new(vec![0; 1024]);
        history.push(vec![1; 1024]);
        let copy = history.clone();
        assert!(Rc::ptr_eq(history.present(), copy.present()));
        assert_eq!(history, copy);
    }
}
