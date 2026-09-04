//! `ScrollArea`'s handle: binding one changes nothing in the markup, and a
//! call with nothing mounted is a no-op rather than a panic.

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{ScrollArea, Text, use_scroll_area},
};

/// The handle only takes over the element; the area renders the same.
#[test]
fn a_bound_handle_renders_the_same_markup() {
    fn with_handle() -> Element {
        let area = use_scroll_area();
        rsx! {
            LiberoProvider {
                ScrollArea { handle: area, Text { "content" } }
            }
        }
    }
    fn without() -> Element {
        rsx! {
            LiberoProvider {
                ScrollArea { Text { "content" } }
            }
        }
    }

    assert_eq!(body(&render(with_handle)), body(&render(without)));
}

/// SSR never mounts, so every call here runs against an unmounted handle -
/// bound or not.
#[test]
fn a_call_before_mount_is_a_no_op() {
    fn app() -> Element {
        let bound = use_scroll_area();
        let unbound = use_scroll_area();
        for area in [bound, unbound] {
            area.scroll_to(0.0, 120.0);
            area.scroll_to_percent(Some(20.0), Some(20.0));
        }
        rsx! {
            LiberoProvider {
                ScrollArea { handle: bound, Text { "content" } }
            }
        }
    }

    assert!(body(&render(app)).contains("content"));
}

#[test]
fn scroll_area_renders_its_content() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ScrollArea { "scrollable content" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("scrollable content"));
}

/// The default opts the viewport *out* of Chromium's implicit tab stop, which
/// is what every existing call site relies on.
#[test]
fn a_scroll_area_is_not_a_tab_stop_by_default() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ScrollArea { "scrollable content" }
            }
        }
    }

    let html = body(&render(app));

    assert!(html.contains(r#"tabindex="-1""#));
    assert!(!html.contains(r#"tabindex="0""#));
}

#[test]
fn a_focusable_scroll_area_is_a_tab_stop() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ScrollArea { focusable: true, "scrollable content" }
            }
        }
    }

    let html = body(&render(app));

    assert!(html.contains(r#"tabindex="0""#));
    assert!(!html.contains(r#"tabindex="-1""#));
}
