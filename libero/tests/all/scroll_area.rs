//! `ScrollArea`'s handle: binding one changes nothing in the markup, and a
//! call with nothing mounted is a no-op rather than a panic.

use std::cell::Cell;

use crate::common::{body, render};

use dioxus::{core::NoOpMutations, prelude::*};
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

thread_local! {
    /// Rows `counted_row` rendered since the last reset.
    static RENDERED: Cell<usize> = const { Cell::new(0) };
}

fn counted_row(i: usize) -> Element {
    RENDERED.with(|rendered| rendered.set(rendered.get() + 1));
    rsx! { div { "data-row": "{i}", "Row {i}" } }
}

/// Runs what is queued, effects too, and renders it.
fn settle(dom: &mut VirtualDom) {
    for _ in 0..3 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
}

/// A `Virtualize` replacing the owner, as a table's rows when reorder turns on, renders
/// its window: every row of 10k cost 40k blocking mounts on a phone (todo 2584).
#[test]
fn a_replacing_virtualize_renders_its_window_not_every_row() {
    fn app() -> Element {
        let wrapped = use_context_provider(|| Signal::new(false));
        rsx! {
            LiberoProvider {
                ScrollArea {
                    if wrapped() {
                        div { Virtualize { count: 10_000, item_size: Some(20.0), item: counted_row } }
                    } else {
                        Virtualize { count: 10_000, item_size: Some(20.0), item: counted_row }
                    }
                }
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    settle(&mut dom);
    RENDERED.with(|rendered| rendered.set(0));
    dom.in_scope(ScopeId::APP, || consume_context::<Signal<bool>>().set(true));
    settle(&mut dom);

    let rendered = RENDERED.with(Cell::get);
    assert!(
        rendered < 200,
        "{rendered} rows rendered for a 59-row window"
    );
    assert_eq!(rows(&body(&dioxus_ssr::render(&dom))), 59);
}

/// The control: a second list in one area never gets the offsets, so it renders every row.
#[test]
fn a_second_virtualize_in_one_area_renders_every_row() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ScrollArea {
                    Virtualize {
                        count: 10_000,
                        item_size: Some(20.0),
                        item: move |i: usize| rsx! { div { "data-row": "{i}" } },
                    }
                    Virtualize {
                        count: 30,
                        item_size: Some(20.0),
                        item: move |i: usize| rsx! { div { "data-second": "{i}" } },
                    }
                }
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    settle(&mut dom);
    let html = body(&dioxus_ssr::render(&dom));

    assert_eq!(rows(&html), 59, "the owner windows");
    assert_eq!(
        html.matches("data-second=").count(),
        30,
        "the second renders all"
    );
}
