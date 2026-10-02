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

/// A [`Menu`](super::Menu)'s open state and the id tying the trigger to it.
#[derive(Clone, Copy, PartialEq)]
pub struct MenuState {
    id: Signal<String>,
    opened: Signal<bool>,
    /// Bumped per open, so the same item is focused again on a repeat request.
    request: Signal<(u64, MenuFocus)>,
    /// No items: an open state draws no menu, so the trigger says closed.
    empty: Signal<bool>,
}

/// One `Menu`'s state.
///
/// Docs: <https://libero-ui.dev/overlay/menu>
pub fn use_menu() -> MenuState {
    MenuState {
        id: use_id(),
        opened: use_signal(|| false),
        request: use_signal(|| (0, MenuFocus::First)),
        empty: use_signal(|| false),
    }
}

impl MenuState {
    /// Not a hook, for a runtime number of menus (`Menubar`): `use_menu()` in a
    /// loop would hand hook slots between menus as the `Vec` changes.
    pub(crate) fn new(id: String) -> Self {
        Self {
            id: Signal::new(id),
            opened: Signal::new(false),
            request: Signal::new((0, MenuFocus::First)),
            empty: Signal::new(false),
        }
    }

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
        // Copied out: matching on `peek()` holds the borrow through the write.
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

    pub(super) fn set_empty(&self, empty: bool) {
        let mut flag = self.empty;
        if *flag.peek() != empty {
            flag.set(empty);
        }
    }

    /// The trigger's ARIA wiring. Spread it on the control inside the `Menu`:
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
        let opened = self.is_open() && !(self.empty)();
        let mut attributes = vec![
            attr("id", trigger_id(&id)),
            attr("aria-haspopup", "menu"),
            attr("aria-expanded", opened.to_string()),
        ];
        // Only while open, when the menu is in the document.
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
