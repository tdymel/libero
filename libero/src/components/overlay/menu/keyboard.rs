use dioxus::prelude::*;

use super::{hover::HoverDelay, menu::MenuEdge, state::MenuFocus};
use crate::{
    components::common::has_shortcut_modifier,
    hooks::{DismissHandle, ElementHandle, Typeahead, id_selector, typeahead_match},
    platform::{ElementApi, PlatformError, focus_selector, is_tab, logical_key},
};

/// The `Copy` half of one menu's state, shared by every item's handlers.
#[derive(Clone, Copy)]
pub(super) struct Level {
    pub(super) floating: ElementHandle,
    pub(super) wrapper: ElementHandle,
    pub(super) trigger: ElementHandle,
    pub(super) dismiss: DismissHandle,
    /// The level's id, which its rows' ids start with.
    pub(super) id: CopyValue<String>,
    pub(super) close_all: Callback<bool>,
    pub(super) onedge: Option<Callback<MenuEdge>>,
    /// Cancels the pending hover of every level above; `None` on the root.
    pub(super) cancel_above: Option<Callback<()>>,
    pub(super) active: Signal<Option<usize>>,
    pub(super) open_child: Signal<Option<usize>>,
    /// Which submenu to focus into, with a counter so asking twice counts.
    pub(super) child_request: Signal<(u64, usize)>,
    pub(super) len: usize,
    pub(super) depth: usize,
    pub(super) loop_focus: bool,
    pub(super) close_on_select: bool,
}

impl Level {
    /// Moves focus to item `index` - the roving `tabindex` follows `active`.
    pub(super) fn focus(self, index: usize) {
        let mut active = self.active;
        if *active.peek() != Some(index) {
            active.set(Some(index));
        }
        let focused = self
            .floating
            .query_selector(&format!("[data-menu-index=\"{index}\"]"))
            .and_then(|item| item.focus());
        // A WebView queries nothing: the page focuses the row by its id (958).
        if let Err(PlatformError::Unsupported) = focused {
            let row = format!("{}-item-{index}", self.id.peek());
            let _ = focus_selector(&id_selector(&row));
        }
    }

    /// Focuses the menu box itself, with no item active.
    pub(super) fn focus_box(self) {
        if let Err(PlatformError::Unsupported) = self.floating.focus() {
            let _ = focus_selector(&id_selector(&self.id.peek()));
        }
    }

    /// Clicks item `index`: Space on a link, which only Enter activates natively.
    pub(super) fn click(self, index: usize) {
        let _ = self
            .floating
            .query_selector(&format!("[data-menu-index=\"{index}\"]"))
            .and_then(|item| item.click());
    }

    fn step(self, from: usize, forward: bool) {
        let last = self.len - 1;
        let next = match (forward, from) {
            (true, from) if from < last => from + 1,
            (true, _) if self.loop_focus => 0,
            (false, 0) if self.loop_focus => last,
            (false, from) => from.saturating_sub(1),
            (true, _) => last,
        };
        self.focus(next);
    }

    /// Opens the submenu under `index` and asks it to focus its first item.
    fn enter_submenu(self, index: usize) {
        let mut open_child = self.open_child;
        open_child.set(Some(index));
        let mut request = self.child_request;
        let next = request.peek().0.wrapping_add(1);
        request.set((next, index));
    }

    /// Tab leaves via the trigger, so the browser's Tab moves on from there,
    /// not from the portal outlet.
    fn tab_out(self) {
        let inside = self
            .wrapper
            .query_selector("[aria-haspopup=\"menu\"]")
            .and_then(|trigger| trigger.focus());
        // A trigger beside the `Menu` is not in the wrapper: the root's anchor, found by its id.
        if inside.is_err() {
            let _ = self.trigger.focus();
        }
        self.close_all.call(false);
    }

    /// Keys on the box itself, which a click on a group label, a separator or
    /// the padding focuses. An item's own keys bubble here too and pass.
    pub(super) fn box_keydown(self, event: KeyboardEvent) {
        if !self.floating.is_focused() || self.len == 0 || has_shortcut_modifier(&event) {
            return;
        }
        match logical_key(&event) {
            Key::Named(NamedKey::ArrowDown) | Key::Named(NamedKey::Home) => {
                event.prevent_default();
                self.focus(0);
            }
            Key::Named(NamedKey::ArrowUp) | Key::Named(NamedKey::End) => {
                event.prevent_default();
                self.focus(self.len - 1);
            }
            // A tap opens a menubar's menu on its box (4862): the neighbours stay a key away.
            key @ (Key::Named(NamedKey::ArrowLeft) | Key::Named(NamedKey::ArrowRight))
                if self.depth == 0 =>
            {
                if let Some(onedge) = self.onedge {
                    event.prevent_default();
                    onedge.call(match key {
                        Key::Named(NamedKey::ArrowLeft) => MenuEdge::Previous,
                        _ => MenuEdge::Next,
                    });
                }
            }
            _ if is_tab(&event) => self.tab_out(),
            _ => {}
        }
    }

    pub(super) fn choose(
        self,
        index: usize,
        onselect: Option<Callback<()>>,
        submenu: bool,
        close: Option<bool>,
    ) {
        if submenu {
            self.enter_submenu(index);
            return;
        }
        if let Some(onselect) = onselect {
            onselect.call(());
        }
        if close.unwrap_or(self.close_on_select) {
            self.close_all.call(true);
        }
    }

