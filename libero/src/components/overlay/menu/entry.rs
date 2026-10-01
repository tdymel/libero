use dioxus::prelude::*;

use crate::{localization::MenuLabels, utils::warn};

/// One line of a [`Menu`](super::Menu), in order.
#[derive(Clone, PartialEq)]
pub enum MenuEntry {
    Item(MenuItem),
    /// A named, inline section, `role="group"`. Not a submenu.
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

/// Run a command or open a submenu, never both: the command would run on every
/// hover that opened the submenu. A link is a third, exclusive kind.
#[derive(Clone, Default)]
enum Action {
    #[default]
    None,
    Select(Callback<()>),
    Submenu(Vec<MenuEntry>),
    Href(String),
}

/// One command in a [`Menu`](super::Menu).
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::MenuItem;
/// # fn app() -> Element {
/// # fn copy() {}
/// # let _ =
/// MenuItem::new("Copy")
///     .shortcut("Control+C")
///     .onselect(move |_| copy())
/// # ;
/// # rsx! {}
/// # }
/// ```
///
/// `leading` and `trailing` sit inside a `<button>`: never interactive.
#[derive(Clone)]
pub struct MenuItem {
    pub(super) label: String,
    action: Action,
    pub(super) leading: Option<Element>,
    pub(super) trailing: Option<Element>,
    pub(super) check: Option<Check>,
    pub(super) shortcut: Option<String>,
    pub(super) disabled: bool,
    pub(super) close_on_select: Option<bool>,
    pub(super) new_tab_hint: bool,
}

/// A checkable item's kind and state: one choice of several, or a toggle.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Check {
    Radio(bool),
    Checkbox(bool),
}

impl Check {
    pub(super) fn is_checked(self) -> bool {
        matches!(self, Check::Radio(true) | Check::Checkbox(true))
    }

    pub(super) fn role(self) -> &'static str {
        match self {
            Check::Radio(_) => "menuitemradio",
            Check::Checkbox(_) => "menuitemcheckbox",
        }
    }
}

