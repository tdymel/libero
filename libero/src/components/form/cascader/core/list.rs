use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::form::ComboboxState,
    hooks::{ElementHandle, Typeahead, use_localization, use_theme},
    theme::Size,
};

use super::super::{
    body::{Body, use_cascader_body},
    keys::CascaderKeys,
    nodes::FlatPath,
    parent_row::{ParentRow, use_parent_row},
    rows::CascaderRows,
};
use super::{
    CascaderLayout, active_descendant, controlled_id,
    engine::{CascaderConfig, Naming},
};

/// What `use_cascader_list` reads from the engine.
pub(super) struct ListSetup<'a> {
    pub(super) config: &'a CascaderConfig,
    pub(super) names: &'a Naming,
    pub(super) state: ComboboxState,
    pub(super) cursor: Signal<Vec<usize>>,
    pub(super) visible: &'a Rc<Vec<FlatPath>>,
    pub(super) layout: CascaderLayout,
    pub(super) typed: Typeahead,
    pub(super) trigger: ElementHandle,
    pub(super) blurred: Signal<bool>,
    pub(super) searchable: bool,
    /// Disabled or read-only: the keys do nothing.
    pub(super) inert: bool,
    pub(super) size: Size,
    pub(super) radius: Size,
}

/// The keyboard, the rows and the drawn body of the open tree, with the ids the trigger points at.
pub(super) struct CascaderList {
    pub(super) keys: Rc<CascaderKeys>,
    pub(super) open: Rc<dyn Fn(bool)>,
    pub(super) body: Element,
    pub(super) controlled_id: String,
    pub(super) descendant: Option<String>,
    /// The cursor's depth, the number of open columns.
    pub(super) depth: usize,
}

pub(super) fn use_cascader_list(setup: ListSetup) -> CascaderList {
    let ListSetup {
        config,
        names,
        state,
        cursor,
        visible,
        layout,
        typed,
        trigger,
        blurred,
        searchable,
        inert,
        size,
        radius,
    } = setup;
    let theme = use_theme();
    let nodes = config.options.clone();
    let any_level = config.any_level;
    let committed = config.committed.clone();

    let cursor_now = cursor.read().clone();
    // The cursor as a row of `visible`, the `Paths` keyboard's index.
    let path_row = visible.iter().position(|path| path.indices == cursor_now);
    let parent_row = use_parent_row();
    let parent_active = parent_row
        .picks(&nodes, any_level, layout, &cursor_now)
        .is_some();

    let id = state.id();
    let listbox_id = format!("{id}-listbox");
    let controlled_id = controlled_id(&listbox_id, layout, &cursor_now);

    // `Rc`: each is shared by several handlers and captures a non-`Copy` path.
    let open = open_handler(cursor, state, committed.clone(), parent_row);
    let commit = commit_handler(
        state,
        config.onpick,
        committed.clone(),
        config.allow_deselect,
    );

    let keys = Rc::new(CascaderKeys {
        nodes: nodes.clone(),
        visible: visible.clone(),
        cursor,
        state,
        open: open.clone(),
        commit: commit.clone(),
        committed: committed.clone(),
        typed,
        disabled: inert,
        searchable,
        any_level,
        layout,
        trigger,
        trigger_id: names.trigger_id.clone(),
        blurred,
        parent_row,
    });

    let rows = CascaderRows {
        nodes,
        node: config.node.clone(),
        separator: config.separator.clone(),
        id: id.clone(),
        cursor,
        cursor_now: cursor_now.clone(),
        committed: committed.unwrap_or_default(),
        commit,
        any_level,
        size,
        radius,
        labels: use_localization().cascader,
        parent_active,
    };
    let body = use_cascader_body(
        rows,
        Body {
            layout,
            depth: cursor_now.len(),
            visible: visible.clone(),
            path_row,
            listbox_id,
            label_id: names.label_id.clone(),
            aria_label: names.aria_label.clone(),
            column_width: config.column_width.clone(),
            max_height: theme.combobox.max_dropdown_height,
        },
    );

    CascaderList {
        keys,
        open,
        body,
        controlled_id,
        descendant: active_descendant(&id, layout, &cursor_now, path_row, parent_active),
        depth: cursor_now.len(),
    }
}

/// Opens on the committed path, like a native `<select>`; with none, nothing is highlighted.
fn open_handler(
    cursor: Signal<Vec<usize>>,
    state: ComboboxState,
    seed: Option<Vec<usize>>,
    parent_row: ParentRow,
) -> Rc<dyn Fn(bool)> {
    Rc::new(move |next: bool| {
        // A local copy lets an `Fn` closure write the signal.
        let mut cursor = cursor;
        if next && !state.is_open() {
            cursor.set(seed.clone().unwrap_or_default());
            parent_row.leave();
        }
        state.set_open(next);
    })
}

/// The one commit path for keyboard and mouse. `allow_deselect` turns a re-pick into a clear;
/// `close` is false only for an `any_level` branch, which is picked and drilled into.
fn commit_handler(
    state: ComboboxState,
    onpick: EventHandler<Option<Vec<usize>>>,
    picked: Option<Vec<usize>>,
    allow_deselect: bool,
) -> Rc<dyn Fn(Vec<usize>, bool)> {
    Rc::new(move |indices: Vec<usize>, close: bool| {
        let next = match allow_deselect && picked.as_ref() == Some(&indices) {
            true => None,
            false => Some(indices),
        };
        onpick.call(next);
        if close {
            state.close();
        }
    })
}
