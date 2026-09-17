use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Box, Button, Code, CodeBlock, Flex, Text},
    hooks::{Align, PopoverOptions, Side, use_element, use_id, use_popover},
    platform::ElementApi,
    sx::sx,
    use_theme,
};

const SHIPPING: &str = r#"#[component]
fn ShippingInfo() -> Element {
    let theme = use_theme();
    let mut opened = use_signal(|| false);
    let anchor = use_element();
    let box_id = use_id();

    let popover = use_popover(
        anchor,
        opened(),
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            .side(Side::Bottom)
            .align(Align::Start),
    );
    let floating = *popover.floating();

    // A dialog takes focus once placed, once per opening.
    let mut entered = use_signal(|| false);
    use_effect(move || match (opened(), popover.placed()) {
        (true, true) if !*entered.peek() => {
            entered.set(true);
            let _ = floating.focus();
        }
        (false, _) => entered.set(false),
        _ => {}
    });
    // Escape and Tab close it and hand focus back to the trigger.
    let mut close = move |event: &KeyboardEvent| {
        opened.set(false);
        let _ = anchor.focus();
        if event.key() == Key::Escape || event.modifiers().shift() {
            event.prevent_default();
        }
    };

    popover.show(opened().then(|| rsx! {
        Box {
            id: "{box_id}",
            role: "dialog",
            aria_label: "Shipping",
            tabindex: "-1",
            onkeydown: move |event: KeyboardEvent| {
                if matches!(event.key(), Key::Escape | Key::Tab) {
                    close(&event);
                }
            },
            style: popover.style(),
            onmounted: floating.mount(),
            sx: sx().background("surface").padding("var(--lsx-popover-padding)"),
            "Two to four working days."
        }
    }));

    rsx! {
        Button {
            onmounted: anchor.mount(),
            onclick: move |_| opened.toggle(),
            // Focus is still here before the box is placed.
            onkeydown: move |event: KeyboardEvent| match event.key() {
                Key::Escape if opened() => {
                    event.prevent_default();
                    opened.set(false);
                }
                Key::Tab if opened() => opened.set(false),
                _ => {}
            },
            aria_haspopup: "dialog",
            aria_expanded: "{opened()}",
            aria_controls: "{box_id}",
            "Shipping"
        }
    }
}"#;

/// `SHIPPING`, rendered.
#[component]
fn ShippingInfo() -> Element {
    let theme = use_theme();
    let mut opened = use_signal(|| false);
    let anchor = use_element();
    let box_id = use_id();

    let popover = use_popover(
        anchor,
        opened(),
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            .side(Side::Bottom)
            .align(Align::Start),
    );
    let floating = *popover.floating();

    // A dialog takes focus once placed, once per opening.
    let mut entered = use_signal(|| false);
    use_effect(move || match (opened(), popover.placed()) {
        (true, true) if !*entered.peek() => {
            entered.set(true);
            let _ = floating.focus();
        }
        (false, _) => entered.set(false),
        _ => {}
    });
    // Escape and Tab close it and hand focus back to the trigger.
    let mut close = move |event: &KeyboardEvent| {
        opened.set(false);
        let _ = anchor.focus();
        if event.key() == Key::Escape || event.modifiers().shift() {
            event.prevent_default();
        }
    };

    popover.show(opened().then(|| {
        rsx! {
            Box {
                id: "{box_id}",
                role: "dialog",
                aria_label: "Shipping",
                tabindex: "-1",
                onkeydown: move |event: KeyboardEvent| {
                    if matches!(event.key(), Key::Escape | Key::Tab) {
                        close(&event);
                    }
                },
                style: popover.style(),
                onmounted: floating.mount(),
                sx: sx().background("surface").padding("var(--lsx-popover-padding)"),
                "Two to four working days."
            }
        }
    }));

    rsx! {
        Button {
            onmounted: anchor.mount(),
            onclick: move |_| opened.toggle(),
            // Focus is still here before the box is placed.
            onkeydown: move |event: KeyboardEvent| match event.key() {
                Key::Escape if opened() => {
                    event.prevent_default();
                    opened.set(false);
                }
                Key::Tab if opened() => opened.set(false),
                _ => {}
            },
            aria_haspopup: "dialog",
            aria_expanded: "{opened()}",
            aria_controls: "{box_id}",
            "Shipping"
        }
    }
}

#[component]
pub fn UsePopoverPage() -> Element {
    rsx! {
        DocPage {
            title: "use_popover",
            source: "libero/src/hooks/popover/mod.rs",
            markdown: "/md/use_popover.md",
            lead: rsx! {
                Text {
                    Code { source: "use_popover(anchor, open, options) -> PopoverHandle" }
                    " portals a box to the document root and places it next to an anchor, "
                    "flipping and shifting to stay on screen. It owns no open state and adds "
                    "no semantics. "
                    Anchor { to: Route::PopoverPage {}, "Popover" }
                    " has the full story."
                }
            },

            DocSection {
                title: "Usage",
                Flex { align: "flex-start", ShippingInfo {} }
                CodeBlock { source: SHIPPING, language: "rust" }
                Text {
                    "Most of the code is the keyboard, because the hook leaves it to you. "
                    Code { source: "show(None)" }
                    " is how a closed popover stops rendering."
                }
            }

            DocSection {
                title: "Accessibility",
                Text {
                    "Give the box its role and the trigger "
                    Code { source: "aria-haspopup" }
                    ", "
                    Code { source: "aria-expanded" }
                    " and "
                    Code { source: "aria-controls" }
                    ". Drive "
                    Code { source: "aria-expanded" }
                    " from the same signal you pass the hook. Escape must close the box, and "
                    "focus is not trapped. Tab closes it and moves on."
                }
            }

            DocSection {
                title: "Web and native",
                Text {
                    "Natively an open popover drifts when the page scrolls, because only the "
                    "web reports document scrolls. The box is portaled, so listen for Escape "
                    "on the trigger as well, as above."
                }
            }
        }
    }
}
