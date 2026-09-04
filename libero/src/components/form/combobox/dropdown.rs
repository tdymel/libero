use dioxus::prelude::*;

use crate::{
    components::{feedback::Loader, layout::ScrollArea},
    sx::sx,
    theme::Size,
};

use super::option::{ComboboxContext, ComboboxRowContext};

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
    active: Option<usize>,
    id: String,
    max_height: String,
    scroll_y: Option<f64>,
    empty: Option<Element>,
    /// The options are being fetched.
    loading: bool,
    /// Above the rows and outside the scroll, so it stays put while the list
    /// under it changes - or empties.
    header: Option<Element>,
    multiselectable: bool,
    /// Re-provided here, not inherited: the dropdown is portaled, so it mounts
    /// under `PortalOutlet` rather than under `ComboboxCore`, and a context
    /// resolves along the mounted chain. Without this every row loses its `id`,
    /// its highlight and its Enter target - silently, because `ComboboxOption`
    /// looks it up with `try_consume_context`.
    context: ComboboxContext,
) -> Element {
    use_context_provider(|| context);

    // No rows and no `empty` draws nothing at all - an empty bordered box is
    // not a state worth showing. A `header` is the exception: it is drawn
    // either way, because a search that matched nothing still needs its box.
    //
    // Loading wins over both. The loader is silent: `ComboboxCore`'s status
    // region, outside this `aria-busy` dropdown, is what says it.
    let list = match (loading, rows.is_empty()) {
        (true, _) => rsx! {
            Loader { size: Size::Sm, sx: sx().align_self("center") }
        },
        (false, true) => empty.unwrap_or_else(|| rsx! {}),
        (false, false) => rsx! {
            ScrollArea {
                sx: sx().max_height(max_height),
                scroll_position_y: scroll_y,
                id: super::aria::listbox_id(&id),
                "role": "listbox",
                "aria-multiselectable": multiselectable.then_some("true"),
                for (index, row) in rows.into_iter().enumerate() {
                    ComboboxRow { key: "{index}", index, active: active == Some(index), {row} }
                }
            }
        },
    };

    rsx! {
        {header}
        {list}
    }
}
