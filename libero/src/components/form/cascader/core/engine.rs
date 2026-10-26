use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        accessibility::VisuallyHidden,
        common::{ClassList, Input, Parts},
        form::{
            ComboboxState, DropdownPart, TouchHold, combobox::nothing_found_row,
            phone_sheet::SheetSwipe, use_refocus_on_close, use_touch_hold,
        },
        layout::BoxStyle,
    },
    hooks::{
        ElementHandle, TYPEAHEAD_RESET, use_element, use_localization, use_theme, use_typeahead,
    },
    sx::Sx,
    theme::Size,
};

use super::super::{
    dropdown::{Dropdown, DropdownSetup, dropdown_box, use_cascader_dropdown},
    keys::CascaderKeys,
    option::CascaderNode,
    search::{CascaderSearch, search_header},
};
use super::{
    CascaderLayout, CascaderMatch, CascaderRender,
    list::{CascaderList, ListSetup, use_cascader_list},
    open_paths,
};

/// The `T`-free tree and its settings, shared by `Cascader`'s field and `CascaderDropdown`.
pub(in super::super) struct CascaderConfig {
    pub(in super::super) options: Rc<Vec<CascaderNode>>,
    pub(in super::super) committed: Option<Vec<usize>>,
    /// The option to commit next, by index path; `None` clears.
    pub(in super::super) onpick: EventHandler<Option<Vec<usize>>>,
    /// `None` draws the label.
    pub(in super::super) node: Option<CascaderRender>,
    /// `None` is a case-insensitive `contains` over the joined labels.
    pub(in super::super) filter: Option<CascaderMatch>,
    pub(in super::super) separator: String,
    pub(in super::super) any_level: bool,
    pub(in super::super) allow_deselect: bool,
    pub(in super::super) layout: CascaderLayout,
    pub(in super::super) column_width: String,
    pub(in super::super) search_placeholder: Option<String>,
}

/// What names and describes the trigger, the open search box and the listboxes.
pub(in super::super) struct Naming {
    pub(in super::super) trigger_id: String,
    pub(in super::super) label_id: Option<String>,
    /// Names the search box and the listboxes when there is no `label_id`.
    pub(in super::super) aria_label: Option<String>,
    pub(in super::super) describedby: Option<String>,
    pub(in super::super) invalid: bool,
    pub(in super::super) required: bool,
}

pub(in super::super) struct EngineSetup<'a> {
    pub(in super::super) config: &'a CascaderConfig,
    pub(in super::super) names: Naming,
    pub(in super::super) state: ComboboxState,
    pub(in super::super) trigger: ElementHandle,
    pub(in super::super) anchor: ElementHandle,
    /// The field's root, so the bottom sheet clears its helper and error too.
    pub(in super::super) field: Option<ElementHandle>,
    pub(in super::super) opened: bool,
    pub(in super::super) searchable: bool,
    /// Disabled or read-only: the keys do nothing.
    pub(in super::super) inert: bool,
    pub(in super::super) size: Size,
    pub(in super::super) radius: Size,
    pub(in super::super) parts: &'a Input<Parts<DropdownPart>>,
    /// The dropdown box's own `class`, `sx` and attributes, from `CascaderDropdown`.
    pub(in super::super) class: Option<&'a Input<ClassList>>,
    pub(in super::super) sx: Option<&'a Input<Sx>>,
    pub(in super::super) attributes: Vec<Attribute>,
}

/// What the trigger needs from the open tree.
pub(in super::super) struct Engine {
    pub(in super::super) keys: Rc<CascaderKeys>,
    pub(in super::super) open: Rc<dyn Fn(bool)>,
    pub(in super::super) controlled_id: String,
    pub(in super::super) descendant: Option<String>,
    /// No row to list: open, the trigger must not claim an expanded popup.
    pub(in super::super) empty: bool,
    /// Carries the bottom sheet's clearance; the field wraps its frame in it.
    pub(in super::super) wrapper: BoxStyle,
    /// The sheet's clearance in px, for a trigger the engine does not wrap.
    pub(in super::super) clearance: f64,
    /// The always-mounted region saying what a search left.
    pub(in super::super) status: Element,
    /// A touch open: the trigger keeps focus until the search box's own tap (todo 3899).
    pub(in super::super) hold: TouchHold,
    /// Held now, so the trigger is still the combobox.
    pub(in super::super) held: bool,
    /// [`use_refocus_on_close`]'s blur mark, set by a held trigger's blur.
    pub(in super::super) blurred: Signal<bool>,
}

