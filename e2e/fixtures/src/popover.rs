//! `use_popover` with `PopoverOptions::dismiss` (todo 522).

use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Flex, Text},
    hooks::{PopoverOptions, use_element, use_popover},
    sx::sx,
    use_theme,
};

use crate::Routes;

pub const ROUTES: Routes = &[("/popover", || rsx! { PopoverPage {} })];

/// A click-opened box with a control in it, and plain text beside the
/// trigger for a press that lands on nothing focusable.
#[component]
fn PopoverPage() -> Element {
    let theme = use_theme();
    let mut opened = use_signal(|| false);
    let anchor = use_element();
    let options = PopoverOptions::new(theme.popover.gap, theme.popover.padding).dismiss(true);
    let popover = use_popover(anchor, opened(), options);
    popover.on_dismiss(move || opened.set(false));
    let floating = *popover.floating();

    popover.show(opened().then(|| {
        rsx! {
            Box {
                id: "box",
                role: "dialog",
                aria_label: "Example popover",
                tabindex: "-1",
                attributes: popover.floating_events(),
                style: popover.style(),
                onmounted: floating.mount(),
                sx: sx().background("surface").padding("8px").z_index("var(--lsx-z-index-popover)"),
                Text { id: "box-text", "Popover content" }
                Button { id: "in-box", "Inside" }
            }
        }
    }));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "trigger",
                onmounted: anchor.mount(),
                onclick: move |_| opened.toggle(),
                attributes: popover.anchor_events(),
                aria_haspopup: "dialog",
                aria_expanded: "{opened()}",
                "Popover"
            }
            Text { id: "blank", "Nothing to focus here" }
            Button { id: "after", "After" }
        }
    }
}
