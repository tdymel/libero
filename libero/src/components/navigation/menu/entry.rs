use dioxus::prelude::*;

/// One line of a [`Menu`](super::Menu), in order.
#[derive(Clone, PartialEq)]
pub enum MenuEntry {
    Item(MenuItem),
    /// A named section: `role="group"`, labelled by its name, so a screen
    /// reader announces "Edit" with the items under it. Inline - not a
    /// submenu.
    Group {
        label: String,
        items: Vec<MenuEntry>,
    },
    /// A rule between two runs of items, `role="separator"`.
    Separator,
}

impl From<MenuItem> for MenuEntry {
    fn from(item: MenuItem) -> Self {
        MenuEntry::Item(item)
    }
}

/// What a menu item does when it is chosen: either run a command or open a
/// submenu, never both. An item that opened a submenu *and* ran a command
/// would run it on every ArrowRight and every hover that opened the submenu.
#[derive(Clone, Default)]
enum Action {
    #[default]
    None,
    Select(Callback<()>),
    Submenu(Vec<MenuEntry>),
}

/// One command in a [`Menu`](super::Menu).
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Kbd, MenuItem};
/// # fn app() -> Element {
/// # fn copy() {}
/// # let _ =
/// MenuItem::new("Copy")
///     .trailing(rsx! { Kbd { "Ctrl C" } })
///     .onselect(move |_| copy())
/// # ;
/// # rsx! {}
/// # }
/// ```
///
/// `leading` and `trailing` land inside the item's `<button>`, so they must not
/// be interactive themselves: a button inside a button is invalid HTML and
/// cannot be reached from the keyboard. An icon or a [`Kbd`](crate::components::Kbd)
/// hint is what they are for.
#[derive(Clone)]
pub struct MenuItem {
    pub(super) label: String,
    action: Action,
    pub(super) leading: Option<Element>,
    pub(super) trailing: Option<Element>,
    pub(super) checked: Option<bool>,
    pub(super) disabled: bool,
}

/// **Never equal**, the way an `Element` is never equal. An item holds a
/// `Callback`, and a `Callback` from the same scope compares equal across
/// renders whatever it captures ([[codebase/dioxus-memoization-traps]]), so a
/// derived `PartialEq` would let a submenu memoize over a command the caller
/// had since replaced.
impl PartialEq for MenuItem {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl MenuItem {
    /// `label` is the accessible name, the visible text, and what typeahead
    /// matches against.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            action: Action::None,
            leading: None,
            trailing: None,
            checked: None,
            disabled: false,
        }
    }

    /// Runs when the item is chosen - clicked, or Enter or Space on it. The
    /// menu then closes unless it was told not to (`close_on_select`).
    ///
    /// Replaces a [`submenu`](Self::submenu): an item does one or the other.
    pub fn onselect(mut self, onselect: impl FnMut(()) + 'static) -> Self {
        self.action = Action::Select(Callback::new(onselect));
        self
    }

    /// Makes this item open a second menu, beside it, rather than run
    /// anything. ArrowRight, Enter, Space, a click, or resting the pointer on
    /// it opens it.
    ///
    /// Replaces an [`onselect`](Self::onselect): an item does one or the
    /// other.
    pub fn submenu(mut self, items: Vec<MenuEntry>) -> Self {
        self.action = Action::Submenu(items);
        self
    }

    /// Drawn before the label - an icon.
    pub fn leading(mut self, leading: Element) -> Self {
        self.leading = Some(leading);
        self
    }

    /// Drawn after the label, at the far end - a shortcut hint.
    pub fn trailing(mut self, trailing: Element) -> Self {
        self.trailing = Some(trailing);
        self
    }

    /// Makes it one choice of several - a `menuitemradio` announcing
    /// `aria-checked` - with a check drawn before the label while `checked`.
    /// Put the choices in one [`MenuEntry::Group`], which is the radio group
    /// a reader hears; keeping exactly one of them checked is the caller's.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// Stays in the arrow-key order - a reader still hears that it exists -
    /// but cannot be chosen, and typeahead skips it.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub(super) fn onselect_callback(&self) -> Option<Callback<()>> {
        match &self.action {
            Action::Select(callback) => Some(*callback),
            _ => None,
        }
    }

    pub(super) fn submenu_items(&self) -> Option<&Vec<MenuEntry>> {
        match &self.action {
            Action::Submenu(items) => Some(items),
            _ => None,
        }
    }
}

/// Every item, in order, through groups - the order the arrow keys walk and
/// the index `data-menu-index` carries.
pub(super) fn flatten(entries: &[MenuEntry]) -> Vec<&MenuItem> {
    fn walk<'a>(entries: &'a [MenuEntry], out: &mut Vec<&'a MenuItem>) {
        for entry in entries {
            match entry {
                MenuEntry::Item(item) => out.push(item),
                MenuEntry::Group { items, .. } => walk(items, out),
                MenuEntry::Separator => {}
            }
        }
    }
    let mut out = Vec::new();
    walk(entries, &mut out);
    out
}
