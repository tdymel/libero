//! `ScrollArea`'s handle: binding one changes nothing in the markup, and a
//! call with nothing mounted is a no-op rather than a panic.

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{ScrollArea, Text, Virtualize, use_scroll_area},
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

/// Unmeasured - SSR never lays out - the area is no tab stop: it becomes one
/// only once it is seen to overflow with nothing focusable inside (585).
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
    assert!(!html.contains("role="), "{html}");
}

/// APG's scrollable region: a tab stop is a named `region`.
#[test]
fn a_focusable_scroll_area_is_a_region_tab_stop() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ScrollArea { focusable: true, aria_label: "Terms", "scrollable content" }
            }
        }
    }

    let html = body(&render(app));

    assert!(html.contains(r#"tabindex="0""#));
    assert!(!html.contains(r#"tabindex="-1""#));
    assert!(html.contains(r#"role="region""#), "{html}");
}

/// The caller's own `tabindex` and `role` win over the area's.
#[test]
fn a_callers_tabindex_and_role_win() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ScrollArea { focusable: true, tabindex: "-1", role: "group", "content" }
            }
        }
    }

    let html = body(&render(app));

    assert!(!html.contains(r#"tabindex="0""#), "{html}");
    assert!(!html.contains(r#"role="region""#), "{html}");
}

fn rows(html: &str) -> usize {
    html.matches("data-row=").count()
}

/// A given `item_size` settles the pitch before the `ScrollArea` has measured
/// anything, and a server render never measures. It renders the rows of an
/// assumed 1080px viewport, not the whole list: 54 rows of 20px, one more for
/// the partial row at the edge, and the theme's 4 of overscan.
#[test]
fn a_given_item_size_renders_a_first_screenful_before_anything_is_measured() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ScrollArea {
                    Virtualize {
                        count: 50_000,
                        item_size: Some(20.0),
                        item: move |i: usize| rsx! { div { "data-row": "{i}", "Row {i}" } },
                    }
                }
            }
        }
    }

    let html = body(&render(app));

    assert_eq!(rows(&html), 59, "rendered rows");
    assert!(
        html.contains(r#"data-row="0""#),
        "the window starts at the top"
    );
    assert!(
        !html.contains(r#"data-row="59""#),
        "the window ends at row 58"
    );
}

/// The control: the count above is the window, not a cap on what this markup
/// can show. With no `ScrollArea` there is nothing to window against.
#[test]
fn without_a_scroll_area_every_row_renders() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Virtualize {
                    count: 100,
                    item_size: Some(20.0),
                    item: move |i: usize| rsx! { div { "data-row": "{i}", "Row {i}" } },
                }
            }
        }
    }

    assert_eq!(rows(&body(&render(app))), 100);
}
