use dioxus::prelude::*;

use crate::{localization::MenuLabels, utils::warn};

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
/// `leading` and `trailing` land inside the item's `<button>`, so they must not
/// be interactive themselves: a button inside a button is invalid HTML and
/// cannot be reached from the keyboard. An icon or a badge is what they are
/// for; a shortcut hint is [`shortcut`](Self::shortcut)'s.
#[derive(Clone)]
pub struct MenuItem {
    pub(super) label: String,
    action: Action,
    pub(super) leading: Option<Element>,
    pub(super) trailing: Option<Element>,
    pub(super) check: Option<Check>,
    pub(super) shortcut: Option<String>,
    pub(super) disabled: bool,
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
            check: None,
            shortcut: None,
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

    /// Drawn after the label, at the far end - a badge. It is part of the
    /// accessible name; a shortcut hint belongs in [`shortcut`](Self::shortcut).
    pub fn trailing(mut self, trailing: Element) -> Self {
        self.trailing = Some(trailing);
        self
    }

    /// Makes it one choice of several - a `menuitemradio` announcing
    /// `aria-checked` - with a check drawn before the label while `checked`.
    /// Put the choices in one [`MenuEntry::Group`], which is the radio group
    /// a reader hears; keeping exactly one of them checked is the caller's.
    /// A menu holding a checked item opens with focus on it.
    ///
    /// Replaces a [`checkbox`](Self::checkbox), with a debug warning: an item
    /// is one or the other.
    pub fn radio(self, checked: bool) -> Self {
        self.with_check(Check::Radio(checked))
    }

    /// Makes it an independent on/off setting ("Show ruler") - a
    /// `menuitemcheckbox` announcing `aria-checked` - with a check drawn
    /// before the label while `checked`. Flipping it is the caller's, in
    /// [`onselect`](Self::onselect).
    ///
    /// Replaces a [`radio`](Self::radio), with a debug warning: an item is
    /// one or the other.
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

    /// The key that runs this item outside the menu, in `aria-keyshortcuts`
    /// syntax: `"Control+X"`, `"Control+Shift+S"`. Announced through that
    /// attribute and drawn as a hint at the far end, hidden from readers so
    /// it stays out of the name. Drawn as given, `Control` shortened to
    /// `Ctrl`: no platform mapping (`Meta` is not shown as Cmd). The menu does
    /// not listen for it: binding the key is the caller's.
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

/// The visible hint for an `aria-keyshortcuts` value: its first alternative,
/// with each modifier drawn in `labels`' words - "Control+X Meta+X" draws
/// "Ctrl+X".
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
