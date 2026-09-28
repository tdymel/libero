use dioxus::prelude::*;

use crate::hooks::use_id;

use super::combobox_aria::trigger_aria;

/// A [`Combobox`](crate::components::Combobox)'s open state and keyboard highlight, held
/// in the caller's scope. Spread [`a11y_attributes`](Self::a11y_attributes) on the trigger.
#[derive(Clone, Copy, PartialEq)]
pub struct ComboboxState {
    id: Signal<String>,
    opened: Signal<bool>,
    active: Signal<Option<usize>>,
    /// Rows the open list drew, written by the list: the trigger must not name a missing row.
    rows: Signal<usize>,
    /// Open, but refused by a disabled or read-only list, written by the list.
    held: Signal<bool>,
}

/// One `Combobox`'s state. Positional, like every hook.
pub fn use_combobox() -> ComboboxState {
    ComboboxState {
        id: use_id(),
        opened: use_signal(|| false),
        active: use_signal(|| None),
        rows: use_signal(|| 0),
        held: use_signal(|| false),
    }
}

impl ComboboxState {
    /// The id every part of the wiring is built from.
    pub fn id(&self) -> String {
        (self.id)()
    }

    pub fn is_open(&self) -> bool {
        (self.opened)()
    }

    pub fn set_open(&self, opened: bool) {
        let mut open = self.opened;
        open.set(opened);
    }

    pub fn open(&self) {
        self.set_open(true);
    }

    pub fn close(&self) {
        self.set_open(false);
    }

    pub fn toggle(&self) {
        self.set_open(!self.is_open());
    }

    /// The row the arrow keys are on, an index into the `options`. `None` leaves Enter to
    /// the control.
    pub fn active(&self) -> Option<usize> {
        (self.active)()
    }

    /// [`active`](Self::active) unsubscribed, for a key press that must not wait for a render.
    pub(crate) fn active_now(&self) -> Option<usize> {
        *self.active.peek()
    }

    pub fn set_active(&self, row: Option<usize>) {
        let mut active = self.active;
        active.set(row);
    }

    /// Peek-compared, so an unchanged count does not re-render the caller.
    pub(crate) fn set_rows(&self, count: usize) {
        let mut rows = self.rows;
        if *rows.peek() != count {
            rows.set(count);
        }
    }

    /// Moves a highlight the list snapped off a refused row, so the trigger names the lit row.
    pub(crate) fn snap_active(&self, from: Option<usize>, to: usize) {
        let mut active = self.active;
        if *active.peek() == from && from != Some(to) {
            active.set(Some(to));
        }
    }

    /// Peek-compared like `set_rows`.
    pub(crate) fn set_held(&self, held: bool) {
        let mut signal = self.held;
        if *signal.peek() != held {
            signal.set(held);
        }
    }

    /// The trigger's combobox ARIA. `aria-controls` only while the listbox is in the DOM (todo 360).
    ///
    /// ```no_run
    /// # use dioxus::prelude::*;
    /// # use libero::components::{TextField, use_combobox};
    /// # fn app() -> Element {
    /// # let fruit = use_combobox();
    /// # let text = use_signal(String::new);
    /// # rsx! {
    /// TextField { attributes: fruit.a11y_attributes(), value: text() }
    /// # } }
    /// ```
    pub fn a11y_attributes(&self) -> Vec<Attribute> {
        // `ComboboxCore` mounts the listbox only while it has rows to draw.
        self.aria(self.is_open() && (self.rows)() > 0)
    }

    /// [`a11y_attributes`](Self::a11y_attributes) for a list that says itself whether it is mounted.
    pub(crate) fn aria(&self, listbox: bool) -> Vec<Attribute> {
        let opened = self.is_open() && !(self.held)();
        let listbox = listbox && opened;
        // Clamped as the list clamps its highlight.
        let rows = (self.rows)();
        let active = self
            .active()
            .filter(|_| opened && rows > 0)
            .map(|row| row.min(rows - 1));
        trigger_aria(&self.id(), opened, listbox, active)
    }
}