/// The keyboard, the rows, the search box and the portaled dropdown, for any trigger.
pub(in super::super) fn use_cascader_engine(setup: EngineSetup) -> Engine {
    let EngineSetup {
        config,
        names,
        state,
        trigger,
        anchor,
        field,
        opened,
        searchable,
        inert,
        size,
        radius,
        parts,
        class,
        sx,
        attributes,
    } = setup;
    let theme = use_theme();
    let words = use_localization();
    let nothing_found = words.combobox.nothing_found;

    // One index per level. `[2, 0]` highlights the third root's first child without expanding it.
    let cursor = use_signal(Vec::<usize>::new);
    let query = use_signal(String::new);
    let search = use_element();
    let typed = use_typeahead(TYPEAHEAD_RESET);

    // Only spaces is no query: it would show every path as results.
    let searching = searchable && opened && !query.read().trim().is_empty();
    let layout = match searching {
        true => CascaderLayout::Paths,
        false => config.layout,
    };

    // The `Paths` rows the query leaves. `Columns` ignores it.
    let visible = open_paths(
        opened,
        config,
        searching.then(|| query.read().clone()).as_deref(),
    );
    let rows = visible.clone().unwrap_or_default();

    let blurred = use_refocus_on_close(opened, searchable, trigger, query);
    let hold = use_touch_hold();
    let held = searchable && opened && hold.held();

    let CascaderList {
        keys,
        open,
        body,
        controlled_id,
        descendant,
        depth,
    } = use_cascader_list(ListSetup {
        config,
        names: &names,
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
    });

    let Dropdown {
        popover,
        search_box,
        wrapper,
        dropdown,
        clearance,
        swipe: SheetSwipe {
            strip, listeners, ..
        },
    } = use_cascader_dropdown(DropdownSetup {
        state,
        anchor,
        field,
        opened,
        // Focused on open, unless a touch held the trigger.
        searchable: searchable && !held,
        search,
        size,
        radius,
        layout,
        // A new column or a shorter match list reshapes the open box.
        remeasure: (depth * 4096 + rows.len()) as u64,
        gap: theme.popover.gap,
        padding: theme.popover.padding,
        parts,
        class,
        sx,
    });

    // Drawn in the list's place and said by the status region below.
    let nothing_found = (searching && rows.is_empty()).then_some(nothing_found);
    // What a query left, said but not shown (WCAG 4.1.3, todo 2035).
    let results = (searching && !rows.is_empty()).then(|| (words.combobox.results)(rows.len()));
    let body = match nothing_found {
        Some(text) => nothing_found_row(text),
        None => body,
    };

    let header = searchable.then(|| {
        let search = CascaderSearch {
            element: search,
            query,
            cursor,
            state,
            blurred,
            hold,
            combobox: !held,
        };
        // No listbox while nothing matches; a dangling id is invalid (todo 2291).
        let controls = nothing_found.is_none().then(|| controlled_id.clone());
        let placeholder = config.search_placeholder.clone().unwrap_or_default();
        let name = names
            .aria_label
            .clone()
            .filter(|_| names.label_id.is_none());
        search_header(
            search_box.attr("aria-label", name),
            search,
            controls,
            descendant.clone(),
            placeholder,
            &names,
        )
    });

    let mut attributes = attributes;
    attributes.extend(listeners);
    let dropdown_keys = keys.clone();
    popover.show(opened.then(|| {
        dropdown_box(
            dropdown,
            popover.floating(),
            dropdown_keys,
            attributes,
            rsx! {
                {strip}
                {header}
                {body}
            },
        )
    }));

    let status = rsx! {
        // Always mounted: a region inserted with its text is not announced.
        VisuallyHidden { role: "status",
            if let Some(text) = nothing_found {
                "{text}"
            }
            if let Some(text) = results {
                "{text}"
            }
        }
    };

    Engine {
        keys,
        open,
        controlled_id,
        descendant,
        // A tree without roots has no path; closed, the walk is skipped.
        empty: rows.is_empty() && (opened || config.options.is_empty()),
        wrapper,
        clearance,
        status,
        hold,
        held,
        blurred,
    }
}
