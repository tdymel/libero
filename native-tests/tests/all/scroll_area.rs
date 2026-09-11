//! `ScrollArea` round a `Virtualize` list: the window fits the pane, a wheel
//! renders the rows it scrolls to, and a pane resize re-measures (todo 425).

use std::time::Duration;

use dioxus::prelude::*;
use libero::components::{Button, ScrollArea, Virtualize};
use native_tests::{Page, mount};

const PANE: &str = "#list-pane";

fn app() -> Element {
    let mut tall = use_signal(|| false);
    rsx! {
        Button { id: "grow", onclick: move |_| tall.set(true), "Grow" }
        div { id: "list-pane", height: if tall() { "600px" } else { "120px" },
            ScrollArea {
                Virtualize {
                    count: 1000,
                    item_size: Some(20.0),
                    item: move |i: usize| rsx! {
                        div { "data-row": i, height: "20px", "Row {i}" }
                    },
                }
            }
        }
    }
}

fn rows(page: &Page) -> (usize, usize) {
    let rows: Vec<usize> = page
        .query_all(&format!("{PANE} [data-row]"))
        .into_iter()
        .filter_map(|id| page.attr_of(id, "data-row")?.parse().ok())
        .collect();
    let first = rows
        .iter()
        .copied()
        .min()
        .unwrap_or_else(|| panic!("no rows:\n{}", page.tree()));
    (first, rows.iter().copied().max().unwrap())
}

#[test]
fn the_window_fits_the_short_pane() {
    let mut page = mount(app);
    page.wait(Duration::from_millis(50));
    let (_, last) = rows(&page);
    assert!(last < 29, "rows to {last} for a 120px pane");
}

#[test]
fn a_wheel_renders_the_rows_it_scrolls_to() {
    let mut page = mount(app);
    page.hover("[data-row='2']");
    page.wheel("[data-row='2']", 2000.0);
    let (first, last) = rows(&page);
    assert!(
        first > 50 && last >= 100,
        "rows {first}..={last} after a 2000px wheel"
    );
}

fn probed() -> Element {
    rsx! {
        div { id: "list-pane", height: "120px",
            ScrollArea {
                Virtualize {
                    count: 1000,
                    item: move |i: usize| rsx! {
                        div { "data-row": i, height: "20px", "Row {i}" }
                    },
                }
            }
        }
    }
}

/// Without `item_size` the list measures its rows first.
#[test]
fn a_probed_row_height_windows_the_list() {
    let mut page = mount(probed);
    for _ in 0..10 {
        page.wait(Duration::from_millis(20));
    }
    let (_, last) = rows(&page);
    assert!(last < 29, "rows to {last} for a 120px pane");
}

/// Blitz sends no `resize` (measured, [[codebase/platform/blitz-platform-gaps]]).
#[test]
#[ignore = "needs Blitz: no resize event reaches ScrollArea's onresize"]
fn a_taller_pane_renders_rows_to_its_new_bottom() {
    let mut page = mount(app);
    page.wait(Duration::from_millis(50));
    page.click("#grow");
    page.wait(Duration::from_millis(50));
    let (_, last) = rows(&page);
    assert!(last >= 29, "rows to {last} for a 600px pane");
}