/// Never equal: a same-scope `Callback` compares equal whatever it captures, so a
/// submenu would memoize over a replaced command ([[codebase/dioxus-memoization-traps]]).
impl PartialEq for MenuItem {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl MenuItem {
    /// `label` is the visible text, the accessible name and the typeahead key.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            action: Action::None,
            leading: None,
            trailing: None,
            check: None,
            shortcut: None,
            disabled: false,
            close_on_select: None,
            new_tab_hint: true,
        }
    }

    /// Runs when the item is chosen. Replaces a [`submenu`](Self::submenu).
    pub fn onselect(mut self, onselect: impl FnMut(()) + 'static) -> Self {
        self.action = Action::Select(Callback::new(onselect));
        self
    }

    /// Opens a second menu beside it. Replaces an [`onselect`](Self::onselect).
    pub fn submenu(mut self, items: Vec<MenuEntry>) -> Self {
        self.action = Action::Submenu(items);
        self
    }

    /// Opens `url` in a new tab: the item is an `<a>` with `role="menuitem"`, so
    /// middle-click and the context menu work. As on `Anchor`, the label ends in a
    /// new-tab icon and a hidden "(opens in a new tab)". Replaces an
    /// [`onselect`](Self::onselect) or [`submenu`](Self::submenu). A disabled link has no `href`.
    ///
    /// ```no_run
    /// # use libero::components::MenuItem;
    /// # let _ =
    /// MenuItem::new("Read the docs").href("https://libero-ui.dev")
    /// # ;
    /// ```
    pub fn href(mut self, url: impl Into<String>) -> Self {
        self.action = Action::Href(url.into());
        self
    }

    /// With [`href`](Self::href): `false` drops the new-tab icon and hidden hint.
    pub fn new_tab_hint(mut self, hint: bool) -> Self {
        self.new_tab_hint = hint;
        self
    }

    /// Drawn before the label, e.g. an icon.
    pub fn leading(mut self, leading: Element) -> Self {
        self.leading = Some(leading);
        self
    }

    /// Drawn at the far end, e.g. a badge. Part of the accessible name.
    pub fn trailing(mut self, trailing: Element) -> Self {
        self.trailing = Some(trailing);
        self
    }

    /// One choice of several (`menuitemradio`); group the choices in a
    /// [`MenuEntry::Group`]. Replaces a [`checkbox`](Self::checkbox).
    pub fn radio(self, checked: bool) -> Self {
        self.with_check(Check::Radio(checked))
    }

    /// An on/off setting (`menuitemcheckbox`); flip it in
    /// [`onselect`](Self::onselect). Replaces a [`radio`](Self::radio).
    pub fn checkbox(self, checked: bool) -> Self {
        self.with_check(Check::Checkbox(checked))
    }

    fn with_check(mut self, check: Check) -> Self {
        if cfg!(debug_assertions)
            && let Some(before) = self.check
            && before.role() != check.role()
        {
            warn(&format!(
                "MenuItem \"{}\": both `radio` and `checkbox` were called; the later one wins.",
                self.label
            ));
        }
        self.check = Some(check);
        self
    }

    /// A shortcut hint in `aria-keyshortcuts` syntax. The caller binds the key;
    /// the menu does not listen for it.
    ///
    /// ```no_run
    /// # use libero::components::MenuItem;
    /// # fn cut() {}
    /// # let _ =
    /// MenuItem::new("Cut").shortcut("Control+X").onselect(move |_| cut())
    /// # ;
    /// ```
    pub fn shortcut(mut self, keys: &str) -> Self {
        self.shortcut = Some(keys.to_string());
        self
    }

    /// Stays in the arrow-key order but cannot be chosen; typeahead skips it.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Choosing this item leaves the menu open, whatever the menu's
    /// `close_on_select` says. Focus stays on the item.
    pub fn keep_open(self) -> Self {
        self.close_on_select(false)
    }

    /// Overrides the menu's `close_on_select` for this item, either way.
    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = Some(close);
        self
    }

    pub(super) fn onselect_callback(&self) -> Option<Callback<()>> {
        match &self.action {
            Action::Select(callback) => Some(*callback),
            _ => None,
        }
    }

    pub(super) fn href_url(&self) -> Option<&str> {
        match &self.action {
            Action::Href(url) => Some(url),
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

/// The first alternative, modifiers in `labels`' words: "Control+X Meta+X" is "Ctrl+X".
pub(super) fn shortcut_hint(keys: &str, labels: &MenuLabels) -> String {
    let first = keys.split_whitespace().next().unwrap_or_default();
    first
        .split('+')
        .map(|key| match key {
            "Control" => labels.control,
            "Shift" => labels.shift,
            "Alt" => labels.alt,
            "Meta" => labels.meta,
            key => key,
        })
        .collect::<Vec<_>>()
        .join("+")
}

/// Every item through groups, in arrow-key and `data-menu-index` order.
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

#[cfg(test)]
mod tests {
    use super::{MenuItem, MenuLabels, shortcut_hint};

    #[test]
    fn radio_and_checkbox_on_one_item_warn() {
        crate::utils::take_warnings();
        let _ = MenuItem::new("Ruler").radio(true).checkbox(false);
        let warnings = crate::utils::take_warnings();
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("Ruler"), "{warnings:?}");
        let _ = MenuItem::new("Ruler").checkbox(true).checkbox(false);
        assert!(crate::utils::take_warnings().is_empty());
    }

    #[test]
    fn the_hint_shortens_control_and_keeps_the_first_alternative() {
        let en = &MenuLabels::ENGLISH;
        assert_eq!(shortcut_hint("Control+Shift+S", en), "Ctrl+Shift+S");
        assert_eq!(shortcut_hint("Control+X Meta+X", en), "Ctrl+X");
        assert_eq!(shortcut_hint("Control+C", &MenuLabels::GERMAN), "Strg+C");
        assert_eq!(shortcut_hint("F2", en), "F2");
    }

    /// Todo 725: every modifier takes the localization's word.
    #[test]
    fn the_hint_names_every_modifier_in_the_language() {
        let de = &MenuLabels::GERMAN;
        assert_eq!(shortcut_hint("Control+Shift+S", de), "Strg+Umschalt+S");
        assert_eq!(shortcut_hint("Alt+Meta+F4", de), "Alt+Meta+F4");
        let custom = MenuLabels {
            alt: "Option",
            meta: "Cmd",
            ..MenuLabels::ENGLISH
        };
        assert_eq!(shortcut_hint("Alt+Meta+K", &custom), "Option+Cmd+K");
    }
}
