//! `use_popover` with `PopoverOptions::dismiss` (todo 522).

use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Flex, Text},
    hooks::{Align, PopoverOptions, PopoverWidth, Side, use_element, use_popover},
    sx::sx,
    use_theme,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/popover", || rsx! { PopoverPage {} }),
    (
        "/popover/place/below",
        || rsx! { PlacedPage { left: "200px", top: "100px" } },
    ),
    (
        "/popover/place/flip",
        || rsx! { PlacedPage { left: "200px", top: "calc(100vh - 60px)" } },
    ),
    (
        "/popover/place/shift",
        || rsx! { PlacedPage { left: "calc(100vw - 230px)", top: "100px", align: Align::Center } },
    ),
    (
        "/popover/place/match",
        || rsx! { PlacedPage { left: "200px", top: "100px", width: PopoverWidth::Match } },
    ),
    (
        "/popover/place/scroll",
        || rsx! { PlacedPage { left: "200px", top: "100px", tall: true } },
    ),
];

/// A 220px anchor placed absolutely, so the page's padding does not move it,
/// and a 150x60 box 8px off it (`use_popover`'s placement).
#[component]
fn PlacedPage(
    left: &'static str,
    top: &'static str,
    #[props(default = Align::Start)] align: Align,
    #[props(default = PopoverWidth::Auto)] width: PopoverWidth,
    /// A page taller than the viewport and a fixed `#scroll-by` button that
    /// scrolls it 120px, so an open box has to follow its anchor.
    #[props(default)]
    tall: bool,
) -> Element {
    let anchor = use_element();
    let mut opened = use_signal(|| false);
    let options = PopoverOptions::new(8.0, 0.0)
        .side(Side::Bottom)
        .align(align)
        .width(width);
    let popover = use_popover(anchor, opened(), options);
    let floating = *popover.floating();
    popover.show(opened().then(|| {
        rsx! {
            Box {
                id: "floating",
                style: popover.style(),
                onmounted: floating.mount(),
                div { width: "150px", height: "60px", "Popover content" }
            }
        }
    }));

    rsx! {
        div { position: "absolute", left, top,
            Button {
                id: "anchor",
                width: "220px",
                onmounted: anchor.mount(),
                onclick: move |_| opened.toggle(),
                "Anchor"
            }
        }
        if tall {
            div { height: "300vh" }
            div { position: "fixed", left: "8px", bottom: "8px",
                Button {
                    id: "scroll-by",
                    onclick: move |_| {
                        document::eval("window.scrollBy(0, 120)");
                    },
                    "Scroll"
                }
            }
        }
    }
}

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
