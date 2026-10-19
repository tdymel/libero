use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        common::{NavigationChord, has_shortcut_modifier, navigation_chord},
        form::ComboboxState,
    },
    hooks::{ElementHandle, Typeahead, typeahead_match},
    platform::{ElementApi, focus_tab_from, is_tab, logical_key},
};

use super::{
    core::CascaderLayout,
    nodes::{FlatPath, children_at, disabled_at, first_enabled, last_enabled, step, step_in},
    option::CascaderNode,
    parent_row::ParentRow,
};

/// The keyboard, by layout: `Columns` walks the tree, `Paths` a flat list (Left/Right stay the caret's).
/// In an `Rc`: the trigger and the portaled dropdown share it.
pub(super) struct CascaderKeys {
    pub(super) nodes: Rc<Vec<CascaderNode>>,
    pub(super) visible: Rc<Vec<FlatPath>>,
    pub(super) cursor: Signal<Vec<usize>>,
    pub(super) state: ComboboxState,
    pub(super) open: Rc<dyn Fn(bool)>,
    pub(super) commit: Rc<dyn Fn(Vec<usize>, bool)>,
    pub(super) committed: Option<Vec<usize>>,
    pub(super) typed: Typeahead,
    pub(super) disabled: bool,
    pub(super) searchable: bool,
    pub(super) any_level: bool,
    pub(super) layout: CascaderLayout,
    pub(super) trigger: ElementHandle,
    /// The trigger's `id` while the search box is open.
    pub(super) trigger_id: String,
    /// `use_refocus_on_close`'s blur mark.
    pub(super) blurred: Signal<bool>,
    pub(super) parent_row: ParentRow,
}

impl CascaderKeys {
    /// The path a pick takes while the highlight is on the narrow list's parent row.
    fn on_parent(&self) -> Option<Vec<usize>> {
        let cursor = self.cursor.read();
        self.parent_row
            .picks(&self.nodes, self.any_level, self.layout, &cursor)
    }

    pub(super) fn handle(&self, event: KeyboardEvent) {
        // A composing key picks an IME candidate in the search box (todo 3033).
        if self.disabled || event.is_composing() {
            return;
        }
        // APG: Alt+ArrowDown opens in place, Alt+ArrowUp closes; other chords aren't ours.
        match navigation_chord(&event) {
            Some(NavigationChord::Open) => {
                event.prevent_default();
                if !self.state.is_open() {
                    (self.open)(true);
                }
                return;
            }
            Some(NavigationChord::Close) if self.state.is_open() => {
                event.prevent_default();
                self.leave();
                return;
            }
            Some(_) => return,
            None => {}
        }
        if self.typeahead(&event) {
            return;
        }
        if !self.state.is_open() {
            self.closed(event);
            return;
        }
        let key = match is_tab(&event) {
            true => Key::Named(NamedKey::Tab),
            false => event.key(),
        };
        let paths_layout = self.layout == CascaderLayout::Paths;
        match key {
            Key::Named(NamedKey::Escape) => {
                event.prevent_default();
                self.state.close();
            }
            // The search box is portaled after the page: Tab moves on from the trigger (todo 2289).
            Key::Named(NamedKey::Tab) => {
                // Marked blurred: Blitz moves focus without a blur, and the close would refocus the trigger (todo 2354).
                if self.searchable {
                    // A WebView's `focus()` lands after Tab's own move: the page tabs on for it.
                    let from = format!("[id=\"{}\"]", self.trigger_id);
                    match focus_tab_from(&from, event.modifiers().contains(Modifiers::SHIFT)) {
                        Ok(()) => event.prevent_default(),
                        Err(_) => {
                            let _ = self.trigger.focus();
                        }
                    }
                    let mut blurred = self.blurred;
                    blurred.set(true);
                }
                self.leave();
            }
            // APG select-only: Space is Enter, except in the search box.
            Key::Character(ref character) if character == " " && !self.searchable => {
                event.prevent_default();
                self.activate();
            }
            Key::Named(NamedKey::ArrowDown)
            | Key::Named(NamedKey::ArrowUp)
            | Key::Named(NamedKey::Home)
            | Key::Named(NamedKey::End)
            | Key::Named(NamedKey::Enter)
                if paths_layout =>
            {
                self.paths(event)
            }
            _ => self.columns(event),
        }
    }

