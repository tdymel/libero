//! `:hover` on the DOM ancestors Blitz skips: it hovers layout parents only,
//! and a cell's layout parent is the table, not its `tr`. Also the `hover_tint` rows
//! (`NavLink`, `Tabs`): tinted under the mouse, clear after a finger tap.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{NavLink, Options, Tabs, Text};

#[derive(Clone, Copy, PartialEq, Debug, Options)]
enum Section {
    One,
    Two,
    Three,
}

fn nav_links() -> Element {
    rsx! {
        div { width: "240px", padding: "16px",
            NavLink { id: "a", to: "/a", active: true, "Alpha" }
            NavLink { id: "b", to: "/b", "Bravo" }
        }
    }
}

fn panel(section: Section) -> Element {
    rsx! { Text { "{section:?}" } }
}

fn tabs() -> Element {
    let mut section = use_signal(|| Section::One);
    rsx! {
        Tabs {
            aria_label: "Sections",
            value: section(),
            onchange: move |next| section.set(next),
            panel,
        }
    }
}

/// A tap cannot select a tab, so a stuck `:hover` would show on the tapped one.
fn pinned_tabs() -> Element {
    rsx! {
        Tabs { aria_label: "Sections", value: Section::One, onchange: |_| {}, panel }
    }
}

const TAB_TWO: &str = "[role=tab]:nth-of-type(2)";
const TAB_THREE: &str = "[role=tab]:nth-of-type(3)";

fn background(page: &Page, selector: &str) -> String {
    page.computed(selector, "background-color")
}

const HOVERED: &str = "rgb(255, 0, 0)";

fn app() -> Element {
    rsx! {
        style { "tr:hover {{ background-color: {HOVERED}; }}" }
        table {
            tbody {
                tr { id: "a", td { "Apple" } }
                tr { id: "b", td { "Banana" } }
            }
        }
    }
}

#[test]
fn a_hovered_cell_hovers_its_row_and_leaving_unhovers_it() {
    let mut page = mount(app);
    page.hover("#a td");
    assert_eq!(page.computed("#a", "background-color"), HOVERED);
    page.hover("#b td");
    assert_eq!(page.computed("#b", "background-color"), HOVERED);
    assert_ne!(
        page.computed("#a", "background-color"),
        HOVERED,
        "the row the pointer left is still hovered"
    );
}

/// Blitz reports `(pointer: none)`, which `hover_tint` counts as a hovering pointer:
/// the tint shows under the mouse (todo 4776).
#[test]
fn a_hovered_nav_link_row_is_tinted_and_leaving_clears_it() {
    let mut page = mount(nav_links);
    let rest = background(&page, "#b");
    page.hover("#b");
    assert_ne!(background(&page, "#b"), rest, "the hovered row has no tint");
    page.hover("#a");
    assert_eq!(
        background(&page, "#b"),
        rest,
        "the row left is still tinted"
    );
}

#[test]
fn a_finger_tap_on_a_nav_link_leaves_no_tint() {
    let mut page = mount(nav_links);
    let rest = background(&page, "#b");
    let (x, y) = page.touch_down("#b");
    page.touch_up(x, y);
    assert_eq!(
        background(&page, "#b"),
        rest,
        "the tap left the row hovered"
    );
}

#[test]
fn a_hovered_tab_is_tinted_and_leaving_clears_it() {
    let mut page = mount(tabs);
    let rest = background(&page, TAB_TWO);
    page.hover(TAB_TWO);
    assert_ne!(
        background(&page, TAB_TWO),
        rest,
        "the hovered tab has no tint"
    );
    page.hover(TAB_THREE);
    assert_eq!(
        background(&page, TAB_TWO),
        rest,
        "the tab left is still tinted"
    );
}

#[test]
fn a_finger_tap_on_a_tab_leaves_no_tint() {
    let mut page = mount(pinned_tabs);
    let rest = background(&page, TAB_TWO);
    let (x, y) = page.touch_down(TAB_TWO);
    page.touch_up(x, y);
    assert_eq!(
        background(&page, TAB_TWO),
        rest,
        "the tap left the tab hovered"
    );
}
