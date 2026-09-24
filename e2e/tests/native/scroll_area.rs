//! `ScrollArea` round a `Virtualize` list: the window fits the pane, a wheel
//! renders the rows it scrolls to, and a pane resize re-measures (todo 425).

use std::time::Duration;

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::{
    components::{
        Box, Button, Container, Flex, ScrollArea, ScrollPositionEvent, Sidebar, Virtualize,
    },
    sx::sx,
};

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

/// Whether rows are drawn and the last one satisfies `fits`: the pane's
/// measure is a timer, late on a loaded machine.
fn last_row(page: &Page, fits: impl Fn(usize) -> bool) -> bool {
    page.query_all(&format!("{PANE} [data-row]"))
        .into_iter()
        .filter_map(|id| page.attr_of(id, "data-row")?.parse().ok())
        .max()
        .is_some_and(fits)
}

#[test]
fn the_window_fits_the_short_pane() {
    let mut page = mount(app);
    page.wait_for(|page| last_row(page, |last| last < 29));
    let (_, last) = rows(&page);
    assert!(last < 29, "rows to {last} for a 120px pane");
}

/// Blitz keeps painting its own bar under `Always`: a drawn one would double it (1010).
#[test]
fn blitz_draws_no_track_of_ours() {
    let mut page = mount(app);
    page.wait_for(|page| last_row(page, |last| last < 29));
    assert!(
        page.query_all(&format!("{PANE} [data-scrollbars]"))
            .is_empty(),
        "a drawn track under Blitz:\n{}",
        page.tree()
    );
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

fn still() -> Element {
    rsx! {
        div { id: "list-pane", height: "120px", "Still" }
    }
}

/// Todo 1164: a gesture latched in one document does not take the next one's
/// wheel at the same point.
#[test]
fn a_wheel_latch_stays_in_its_document() {
    let mut before = mount(still);
    before.hover(PANE);
    before.wheel(PANE, 40.0);
    let mut page = mount(app);
    page.wait_for(|page| last_row(page, |last| last < 29));
    page.hover(PANE);
    page.wheel(PANE, 40.0);
    assert_eq!(page.scroll_top(&format!("{PANE} > *")), 40.0);
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

fn plain(lines: usize, link: bool) -> Element {
    rsx! {
        div { height: "120px",
            ScrollArea { id: "area", "aria-label": "Notes",
                for i in 0..lines {
                    p { key: "{i}", height: "20px", "Line {i}" }
                }
                if link {
                    a { href: "#top", "Top" }
                }
            }
        }
    }
}

/// Todo 585: an overflowing area of plain text is a region tab stop; one that
/// fits, or holds a link, is none. Checked without a `resize` from Blitz.
#[test]
fn only_an_overflowing_area_of_plain_content_is_a_tab_stop() {
    for (app, stop) in [
        (|| plain(40, false)) as fn() -> Element,
        || plain(2, false),
        || plain(40, true),
    ]
    .into_iter()
    .zip([true, false, false])
    {
        let mut page = mount(app);
        // A loaded machine lays out late: wait for the stop, or long enough to rule it out.
        for _ in 0..25 {
            page.wait(Duration::from_millis(20));
            if stop && page.attr("#area", "tabindex").as_deref() == Some("0") {
                break;
            }
        }
        let tabindex = page.attr("#area", "tabindex");
        let role = page.attr("#area", "role");
        assert_eq!(
            (
                tabindex.as_deref() == Some("0"),
                role.as_deref() == Some("region")
            ),
            (stop, stop),
            "{tabindex:?} {role:?}:\n{}",
            page.tree()
        );
    }
}

fn nested() -> Element {
    let mut grown = use_context_provider(|| Signal::new(false));
    rsx! {
        Button { id: "grow", onclick: move |_| grown.set(true), "Grow" }
        div { height: "120px",
            ScrollArea { id: "area", "aria-label": "Notes", Lines {} }
        }
    }
}

#[component]
fn Lines() -> Element {
    let grown = use_context::<Signal<bool>>();
    rsx! {
        for i in 0..if grown() { 40 } else { 2 } {
            p { key: "{i}", height: "20px", "Line {i}" }
        }
    }
}

/// Todo 681: the web re-checks on a `MutationObserver`; Blitz reports no
/// mutation, so the platform watches the area's scroll size (todo 788).
#[test]
fn a_change_inside_a_child_component_re_checks_the_tab_stop() {
    let mut page = mount(nested);
    page.wait(Duration::from_millis(50));
    page.click("#grow");
    for _ in 0..25 {
        page.wait(Duration::from_millis(20));
    }
    assert_eq!(
        page.attr("#area", "tabindex").as_deref(),
        Some("0"),
        "{}",
        page.tree()
    );
}

/// The docs shell round the ScrollArea page's preview: a header, then a row
/// of the nav and the page area, sized to what is left of the window.
fn preview_in_page() -> Element {
    rsx! {
        div { height: "60px", "Header" }
        Flex { direction: "row", align: "stretch", wrap: false, sx: sx().height("calc(100vh - 60px)"),
            Sidebar {
                for i in 0..60 {
                    p { key: "nav-{i}", id: "nav-{i}", height: "20px", margin: "0", "Nav {i}" }
                }
            }
            ScrollArea { id: "page-area", "aria-label": "Page", sx: sx().flex("1").min_height("0").min_width("0"),
                Container { component: "main",
                    for i in 0..10 {
                        p { key: "intro-{i}", id: "intro-{i}", height: "20px", margin: "0", "Intro {i}" }
                    }
                    Box { sx: sx().height("160px").width("100%"),
                        ScrollArea { id: "preview", "aria-label": "Preview",
                            Box { sx: sx().width("150%"),
                                for i in 0..20 {
                                    p { key: "{i}", id: "item-{i}", height: "20px", margin: "0", "Item {i}" }
                                }
                            }
                        }
                    }
                    for i in 0..60 {
                        p { key: "outro-{i}", id: "outro-{i}", height: "20px", margin: "0", "Outro {i}" }
                    }
                }
            }
        }
    }
}

/// The preview's, the page area's and the window's scroll offsets.
fn scrolled(page: &Page) -> (f64, f64, f32) {
    (
        page.scroll_top("#preview"),
        page.scroll_top("#page-area"),
        page.viewport_scroll().1,
    )
}

/// Todo 717: a wheel moves the scroller under the pointer only.
#[test]
fn a_wheel_outside_a_nested_area_scrolls_the_page_alone() {
    let mut page = mount(preview_in_page);
    page.wait(Duration::from_millis(50));
    page.hover("#intro-1");
    for _ in 0..3 {
        page.wheel("#intro-1", 20.0);
    }
    let (preview, area, window) = scrolled(&page);
    assert_eq!((preview, window), (0.0, 0.0), "{}", page.tree());
    assert!(area > 0.0);
}

/// Todo 717: a wheel stays latched to the scroller it began over, as on the web, though
/// the preview moves under the pointer. Blitz does not latch; the platform does (todo 789).
#[test]
fn a_wheel_keeps_scrolling_the_page_as_a_nested_area_passes_under_it() {
    let mut page = mount(preview_in_page);
    page.wait(Duration::from_millis(50));
    page.hover("#intro-9");
    let (x, y, width, height) = page.rect("#intro-9");
    for _ in 0..10 {
        page.wheel_at((x + width / 2.0) as f32, (y + height / 2.0) as f32, 20.0);
    }
    let (preview, area, window) = scrolled(&page);
    assert_eq!((preview, window), (0.0, 0.0), "page area at {area}");
    assert_eq!(area, 200.0);
}

#[test]
fn a_wheel_inside_a_nested_area_scrolls_it_alone() {
    let mut page = mount(preview_in_page);
    page.wait(Duration::from_millis(50));
    page.hover("#item-1");
    page.wheel("#item-1", 60.0);
    let (preview, area, window) = scrolled(&page);
    assert!(preview > 0.0);
    assert_eq!((area, window), (0.0, 0.0), "{}", page.tree());
}

#[test]
fn a_wheel_over_the_nav_scrolls_it_alone() {
    let mut page = mount(preview_in_page);
    page.wait(Duration::from_millis(50));
    page.hover("#nav-3");
    page.wheel("#nav-3", 60.0);
    let (preview, area, window) = scrolled(&page);
    assert_eq!((preview, area, window), (0.0, 0.0, 0.0), "{}", page.tree());
}

fn raised_row() -> Element {
    let mut percent = use_signal(|| 0.0);
    rsx! {
        div { height: "40px" }
        div { height: "60px",
            ScrollArea {
                id: "area",
                onscroll: move |event: ScrollPositionEvent| {
                    let (ScrollPositionEvent::Start(_, y)
                    | ScrollPositionEvent::Change(_, y)
                    | ScrollPositionEvent::End(_, y)) = event;
                    percent.set(y);
                },
                div { height: "30px" }
                div { id: "raised", position: "relative", z_index: "1", height: "20px", background: "red" }
                div { height: "200px" }
            }
        }
        p { id: "percent", "{percent}" }
    }
}

/// Blitz paints a z-indexed box with its stacking context, past the clip of a
/// scroller that is none; the area is one natively.
#[test]
fn a_z_indexed_row_scrolled_out_is_clipped() {
    let mut page = mount(raised_row);
    page.wait(Duration::from_millis(50));
    page.hover("#raised");
    page.wheel("#raised", 45.0);
    // The row now spans 25..45, the area starts at 40.
    assert_eq!(page.painted_pixel(20, 30), "rgb(255, 255, 255)");
    assert_eq!(page.painted_pixel(20, 42), "rgb(255, 0, 0)");
}

/// Blitz's `scroll_height` is the range, not the content: the bottom is 100%.
#[test]
fn the_bottom_reports_a_hundred_percent() {
    let mut page = mount(raised_row);
    page.wait(Duration::from_millis(50));
    page.hover("#raised");
    page.wheel("#raised", 1000.0);
    assert_eq!(page.text("#percent"), "100");
}

fn wide_rtl() -> Element {
    rsx! {
        div { dir: "rtl", width: "300px", height: "120px",
            ScrollArea { id: "area", "aria-label": "Wide", scrollbars: "horizontal",
                div { id: "wide", width: "600px", height: "20px", "Start" }
            }
        }
    }
}

/// Todo 707: Blitz lays the content out from the right, overflowing to the
/// left, but clamps `scrollLeft` at 0, so that overflow is out of reach.
#[test]
#[ignore = "needs Blitz: RTL scroll origin, no scrolling into negative overflow (todo 707)"]
fn under_rtl_a_wheel_reaches_the_overflow_on_the_left() {
    let mut page = mount(wide_rtl);
    page.wait(Duration::from_millis(50));
    let before = page.rect("#wide").0;
    page.hover("#wide");
    page.wheel_x("#wide", -120.0);
    assert_eq!(page.rect("#wide").0, before + 120.0);
}

/// Blitz sends no `resize`; the platform's watch measures the pane (todo 788).
#[test]
fn a_taller_pane_renders_rows_to_its_new_bottom() {
    let mut page = mount(app);
    page.wait_for(|page| last_row(page, |last| last < 29));
    page.click("#grow");
    page.wait_for(|page| last_row(page, |last| last >= 29));
    let (_, last) = rows(&page);
    assert!(last >= 29, "rows to {last} for a 600px pane");
}