    /// APG select-only: Tab and Alt+ArrowUp keep a pickable highlight, then
    /// close. Re-picking the committed path would clear it under `allow_deselect`.
    fn leave(&self) {
        let here = self
            .on_parent()
            .unwrap_or_else(|| self.cursor.read().clone());
        let pickable = match self.layout {
            CascaderLayout::Paths => self
                .visible
                .iter()
                .any(|path| path.indices == here && !path.disabled),
            CascaderLayout::Columns => {
                !here.is_empty()
                    && !disabled_at(&self.nodes, &here)
                    && (self.any_level || children_at(&self.nodes, &here).is_empty())
            }
        };
        if pickable && self.committed.as_ref() != Some(&here) {
            (self.commit)(here, true);
        }
        self.state.close();
    }

    /// Enter, or Space on a list with no search box, on the highlight.
    fn activate(&self) -> bool {
        let here = self.cursor.read().clone();
        match self.layout {
            CascaderLayout::Paths => {
                // Nothing highlighted: Enter bubbles, so a form still submits.
                let Some(path) = self.visible.iter().find(|path| path.indices == here) else {
                    return false;
                };
                if !path.disabled {
                    (self.commit)(path.indices.clone(), true);
                }
                true
            }
            CascaderLayout::Columns => {
                if let Some(parent) = self.on_parent() {
                    (self.commit)(parent, true);
                    return true;
                }
                if here.is_empty() || disabled_at(&self.nodes, &here) {
                    return false;
                }
                let children = children_at(&self.nodes, &here);
                if children.is_empty() {
                    (self.commit)(here, true);
                    return true;
                }
                // A branch expands; with `any_level` it is also picked, and the list stays open.
                // The committed one only expands: navigation never deselects.
                if self.any_level && self.committed.as_ref() != Some(&here) {
                    (self.commit)(here.clone(), false);
                }
                if let Some(index) = first_enabled(children) {
                    let mut cursor = self.cursor;
                    let mut next = here;
                    next.push(index);
                    cursor.set(next);
                }
                true
            }
        }
    }

    /// A closed list: the keys that open it.
    fn closed(&self, event: KeyboardEvent) {
        let mut cursor = self.cursor;
        let open = &self.open;
        let key = logical_key(&event);
        let forward = match key {
            Key::Named(NamedKey::ArrowDown)
            | Key::Named(NamedKey::ArrowRight)
            | Key::Named(NamedKey::Enter)
            | Key::Named(NamedKey::Home) => true,
            Key::Named(NamedKey::ArrowUp) | Key::Named(NamedKey::End) => false,
            Key::Character(ref character) if character == " " => true,
            _ => return,
        };
        event.prevent_default();
        open(true);
        // APG select-only: Home/End open on the first/last row; other keys keep the committed path.
        if (matches!(key, Key::Named(NamedKey::Home) | Key::Named(NamedKey::End))
            || cursor.read().is_empty())
            && let Some(next) = self.edge(forward)
        {
            cursor.set(next);
        }
    }

    /// The first or last enabled row of what opens: a root, or a `Paths` row.
    fn edge(&self, forward: bool) -> Option<Vec<usize>> {
        match self.layout {
            CascaderLayout::Columns => match forward {
                true => first_enabled(&self.nodes),
                false => last_enabled(&self.nodes),
            }
            .map(|index| vec![index]),
            CascaderLayout::Paths => step(
                self.visible.len(),
                |row| self.visible[row].disabled,
                None,
                forward,
            )
            .map(|row| self.visible[row].indices.clone()),
        }
    }

    /// APG typeahead within the cursor's column; opens a closed list on the roots. Off while `searchable`.
    fn typeahead(&self, event: &KeyboardEvent) -> bool {
        if self.searchable || has_shortcut_modifier(event) {
            return false;
        }
        let Key::Character(ref key) = event.key() else {
            return false;
        };
        let Some(ch) = key.chars().next() else {
            return false;
        };
        // A space mid-query is part of "new york", otherwise an activation.
        if ch == ' ' && !self.typed.is_typing() {
            return false;
        }
        let open = self.state.is_open();
        let query = self.typed.push(ch);
        let here = match open {
            true => self.cursor.read().clone(),
            false => Vec::new(),
        };
        let found = match self.layout {
            CascaderLayout::Paths => {
                let labels: Vec<String> = self
                    .visible
                    .iter()
                    .map(|path| path.labels.join(" "))
                    .collect();
                let current = self.visible.iter().position(|path| path.indices == here);
                typeahead_match(labels.len(), current, &query, |row| {
                    (!self.visible[row].disabled).then(|| labels[row].as_str())
                })
                .map(|row| self.visible[row].indices.clone())
            }
            CascaderLayout::Columns => {
                let (parents, current) = match here.split_last() {
                    Some((last, parents)) => (parents.to_vec(), Some(*last)),
                    None => (Vec::new(), None),
                };
                let column = children_at(&self.nodes, &parents);
                typeahead_match(column.len(), current, &query, |index| {
                    let mut path = parents.clone();
                    path.push(index);
                    (!disabled_at(&self.nodes, &path)).then(|| column[index].label.as_str())
                })
                .map(|index| {
                    let mut next = parents.clone();
                    next.push(index);
                    next
                })
            }
        };
        let Some(next) = found else {
            return false;
        };
        event.prevent_default();
        if !open {
            (self.open)(true);
        }
        self.parent_row.leave();
        let mut cursor = self.cursor;
        cursor.set(next);
        true
    }

