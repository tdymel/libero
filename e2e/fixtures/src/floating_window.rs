//! `FloatingWindow`, for the overlay archetype.

use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, FloatingWindowOptions, MenuPart, Parts, Text, WindowRect},
    hooks::use_floating_window,
    sx::sx,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/floating-window", || rsx! { FloatingWindowPage {} }),
    ("/floating-window-pair", || rsx! { WindowPairPage {} }),
    ("/floating-window-sized", || rsx! { SizedWindowPage {} }),
    ("/floating-window-tall", || rsx! { TallWindowPage {} }),
    (
        "/floating-window-anchored",
        || rsx! { AnchoredWindowPage {} },
    ),
    ("/floating-window-pinned", || rsx! { PinnedWindowPage {} }),
];

/// Todo 2353: a window placed at the bottom end, not yet moved, resized by key.
#[component]
fn AnchoredWindowPage() -> Element {
    let window = use_floating_window(
        FloatingWindowOptions {
            title: Some("Anchored".into()),
            resizable: true,
            placement: "bottom-end".into(),
            sx: sx().width("400px").height("200px").into(),
            ..Default::default()
        },
        |_| rsx! { Text { "A window at the bottom end." } },
    );

    rsx! {
        Button { id: "open-window", variant: "outlined", onclick: move |_| window.open(), "Anchored" }
    }
}

/// Todo 2641: a pinned window at the bottom end, resized by key like a pointer drag.
#[component]
fn PinnedWindowPage() -> Element {
    let window = use_floating_window(
        FloatingWindowOptions {
            title: Some("Pinned".into()),
            resizable: true,
            pinned: true,
            placement: "bottom-end".into(),
            sx: sx().width("400px").height("200px").into(),
            ..Default::default()
        },
        |_| rsx! { Text { "A pinned window at the bottom end." } },
    );

    rsx! {
        Button { id: "open-window", variant: "outlined", onclick: move |_| window.open(), "Pinned" }
    }
}

/// Content taller than the viewport: the window caps and its body scrolls (todo 1307).
#[component]
fn TallWindowPage() -> Element {
    let window = use_floating_window(
        FloatingWindowOptions {
            title: Some("Terms".into()),
            ..Default::default()
        },
        |window| {
            rsx! {
                for line in 0..60 {
                    Text { "Clause {line}: a long line of terms the reader has to scroll to." }
                }
                Button { id: "window-done", variant: "text", onclick: move |_| window.close(), "Done" }
            }
        },
    );

    rsx! {
        Button { id: "open-window", variant: "outlined", onclick: move |_| window.open(), "Terms" }
    }
}

/// A caller's `sx` size is only the initial size: a resize wins over it (todo 922).
/// `menu_parts` reaches the portaled title-bar menu's labels.
#[component]
fn SizedWindowPage() -> Element {
    let mut resized = use_signal(String::new);
    let record_resize = use_callback(move |rect: WindowRect| resized.set(rect_text(rect)));
    let window = use_floating_window(
        FloatingWindowOptions {
            title: Some("Sized".into()),
            resizable: true,
            sx: sx().width("400px").height("200px").into(),
            menu_parts: Parts::new()
                .part(MenuPart::Label, sx().font_style("italic"))
                .into(),
            onresize: Some(record_resize),
            ..Default::default()
        },
        |_| rsx! { Text { "A window the caller sized." } },
    );

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "open-window", variant: "outlined", onclick: move |_| window.open(), "Sized" }
            div { id: "resize-report", "{resized}" }
        }
    }
}

/// Two windows over one page: stacking, one Escape per window, focus return
/// from each, a close from the page, and a long title at 320px.
#[component]
fn WindowPairPage() -> Element {
    let first = use_floating_window(
        FloatingWindowOptions {
            title: Some("Configurationinspectorforthecurrentlyselectedlayer".into()),
            resizable: true,
            ..Default::default()
        },
        |window| {
            rsx! {
                Button { id: "first-done", variant: "text", onclick: move |_| window.close(), "Done" }
            }
        },
    );
    let second = use_floating_window(
        FloatingWindowOptions {
            title: Some("Second".into()),
            placement: "top-start".into(),
            ..Default::default()
        },
        |_| rsx! { Text { "The second window." } },
    );

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "open-first", variant: "outlined", onclick: move |_| first.open(), "First" }
            Button { id: "open-second", variant: "outlined", onclick: move |_| second.open(), "Second" }
            Button { id: "close-first", variant: "outlined", onclick: move |_| first.close(), "Close first" }
        }
    }
}

/// Resizable window with explicit `sx` bounds, so Home/End clamp to them rather than the viewport.
/// `onmove`/`onresize` render into readouts, empty at rest (`codebase/components/floating-window`).
#[component]
fn FloatingWindowPage() -> Element {
    let mut moved = use_signal(String::new);
    let mut resized = use_signal(String::new);
    let record_move = use_callback(move |rect: WindowRect| moved.set(rect_text(rect)));
    let record_resize = use_callback(move |rect: WindowRect| resized.set(rect_text(rect)));

    let inspector = use_floating_window(
        FloatingWindowOptions {
            title: Some("Inspector".into()),
            resizable: true,
            sx: sx()
                .min_width("240px")
                .min_height("120px")
                .max_width("480px")
                .max_height("360px")
                .into(),
            onmove: Some(record_move),
            onresize: Some(record_resize),
            ..Default::default()
        },
        |window| {
            // A set width: a text-sized window follows the machine's fonts into the snapshot.
            rsx! {
                div { style: "width: 360px; max-width: 100%;",
                    Text { "Drag the title bar, or focus it and use the arrow keys." }
                    Button { id: "window-done", variant: "text", onclick: move |_| window.close(), "Done" }
                }
            }
        },
    );

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "open-window",
                variant: "outlined",
                onclick: move |_| {
                    inspector.open();
                },
                "Inspector"
            }
            div { id: "move-report", "{moved}" }
            div { id: "resize-report", "{resized}" }
        }
    }
}

/// `x y width height`, rounded, which is what the tests parse.
fn rect_text(rect: WindowRect) -> String {
    format!(
        "{} {} {} {}",
        rect.x.round(),
        rect.y.round(),
        rect.width.round(),
        rect.height.round()
    )
}
