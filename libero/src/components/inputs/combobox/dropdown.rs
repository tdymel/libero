use dioxus::prelude::*;

use crate::{components::layout::ScrollArea, sx::sx};

use super::option::ComboboxRowContext;

/// Publishes where this row sits, so `ComboboxOption` needs no props for its
/// `id` or its highlight. A `Signal`, written during render: a provider runs
/// once, but `active` moves with the arrow keys.
#[component]
fn ComboboxRow(index: usize, active: bool, children: Element) -> Element {
    let mut context = use_context_provider(|| Signal::new(ComboboxRowContext { index, active }));
    let next = ComboboxRowContext { index, active };
    if *context.peek() != next {
        context.set(next);
    }

    children
}

/// The scrolling option list. The rows arrive drawn, which is what erases the
/// `Combobox`'s `T` - and what keeps them from being memoized.
#[component]
pub(super) fn ComboboxDropdown(
    rows: Vec<Element>,
    active: usize,
    id: String,
    max_height: String,
    scroll_y: Option<f64>,
    empty: Option<Element>,
) -> Element {
    if rows.is_empty() {
        return empty.unwrap_or_else(|| rsx! {});
    }

    rsx! {
        ScrollArea {
            sx: sx().max_height(max_height),
            scroll_position_y: scroll_y,
            id: super::aria::listbox_id(&id),
            "role": "listbox",
            for (index, row) in rows.into_iter().enumerate() {
                ComboboxRow { key: "{index}", index, active: index == active, {row} }
            }
        }
    }
}
