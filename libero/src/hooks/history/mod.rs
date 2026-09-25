//! `use_history`, a reactive [`UndoHistory`] whose merged changes group by time.

mod stack;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;

use std::rc::Rc;

use dioxus::prelude::*;

pub use stack::UndoHistory;

use super::timers::{Scheduled, use_latest_ms, use_scheduled};

/// The [`UndoHistory`] of [`use_history`]. `Copy`, so handlers can move it.
pub struct HistoryHandle<T: 'static> {
    history: CopyValue<UndoHistory<T>>,
    /// Bumped on every visible change; sealing a group changes nothing visible.
    changed: Signal<u32>,
    quiet: Scheduled,
    group_ms: CopyValue<u64>,
}

impl<T> Clone for HistoryHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for HistoryHandle<T> {}

impl<T> PartialEq for HistoryHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.history == other.history
    }
}

impl<T: 'static> HistoryHandle<T> {
    /// The current snapshot. Reactive.
    pub fn value(&self) -> Rc<T> {
        self.read(|history| history.present().clone())
    }

    /// Reads the whole history. Reactive.
    pub fn read<R>(&self, read: impl FnOnce(&UndoHistory<T>) -> R) -> R {
        (self.changed)();
        read(&self.history.peek())
    }

    /// Whether there is a step to undo. Reactive.
    pub fn can_undo(&self) -> bool {
        self.read(UndoHistory::can_undo)
    }

    /// Whether there is a step to redo. Reactive.
    pub fn can_redo(&self) -> bool {
        self.read(UndoHistory::can_redo)
    }

    /// Records `value` as an entry of its own, see [`UndoHistory::push`].
    pub fn push(&self, value: T) {
        self.quiet.cancel();
        self.change(|history| history.push(value));
    }

    /// Records `value` into the open group, see [`UndoHistory::merge`]. The group
    /// closes after `group_ms` without another merge.
    pub fn merge(&self, value: T) {
        self.change(|history| history.merge(value));
        match *self.group_ms.peek() {
            0 => self.quiet.cancel(),
            ms => self.quiet.after(ms),
        }
    }

    /// Closes the open group now, see [`UndoHistory::seal`].
    pub fn seal(&self) {
        self.quiet.cancel();
        let mut history = self.history;
        history.write().seal();
    }

    /// Steps back one entry. `false` when there is nothing to undo.
    pub fn undo(&self) -> bool {
        self.quiet.cancel();
        self.change(UndoHistory::undo)
    }

    /// Steps forward one undone entry. `false` when there is nothing to redo.
    pub fn redo(&self) -> bool {
        self.quiet.cancel();
        self.change(UndoHistory::redo)
    }

    /// Starts over at `value`, nothing to undo or redo.
    pub fn reset(&self, value: T) {
        self.quiet.cancel();
        self.change(|history| history.reset(value));
    }

    fn change<R>(&self, change: impl FnOnce(&mut UndoHistory<T>) -> R) -> R {
        let mut history = self.history;
        let result = change(&mut history.write());
        let mut changed = self.changed;
        changed += 1;
        result
    }
}

/// Undo and redo over snapshots of a value. [`merge`](HistoryHandle::merge)
/// groups rapid changes, such as typing, into one undo step;
/// [`push`](HistoryHandle::push) records a step of its own.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::{UndoHistory, use_history};
/// # fn app() -> Element {
/// let text = use_history(|| UndoHistory::new(String::new()), 500);
///
/// rsx! {
///     input {
///         value: "{text.value()}",
///         oninput: move |event| text.merge(event.value()),
///     }
///     button { disabled: !text.can_undo(), onclick: move |_| { text.undo(); }, "Undo" }
///     button { disabled: !text.can_redo(), onclick: move |_| { text.redo(); }, "Redo" }
/// }
/// # }
/// ```
///
/// `initial` builds the [`UndoHistory`], so it sets the cap and group size too.
/// A group closes on whichever comes first: `group_ms` without a merge, its
/// [`UndoHistory::with_group_max`] changes, or a [`seal`](HistoryHandle::seal),
/// `push`, `undo` or `redo`. `group_ms` 0 never closes one by time, nor does a
/// target without a timer, such as a server render.
pub fn use_history<T: 'static>(
    initial: impl FnOnce() -> UndoHistory<T>,
    group_ms: u64,
) -> HistoryHandle<T> {
    let group_ms = use_latest_ms(group_ms);
    let history = use_hook(|| CopyValue::new(initial()));
    let changed = use_signal(|| 0);
    let quiet = use_scheduled(move |_| {
        let mut history = history;
        history.write().seal();
    });
    HistoryHandle {
        history,
        changed,
        quiet,
        group_ms,
    }
}
