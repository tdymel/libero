use dioxus::prelude::*;

use super::{hover::HoverDelay, menu::MenuEdge, state::MenuFocus};
use crate::{
    components::common::has_shortcut_modifier,
    hooks::{DismissHandle, ElementHandle, Typeahead, id_selector, typeahead_match},
    platform::{ElementApi, PlatformError, focus_selector, logical_key},
};

/// The `Copy` half of one menu's state, shared by every item's handlers.
#[derive(Clone, Copy)]
pub(super) struct Level {
    pub(super) floating: ElementHandle,
    pub(super) wrapper: ElementHandle,
    pub(super) dismiss: DismissHandle,
    /// The level's id, which its rows' ids start with.
    pub(super) id: CopyValue<String>,
    pub(super) close_all: Callback<bool>,
    pub(super) onedge: Option<Callback<MenuEdge>>,
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
        let _ = self
            .wrapper
            .query_selector("[aria-haspopup=\"menu\"]")
            .and_then(|trigger| trigger.focus());
        self.close_all.call(false);
    }

    /// Keys on the box itself, which a click on a group label, a separator or
    /// the padding focuses. An item's own keys bubble here too and pass.
    pub(super) fn box_keydown(self, event: KeyboardEvent) {
        if !self.floating.is_focused() || self.len == 0 || has_shortcut_modifier(&event) {
            return;
        }
        match event.key() {
            Key::ArrowDown | Key::Home => {
                event.prevent_default();
                self.focus(0);
            }
            Key::ArrowUp | Key::End => {
                event.prevent_default();
                self.focus(self.len - 1);
            }
            Key::Tab => self.tab_out(),
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
        // Ctrl/Alt/Meta with a navigation key is the browser's chord.
        let navigation = matches!(
            event.key(),
            Key::ArrowDown | Key::ArrowUp | Key::ArrowLeft | Key::ArrowRight | Key::Home | Key::End
        );
        if navigation && has_shortcut_modifier(&event) {
            return;
        }
        let key = logical_key(&event);
        match key {
            Key::ArrowDown => {
                event.prevent_default();
                self.step(index, true);
            }
            Key::ArrowUp => {
                event.prevent_default();
                self.step(index, false);
            }
            Key::Home => {
                event.prevent_default();
                self.focus(0);
            }
            Key::End => {
                event.prevent_default();
                self.focus(self.len - 1);
            }
            Key::ArrowRight if opens => {
                event.prevent_default();
                hover.cancel();
                self.enter_submenu(index);
            }
            // Closes this submenu only. On the root it is the menubar's.
            Key::ArrowLeft if self.depth > 0 => {
                event.prevent_default();
                self.dismiss.dismiss();
            }
            // Unanswered here, so they go up.
            Key::ArrowLeft | Key::ArrowRight if self.onedge.is_some() => {
                event.prevent_default();
                if let Some(onedge) = self.onedge {
                    onedge.call(match key {
                        Key::ArrowLeft => MenuEdge::Previous,
                        _ => MenuEdge::Next,
                    });
                }
            }
            Key::Tab => self.tab_out(),
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
                    self.focus(row);
                }
            }
            _ => {}
        }
    }
}

/// Remembers where focus returns to, and focuses the requested item.
/// [`MenuFocus::First`] lands on `first`: the checked item, if any.
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
        };
        // Out of this dispatch: the click that opened the menu ends by
        // focusing the trigger.
        spawn(async move { level.focus(index) });
    }));
}
