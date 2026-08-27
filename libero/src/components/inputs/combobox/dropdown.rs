use std::collections::HashSet;

use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, OptionLabel, States,
        layout::{ScrollArea, Virtualize, use_box},
    },
    sx::{StaticSx, sx},
    theme::{ComboboxDefaults, Size},
};

static COMBOBOX_ROW_SX: StaticSx = StaticSx::new(|| {
    ComboboxDefaults::row_theme_vars()
        .display("flex")
        .align_items("center")
        .gap("8px")
        .width("100%")
        .cursor("pointer")
        .user_select("none")
        .white_space("nowrap")
        .overflow("hidden")
        .text_overflow("ellipsis")
        .border_radius("4px")
        .hover(sx().background("grey.1"))
        .when("active", sx().background("grey.2"))
        .when("selected", sx().background("primary.1").color("primary.7"))
});

/// One row. A component, not a closure, because it needs `use_box`.
#[component]
fn ComboboxOption(
    id: String,
    label: OptionLabel,
    active: bool,
    selected: bool,
    size: Size,
    onpick: EventHandler<()>,
) -> Element {
    let states: Input<States> = States::new()
        .with(size.state_name(), true)
        .with("active", active)
        .with("selected", selected)
        .into();

    use_box()
        .framework_sx(&COMBOBOX_ROW_SX)
        .states(&states)
        .prepare()
        .attr("id", id)
        .attr("role", "option")
        .attr("aria-selected", selected)
        // Or the click blurs the search field first, closing the dropdown
        // before the row ever hears about it.
        .event("onmousedown", move |event: MouseEvent| {
            event.prevent_default();
        })
        .event("onclick", move |_: MouseEvent| onpick.call(()))
        .render(HtmlTag::Div, Vec::new(), label.render())
}

/// The scrolling option list. `visible` holds the indices that passed the
/// filter, so a row's position and its option's index are different numbers.
#[component]
pub(super) fn ComboboxDropdown(
    labels: Vec<OptionLabel>,
    visible: Vec<usize>,
    selected: HashSet<usize>,
    active: usize,
    id: String,
    size: Size,
    item_size: f64,
    max_height: String,
    scroll_y: Option<f64>,
    onpick: EventHandler<usize>,
    empty: Option<Element>,
) -> Element {
    let count = visible.len();
    let row_id = use_callback(move |index: usize| format!("{id}-option-{index}"));
    let item = use_callback(move |row: usize| {
        let Some(&index) = visible.get(row) else {
            return rsx! {};
        };
        let Some(label) = labels.get(index).cloned() else {
            return rsx! {};
        };
        rsx! {
            ComboboxOption {
                id: row_id.call(index),
                label,
                active: row == active,
                selected: selected.contains(&index),
                size,
                onpick: move |()| onpick.call(index),
            }
        }
    });

    // After the hooks, never before them.
    if count == 0 {
        return empty.unwrap_or_else(|| rsx! {});
    }

    rsx! {
        ScrollArea {
            sx: sx().max_height(max_height),
            scroll_position_y: scroll_y,
            "role": "listbox",
            Virtualize { count, item, item_size: Some(item_size) }
        }
    }
}
