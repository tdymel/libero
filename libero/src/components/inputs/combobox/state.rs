use dioxus::prelude::*;

use crate::hooks::use_id;

use super::aria::trigger_aria;

/// A [`Combobox`](super::Combobox)'s open state, its keyboard highlight, and
/// the id that ties the two together - held in the caller's own scope, so the
/// component holds nothing.
///
/// It exists because `aria-activedescendant` belongs on the focused trigger,
/// and the trigger is the one element `Combobox` does not render.
/// [`a11y_attributes`](Self::a11y_attributes) hands that wiring out; the rest
/// is the open state, to drive however the control wants to.
#[derive(Clone, Copy, PartialEq)]
pub struct ComboboxState {
    id: Signal<String>,
    opened: Signal<bool>,
    active: Signal<usize>,
}

/// One `Combobox`'s state. Positional, like every hook.
pub fn use_combobox() -> ComboboxState {
    ComboboxState {
        id: use_id(),
        opened: use_signal(|| false),
        active: use_signal(|| 0),
    }
}

impl ComboboxState {
    /// The id every part of the wiring is built from.
    pub fn id(&self) -> String {
        (self.id)()
    }

    pub fn opened(&self) -> bool {
        (self.opened)()
    }

    pub fn set_opened(&self, opened: bool) {
        let mut open = self.opened;
        open.set(opened);
    }

    pub fn open(&self) {
        self.set_opened(true);
    }

    pub fn close(&self) {
        self.set_opened(false);
    }

    pub fn toggle(&self) {
        self.set_opened(!self.opened());
    }

    /// The row the arrow keys are on - an index into the `options` last handed
    /// to the `Combobox`.
    pub fn active(&self) -> usize {
        (self.active)()
    }

    pub fn set_active(&self, row: usize) {
        let mut active = self.active;
        active.set(row);
    }

    /// `role`, `aria-haspopup`, `aria-expanded`, `aria-controls` and, while the
    /// list is open, `aria-activedescendant`. Spread it on whatever control
    /// sits inside the `Combobox`:
    ///
    /// ```ignore
    /// TextField { attributes: fruit.a11y_attributes(), value: text() }
    /// ```
    pub fn a11y_attributes(&self) -> Vec<Attribute> {
        let opened = self.opened();
        trigger_aria(&self.id(), opened, opened.then(|| self.active()))
    }
}
