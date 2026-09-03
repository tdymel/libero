//! `ScrollArea`'s handle: binding one changes nothing in the markup, and a
//! call with nothing mounted is a no-op rather than a panic.

mod common;

use common::{body, render};

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
