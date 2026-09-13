//! `FloatingWindow`, for the overlay archetype.

use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, FloatingWindowOptions, Text, WindowRect},
    hooks::use_floating_window,
    sx::sx,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/floating-window", || rsx! { FloatingWindowPage {} }),
    ("/floating-window-pair", || rsx! { WindowPairPage {} }),
];

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

/// `FloatingWindow`: a non-modal window whose live state is its geometry.
///
/// The window is resizable and carries explicit bounds in its own `sx`, so the
/// separator's Home and End have something to clamp against: Home asks for
/// `0x0` and End for `u16::MAX`, and what comes back is the caller's minimum
/// and maximum. Without bounds both would land on the viewport, which measures
/// the browser rather than the component.
///
/// Both reports are wired into text on the page. `onmove` and `onresize` are
/// the only way a caller learns where the window went, and they are owed to an
/// effect rather than written in the handler - reading the rect in the same
/// task reports the *previous* one
/// (`codebase/components/floating-window`). A report nobody reads is a claim
/// no test can check, so the fixture reads them.
///
/// The two readouts are empty until the window says something, so the resting
/// accessibility baseline carries no text of theirs.
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
            rsx! {
                Text { "Drag the title bar, or focus it and use the arrow keys." }
                Button { id: "window-done", variant: "text", onclick: move |_| window.close(), "Done" }
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
