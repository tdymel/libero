//! A sticky `Header` natively: Blitz lays `position: sticky` out as `relative`,
//! so the platform moves it to its scroller's top edge (todo 907).

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::{
    components::{Header, HeaderPosition, ScrollArea, Select, Table, column},
    sx::sx,
};

fn page_banner() -> Element {
    rsx! {
        p { id: "intro", height: "40px", margin: "0", "Intro" }
        Header { id: "banner", sx: sx().background("rgb(255, 0, 0)"), "Libero" }
        div { id: "content", height: "3000px", "Content" }
    }
}

fn top(page: &Page, selector: &str) -> f64 {
    page.rect(selector).1
}

#[test]
fn a_page_scroll_holds_the_banner_at_the_top() {
    let mut page = mount(page_banner);
    assert_eq!(top(&page, "#banner"), 40.0);
    page.hover("#content");
    page.wheel("#content", 300.0);
    page.wait_for(|page| top(page, "#banner") == 0.0);
    assert_eq!(top(&page, "#banner"), 0.0, "{}", page.tree());
    let (_, _, width, height) = page.rect("#banner");
    let (x, y) = ((width / 2.0) as u32, (height / 2.0) as u32);
    assert_eq!(page.painted_pixel(x, y), "rgb(255, 0, 0)");

    page.wheel("#content", -300.0);
    page.wait_for(|page| top(page, "#banner") == 40.0);
    assert_eq!(top(&page, "#banner"), 40.0, "it stayed stuck");
}

fn fixed_banner() -> Element {
    let mut presses = use_signal(|| 0);
    rsx! {
        div { padding: "40px",
            Header { id: "banner", position: HeaderPosition::Fixed, sx: sx().background("rgb(255, 0, 0)"),
                button { id: "press", onclick: move |_| presses += 1, "Pressed {presses}" }
            }
            div { id: "content", height: "3000px", "Content" }
        }
    }
}

/// A fixed banner in a padded parent holds at the window's corner through a
/// scroll and takes its presses there (todo 1409).
#[test]
fn a_fixed_banner_holds_at_the_window_corner() {
    let mut page = mount(fixed_banner);
    let corner = |page: &Page| (page.rect("#banner").0, top(page, "#banner"));
    page.wait_for(|page| corner(page) == (0.0, 0.0));
    assert_eq!(corner(&page), (0.0, 0.0), "{}", page.tree());
    page.hover("#content");
    page.wheel("#content", 300.0);
    page.wait_for(|page| top(page, "#content") < 0.0 && corner(page) == (0.0, 0.0));
    assert_eq!(corner(&page), (0.0, 0.0), "{}", page.tree());
    let (_, _, width, height) = page.rect("#banner");
    let (x, y) = ((width - 4.0) as u32, (height / 2.0) as u32);
    assert_eq!(page.painted_pixel(x, y), "rgb(255, 0, 0)");
    page.click("#press");
    assert_eq!(page.text("#press"), "Pressed 1", "{}", page.tree());
}

fn framed() -> Element {
    rsx! {
        div { height: "60px" }
        div { id: "frame", height: "200px", overflow: "auto",
            Header { id: "banner", "Libero" }
            div { id: "content", height: "1000px", "Content" }
        }
    }
}

#[test]
fn a_scrolling_frame_holds_its_banner_at_its_top() {
    let mut page = mount(framed);
    page.hover("#content");
    page.wheel("#content", 150.0);
    page.wait_for(|page| top(page, "#banner") == top(page, "#frame"));
    assert_eq!(
        top(&page, "#banner"),
        top(&page, "#frame"),
        "{}",
        page.tree()
    );
}

fn published() -> Element {
    rsx! {
        Header { id: "banner", publish_height: true, "Libero" }
        div { id: "box", height: "200px", overflow_y: "auto",
            for index in 0..10 {
                a { key: "{index}", id: "item-{index}", href: "#", display: "block", height: "40px", "Item {index}" }
            }
        }
        for index in 0..40 {
            a { key: "{index}", id: "link-{index}", href: "#", display: "block", height: "40px", "Link {index}" }
        }
    }
}

fn bottom(page: &Page, selector: &str) -> f64 {
    let (_, y, _, height) = page.rect(selector);
    y + height
}

/// Todo 1521: a published banner's padding reaches Blitz's focus scroll, so Shift+Tab
/// onto a link under the stuck banner scrolls it clear.
#[test]
fn a_shift_tab_under_a_published_banner_scrolls_clear_of_it() {
    let mut page = mount(published);
    page.hover("#link-20");
    page.wheel("#link-20", 1000.0);
    page.wait_for(|page| top(page, "#banner") == 0.0);
    // The link just above the banner's bottom edge, under it.
    let link = |index: usize| format!("#link-{index}");
    let index = (0..39)
        .find(|&index| {
            let y = top(&page, &link(index));
            y >= 0.0 && y < bottom(&page, "#banner")
        })
        .expect("a link under the banner");
    let under = link(index);
    page.focus(&link(index + 1));
    page.shift_tab();
    page.settle();
    assert!(page.is_focused(&under), "{}", page.focus_owner());
    // The scroll lands first, the sticky move after it: wait for the banner to stick again.
    page.wait_for(|page| top(page, "#banner") == 0.0);
    assert!(
        top(&page, &under) >= bottom(&page, "#banner") - 0.5,
        "{under} at {} under the banner's bottom {}",
        top(&page, &under),
        bottom(&page, "#banner")
    );
}

