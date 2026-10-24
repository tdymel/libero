use dioxus::prelude::*;

use crate::{
    localization::MenuLabels,
    sx::{ColorRole, ThemeAwareValue},
    utils::warn,
};

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
/// `leading` and `trailing` sit inside the row's `<button>` (an `<a>` for an `href` item): never interactive.
#[derive(Clone)]
pub struct MenuItem {
    pub(super) label: String,
    action: Action,
    pub(super) leading: Option<Element>,
    pub(super) trailing: Option<Element>,
    pub(super) check: Option<Check>,
    pub(super) shortcut: Option<String>,
    pub(super) disabled: bool,
    pub(super) description: Option<String>,
    pub(super) close_on_select: Option<bool>,
    pub(super) new_tab_hint: bool,
    pub(super) new_tab_icon: Option<bool>,
    pub(super) color: Option<ThemeAwareValue>,
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
            description: None,
            close_on_select: None,
            new_tab_hint: true,
            new_tab_icon: None,
            color: None,
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

    /// Opens `url` in a new tab as an `<a role="menuitem">`, with `Anchor`'s hidden new-tab
    /// hint and [`new_tab_icon`](Self::new_tab_icon). Replaces [`onselect`](Self::onselect) or
    /// [`submenu`](Self::submenu). Disabled: an `<a>` without `href`. A `javascript:`/`data:`/`vbscript:` URL: a plain button.
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

    /// With [`href`](Self::href): `false` drops the hidden hint and the new-tab icon.
    pub fn new_tab_hint(mut self, hint: bool) -> Self {
        self.new_tab_hint = hint;
        self
    }

    /// With [`href`](Self::href): draws the new-tab icon. Unset, the theme's `anchor.new_tab_icon`, off.
    pub fn new_tab_icon(mut self, icon: bool) -> Self {
        self.new_tab_icon = Some(icon);
        self
    }

    /// Drawn before the label, e.g. an icon.
    pub fn leading(mut self, leading: Element) -> Self {
        self.leading = Some(leading);
        self
    }

    /// Colours the label and [`leading`](Self::leading), e.g. `"error"` for a destructive
    /// command. A palette colour takes its readable text shade.
    ///
    /// ```no_run
    /// # use libero::components::MenuItem;
    /// # let _ =
    /// MenuItem::new("Delete").color("error")
    /// # ;
    /// ```
    pub fn color(mut self, color: impl Into<ThemeAwareValue>) -> Self {
        self.color = Some(color.into());
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

    /// Read after the label, never shown: e.g. why a [`disabled`](Self::disabled)
    /// item cannot be chosen.
    ///
    /// ```no_run
    /// # use libero::components::MenuItem;
    /// # let _ =
    /// MenuItem::new("Paste").disabled(true).description("The clipboard is empty")
    /// # ;
    /// ```
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
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

    /// Draws the same row: everything but the callbacks, which a row reads when it runs.
    pub(super) fn draws_like(&self, other: &Self) -> bool {
        self.disabled == other.disabled
            && self.check == other.check
            && self.new_tab_hint == other.new_tab_hint
            && self.new_tab_icon == other.new_tab_icon
            && self.submenu_items().is_some() == other.submenu_items().is_some()
            && self.label == other.label
            && self.href_url() == other.href_url()
            && self.shortcut == other.shortcut
            && self.description == other.description
            && self.leading == other.leading
            && self.trailing == other.trailing
            && self.color == other.color
    }

    /// The `color` as CSS, in the text role.
    pub(super) fn color_css(&self) -> Option<String> {
        self.color
            .as_ref()
            .and_then(|color| color.in_color_role(ColorRole::Text).resolve(None))
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

/// The row an opening menu focuses: the chosen radio, else the first enabled
/// item, as React Aria never lands on a disabled one.
pub(super) fn first_focus(flat: &[&MenuItem]) -> usize {
    // Only a chosen radio: a toggled setting is no "choice in effect".
    flat.iter()
        .position(|item| item.check == Some(Check::Radio(true)))
        .or_else(|| flat.iter().position(|item| !item.disabled))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{MenuItem, MenuLabels, first_focus, shortcut_hint};

    #[test]
    fn an_opening_menu_skips_disabled_items_but_not_a_chosen_radio() {
        let items = [
            MenuItem::new("To do").disabled(true),
            MenuItem::new("Doing"),
            MenuItem::new("Done").radio(true).disabled(true),
        ];
        assert_eq!(first_focus(&items.iter().collect::<Vec<_>>()), 2);
        assert_eq!(first_focus(&items[..2].iter().collect::<Vec<_>>()), 1);
        assert_eq!(first_focus(&items[..1].iter().collect::<Vec<_>>()), 0);
    }

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