    /// The flat list: Up, Down, Home, End and Enter.
    fn paths(&self, event: KeyboardEvent) {
        let (mut cursor, visible) = (self.cursor, &self.visible);
        let here = cursor.read().clone();
        let row = visible.iter().position(|path| path.indices == here);
        let key = event.key();
        if key == Key::Named(NamedKey::Enter) {
            if self.activate() {
                event.prevent_default();
            }
            return;
        }
        event.prevent_default();
        let from = match key {
            Key::Named(NamedKey::Home) | Key::Named(NamedKey::End) => None,
            _ => row,
        };
        let forward = matches!(
            key,
            Key::Named(NamedKey::ArrowDown) | Key::Named(NamedKey::Home)
        );
        if let Some(index) = step(
            visible.len(),
            |index| visible[index].disabled,
            from,
            forward,
        ) {
            cursor.set(visible[index].indices.clone());
        }
    }

    /// The tree. Under `Paths`, Left and Right stay the search box's caret keys.
    fn columns(&self, event: KeyboardEvent) {
        let (mut cursor, nodes) = (self.cursor, &self.nodes);
        let paths_layout = self.layout == CascaderLayout::Paths;
        let here = cursor.read().clone();
        let key = logical_key(&event);
        match key {
            Key::Named(NamedKey::ArrowDown)
            | Key::Named(NamedKey::ArrowUp)
            | Key::Named(NamedKey::Home)
            | Key::Named(NamedKey::End) => {
                event.prevent_default();
                // The parent row is the column's first: Down goes back to the cursor's row.
                if self.on_parent().is_some()
                    && matches!(
                        key,
                        Key::Named(NamedKey::ArrowUp) | Key::Named(NamedKey::Home)
                    )
                {
                    return;
                }
                if self.on_parent().is_some() && key == Key::Named(NamedKey::ArrowDown) {
                    self.parent_row.leave();
                    return;
                }
                let (parents, from) = match here.split_last() {
                    Some((last, parents)) => (parents.to_vec(), Some(*last)),
                    None => (Vec::new(), None),
                };
                let from = match key {
                    Key::Named(NamedKey::Home) | Key::Named(NamedKey::End) => None,
                    _ => from,
                };
                let forward = matches!(
                    key,
                    Key::Named(NamedKey::ArrowDown) | Key::Named(NamedKey::Home)
                );
                match step_in(nodes, &parents, from, forward) {
                    Some(index) => {
                        let mut next = parents;
                        next.push(index);
                        self.parent_row.leave();
                        cursor.set(next.clone());
                        if key == Key::Named(NamedKey::Home) {
                            self.parent_row.reach(nodes, self.any_level, &next);
                        }
                    }
                    // Up from the first row.
                    None if key == Key::Named(NamedKey::ArrowUp) => {
                        self.parent_row.reach(nodes, self.any_level, &here)
                    }
                    None => {}
                }
            }
            Key::Named(NamedKey::ArrowRight) if !paths_layout && self.on_parent().is_none() => {
                event.prevent_default();
                let column = children_at(nodes, &here);
                // A disabled branch does not open, as a click on it does not.
                if !here.is_empty()
                    && !disabled_at(nodes, &here)
                    && let Some(index) = first_enabled(column)
                {
                    let mut next = here;
                    next.push(index);
                    cursor.set(next);
                }
            }
            Key::Named(NamedKey::ArrowLeft) if !paths_layout => {
                event.prevent_default();
                if here.len() > 1 {
                    let mut next = here;
                    next.pop();
                    cursor.set(next);
                }
            }
            // Unhandled, Enter bubbles, so a form still submits.
            Key::Named(NamedKey::Enter) if self.activate() => event.prevent_default(),
            _ => {}
        }
    }
}