    /// The keys on item `index`. `opens` says it has a submenu that is enabled.
    pub(super) fn item_keydown(
        self,
        event: KeyboardEvent,
        index: usize,
        opens: bool,
        typeahead: &Typeahead,
        labels: &[Option<String>],
        hover: &HoverDelay,
    ) {
        // Keys in a submenu: a row above the pointer rests on must not take focus back (4423).
        if let Some(cancel_above) = self.cancel_above {
            cancel_above.call(());
        }
        // Ctrl/Alt/Meta with a navigation key is the browser's chord.
        let navigation = matches!(
            event.key(),
            Key::Named(NamedKey::ArrowDown)
                | Key::Named(NamedKey::ArrowUp)
                | Key::Named(NamedKey::ArrowLeft)
                | Key::Named(NamedKey::ArrowRight)
                | Key::Named(NamedKey::Home)
                | Key::Named(NamedKey::End)
        );
        if navigation && has_shortcut_modifier(&event) {
            return;
        }
        // The keys take over: a pending hover must not pull focus back (4403).
        if navigation || is_tab(&event) {
            hover.cancel();
        }
        let key = logical_key(&event);
        match key {
            Key::Named(NamedKey::ArrowDown) => {
                event.prevent_default();
                self.step(index, true);
            }
            Key::Named(NamedKey::ArrowUp) => {
                event.prevent_default();
                self.step(index, false);
            }
            // Home on the first or End on the last item scrolls the page.
            Key::Named(NamedKey::Home) if index == 0 => {}
            Key::Named(NamedKey::End) if index + 1 == self.len => {}
            Key::Named(NamedKey::Home) => {
                event.prevent_default();
                self.focus(0);
            }
            Key::Named(NamedKey::End) => {
                event.prevent_default();
                self.focus(self.len - 1);
            }
            Key::Named(NamedKey::ArrowRight) if opens => {
                event.prevent_default();
                self.enter_submenu(index);
            }
            // Closes this submenu only. On the root it is the menubar's.
            Key::Named(NamedKey::ArrowLeft) if self.depth > 0 => {
                event.prevent_default();
                self.dismiss.dismiss();
            }
            // Unanswered here, so they go up.
            Key::Named(NamedKey::ArrowLeft) | Key::Named(NamedKey::ArrowRight)
                if self.onedge.is_some() =>
            {
                event.prevent_default();
                if let Some(onedge) = self.onedge {
                    onedge.call(match key {
                        Key::Named(NamedKey::ArrowLeft) => MenuEdge::Previous,
                        _ => MenuEdge::Next,
                    });
                }
            }
            _ if is_tab(&event) => self.tab_out(),
            Key::Character(ref text) if !has_shortcut_modifier(&event) => {
                let Some(ch) = text.chars().next() else {
                    return;
                };
                // A space mid-query is part of "save as"; otherwise it clicks.
                if ch == ' ' && !typeahead.is_typing() {
                    return;
                }
                let query = typeahead.push(ch);
                let found = typeahead_match(labels.len(), Some(index), &query, |row| {
                    labels[row].as_deref()
                });
                if ch == ' ' || found.is_some() {
                    event.prevent_default();
                }
                if let Some(row) = found {
                    hover.cancel();
                    self.focus(row);
                }
            }
            _ => {}
        }
    }
}

/// Remembers where focus returns to, and focuses the requested item.
/// [`MenuFocus::First`] lands on `first`: the checked item, else the first enabled one.
pub(super) fn use_level_focus(
    level: Level,
    open: bool,
    placed: bool,
    request: u64,
    initial: MenuFocus,
    first: usize,
) {
    let (floating, dismiss, len) = (level.floating, level.dismiss, level.len);
    // Remembered on the opening edge, before focus enters the placed box: the
    // trigger, or a submenu's parent item.
    use_effect(use_reactive!(|(open,)| {
        let (mut active, mut open_child) = (level.active, level.open_child);
        match open {
            true => dismiss.focus_return().remember_focused(),
            // A set notifies even unchanged, which redrew a closing level twice (todo 2095).
            false => {
                if active.peek().is_some() {
                    active.set(None);
                }
                if open_child.peek().is_some() {
                    open_child.set(None);
                }
            }
        }
    }));

    // A removed focused row drops focus to the page, which closes the menu: refocus the row
    // now at its index, else the last (todo 3617).
    use_effect(use_reactive!(|(open, len)| {
        let Some(index) = level.active.peek().filter(|_| open && len > 0) else {
            return;
        };
        if floating.mount_token().is_some() && !dismiss.holds_focus() {
            level.focus(index.min(len - 1));
        }
    }));

    // Once placed: `focus()` on the hidden box answers `Ok` and moves nothing.
    // Only a newer request counts, so a hover-opened submenu steals no focus.
    let mut seen = use_signal(|| if open { 0 } else { request });
    use_effect(use_reactive!(|(open, placed, request, len, first)| {
        // Read first, branch second: this subscribes the effect to a remount.
        let mounted = floating.mount_token().is_some();
        if !open || !placed || !mounted || len == 0 || request <= *seen.peek() {
            return;
        }
        seen.set(request);
        let index = match initial {
            MenuFocus::First => first,
            MenuFocus::Last => len - 1,
            // ArrowDown on the box goes on to the first item.
            MenuFocus::Container => {
                spawn(async move { level.focus_box() });
                return;
            }
        };
        // Out of this dispatch: the click that opened the menu ends by
        // focusing the trigger.
        spawn(async move { level.focus(index) });
    }));
}
