use dioxus::prelude::*;

use crate::{components::common::attr, hooks::use_id};

/// Which item takes focus when the menu opens. APG's menu button: ArrowUp on
/// the trigger opens on the last item, everything else on the first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum MenuFocus {
    #[default]
    First,
    Last,
}

/// A [`Menu`](super::Menu)'s open state and the id that ties the trigger to
/// the menu - held in the caller's own scope, like `use_combobox`.
///
/// It exists because the trigger is the one element `Menu` does not render:
/// [`a11y_attributes`](Self::a11y_attributes) hands its wiring out.
#[derive(Clone, Copy, PartialEq)]
pub struct MenuState {
    id: Signal<String>,
    opened: Signal<bool>,
    /// Bumped on every open, so the menu focuses an item again even when the
    /// same item is asked for twice - ArrowDown on the trigger of a menu that
    /// is already open.
    request: Signal<(u64, MenuFocus)>,
}

/// One `Menu`'s state. Positional, like every hook.
pub fn use_menu() -> MenuState {
    MenuState {
        id: use_id(),
        opened: use_signal(|| false),
        request: use_signal(|| (0, MenuFocus::First)),
    }
}

impl MenuState {
    /// Not a hook: signals owned by the current scope, for a component that
    /// holds a runtime number of menus - `Menubar`, one per menu, created on
    /// first need and kept. `use_menu()` in a loop over a `Vec` would hand
    /// hook slots from one menu to another as the `Vec` changes.
    pub(crate) fn new(id: String) -> Self {
        Self {
            id: Signal::new(id),
            opened: Signal::new(false),
            request: Signal::new((0, MenuFocus::First)),
        }
    }

    /// The id every part of the wiring is built from.
    pub fn id(&self) -> String {
        (self.id)()
    }

    pub fn is_open(&self) -> bool {
        (self.opened)()
    }

    /// Opens the menu with its first item focused.
    pub fn open(&self) {
        self.open_at(MenuFocus::First);
    }

    pub fn close(&self) {
        let mut opened = self.opened;
        if *opened.peek() {
            opened.set(false);
        }
    }

    pub fn toggle(&self) {
        // Copied out first: matching on the `peek()` itself holds the borrow
        // through the arm that writes.
        let opened = *self.opened.peek();
        match opened {
            true => self.close(),
            false => self.open(),
        }
    }

    pub(crate) fn open_at(&self, focus: MenuFocus) {
        let mut request = self.request;
        let next = request.peek().0.wrapping_add(1);
        request.set((next, focus));
        let mut opened = self.opened;
        if !*opened.peek() {
            opened.set(true);
        }
    }

    pub(crate) fn request(&self) -> (u64, MenuFocus) {
        (self.request)()
    }

    /// The trigger's `id`, `aria-haspopup="menu"`, `aria-expanded`, and
    /// `aria-controls` while the menu is open. Spread it on the control inside
    /// the `Menu`:
    ///
    /// ```no_run
    /// # use dioxus::prelude::*;
    /// # use libero::components::{Button, Menu, use_menu};
    /// # fn app() -> Element {
    /// # let menu = use_menu();
    /// # rsx! {
    /// Button { attributes: menu.a11y_attributes(), "Actions" }
    /// # } }
    /// ```
    pub fn a11y_attributes(&self) -> Vec<Attribute> {
        let id = self.id();
        let opened = self.is_open();
        let mut attributes = vec![
            attr("id", trigger_id(&id)),
            attr("aria-haspopup", "menu"),
            attr("aria-expanded", opened.to_string()),
        ];
        // Only while open: it would otherwise name an element that is not in
        // the document.
        if opened {
            attributes.push(attr("aria-controls", menu_id(&id)));
        }
        attributes
    }
}

pub(super) fn trigger_id(id: &str) -> String {
    format!("{id}-trigger")
}

pub(super) fn menu_id(id: &str) -> String {
    format!("{id}-menu")
}