/// Todo 1521: the root's padding var is inherited, not a nested scroller's own: a shown
/// item at its box's top edge stays put.
#[test]
fn a_nested_scroller_keeps_none_of_the_banners_padding() {
    let mut page = mount(published);
    page.hover("#box");
    page.wheel("#box", 200.0);
    page.wait_for(|page| page.scroll_top("#box") == 200.0);
    page.focus("#item-6");
    page.shift_tab();
    page.settle();
    assert!(page.is_focused("#item-5"), "{}", page.focus_owner());
    assert_eq!(
        page.scroll_top("#box"),
        200.0,
        "the box took the banner's padding"
    );
}

#[derive(Clone, PartialEq)]
struct Note {
    id: usize,
}

fn published_scrollers() -> Element {
    scrollers(true)
}

fn quiet_scrollers() -> Element {
    scrollers(false)
}

/// libero's own scrollers under a banner: a ScrollArea, a Select's list, and a capped
/// Table with a scroller in a cell.
fn scrollers(publish_height: bool) -> Element {
    let mut value = use_signal(|| None::<String>);
    let options = use_hook(|| (0..40).map(|n| format!("Item {n}")).collect::<Vec<_>>());
    rsx! {
        Header { id: "banner", publish_height, "Libero" }
        div { height: "120px",
            ScrollArea { id: "area", "aria-label": "Notes",
                for index in 0..10 {
                    a { key: "{index}", id: "line-{index}", href: "#", display: "block", height: "20px", "Line {index}" }
                }
            }
        }
        Select {
            label: "Page",
            options,
            value: value(),
            onchange: move |next| value.set(next),
        }
        Table {
            max_height: "200px",
            data: (0..3).map(|id| Note { id }).collect::<Vec<_>>(),
            columns: vec![
                column("Notes").value(|note: &Note| note.id).render(|note: &Note| {
                    let id = note.id;
                    rsx! {
                        div { id: "notes-{id}", height: "40px", overflow_y: "auto",
                            for index in 0..5 {
                                a { key: "{index}", id: "note-{id}-{index}", href: "#", display: "block", height: "20px", "Note {index}" }
                            }
                        }
                    }
                }),
            ],
        }
        div { height: "2000px" }
    }
}

const PADDING: &str = "--lsx-scroll-padding-top";
/// A ScrollArea's focus scroll, from a line at its top edge: Shift+Tab onto it.
fn area_scroll_after_shift_tab(app: fn() -> Element) -> f64 {
    let mut page = mount(app);
    assert_eq!(page.computed("#area", PADDING), "0px");
    page.hover("#area");
    page.wheel("#area", 100.0);
    page.wait_for(|page| page.scroll_top("#area") == 100.0);
    page.focus("#line-6");
    page.shift_tab();
    page.settle();
    assert!(page.is_focused("#line-5"), "{}", page.focus_owner());
    page.scroll_top("#area")
}

/// Todo 1521: a ScrollArea under a published banner scrolls as under a quiet one.
#[test]
fn a_scroll_area_keeps_none_of_the_banners_padding() {
    assert_eq!(
        area_scroll_after_shift_tab(published_scrollers),
        area_scroll_after_shift_tab(quiet_scrollers),
        "the area took the banner's padding"
    );
}

/// Todo 1521: an open Select's list under a published banner keeps none of its padding.
#[test]
fn a_select_list_keeps_none_of_the_banners_padding() {
    let mut page = mount(published_scrollers);
    page.click("[role=combobox]");
    assert!(page.exists("[role=listbox]"), "{}", page.tree());
    assert_eq!(page.computed("[role=listbox]", PADDING), "0px");
}

/// Todo 1521: a capped Table pads its own scroller, not a scroller in one of its cells.
#[test]
fn a_tables_padding_stays_out_of_a_cells_scroller() {
    let mut page = mount(published_scrollers);
    assert_eq!(page.computed("#notes-0", PADDING), "0px");
    page.hover("#notes-0");
    page.wheel("#notes-0", 40.0);
    page.wait_for(|page| page.scroll_top("#notes-0") == 40.0);
    page.focus("#note-0-3");
    page.shift_tab();
    page.settle();
    assert!(page.is_focused("#note-0-2"), "{}", page.focus_owner());
    assert_eq!(
        page.scroll_top("#notes-0"),
        40.0,
        "the cell took the table's padding"
    );
}

fn sections() -> Element {
    rsx! {
        section { id: "first", height: "300px",
            Header { id: "banner", "First" }
        }
        div { id: "content", height: "3000px", "Content" }
    }
}

/// The banner sticks inside its parent only: that leaving takes it along.
#[test]
fn a_banner_leaves_with_its_parent() {
    let mut page = mount(sections);
    page.hover("#content");
    page.wheel("#content", 500.0);
    // The scroll lands first, the sticky move after it: wait for both.
    page.wait_for(|page| {
        top(page, "#first") < -100.0 && bottom(page, "#banner") == bottom(page, "#first")
    });
    assert_eq!(
        bottom(&page, "#banner"),
        bottom(&page, "#first"),
        "{}",
        page.tree()
    );
}
