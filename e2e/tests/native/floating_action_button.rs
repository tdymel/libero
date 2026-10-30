//! `FloatingActionButton` natively (todo 5062): docked to the window's corner through a
//! scroll, its `env(safe-area-inset-*)` margins fall back to 0px, and it takes a click.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{FloatingActionButton, NavigationBar, NavigationBarItem};

fn page() -> Element {
    let mut count = use_signal(|| 0);
    rsx! {
        div { id: "content", height: "3000px", "Content" }
        FloatingActionButton {
            id: "fab",
            aria_label: "New message",
            icon: rsx! { "+" },
            onclick: move |_| count += 1,
        }
        FloatingActionButton { id: "top", aria_label: "Search", placement: "top-start", icon: rsx! { "S" } }
        span { id: "count", "{count}" }
    }
}

fn bar() -> Element {
    rsx! {
        div { id: "content", height: "3000px", "Content" }
        FloatingActionButton { id: "fab", icon: rsx! { "+" }, "Compose" }
        NavigationBar { id: "bar", "aria-label": "Main", position: "fixed",
            NavigationBarItem { selected: true, onclick: |_| {}, icon: rsx! { "H" }, "Home" }
            NavigationBarItem { onclick: |_| {}, icon: rsx! { "S" }, "Search" }
        }
    }
}

/// The gaps from the window's end and bottom edges, then from its start and top edges.
fn gaps(page: &Page) -> ((f64, f64), (f64, f64)) {
    let (width, height) = page.window_size();
    let (x, y, w, h) = page.rect("#fab");
    let (top_x, top_y, ..) = page.rect("#top");
    (
        (width as f64 - (x + w), height as f64 - (y + h)),
        (top_x, top_y),
    )
}

/// The bottom gap equals the end gap: `env(safe-area-inset-bottom, 0px)` resolves to its
/// fallback rather than voiding the `calc()`, and no line-box strut lifts the button.
#[test]
fn it_docks_to_the_windows_corner_with_the_safe_area_at_0px() {
    let mut page = mount(page);
    page.wait_for(|page| {
        let ((end, bottom), _) = gaps(page);
        end > 0.0 && (end - bottom).abs() < 1.0
    });
    let ((end, bottom), (start, top)) = gaps(&page);
    assert!(end > 0.0, "no offset: {}", page.tree());
    assert!(
        (end - bottom).abs() < 1.0,
        "end {end}, bottom {bottom}: {}",
        page.tree()
    );
    assert!(
        (start - end).abs() < 1.0 && (top - end).abs() < 1.0,
        "start {start}, top {top}, end {end}"
    );

    page.hover("#content");
    page.wheel("#content", 500.0);
    page.wait_for(|page| page.rect("#content").1 < -100.0 && (gaps(page).0.1 - end).abs() < 1.0);
    let ((_, bottom), (_, top)) = gaps(&page);
    assert!(
        (bottom - end).abs() < 1.0,
        "scrolled: bottom {bottom}, {}",
        page.tree()
    );
    assert!((top - end).abs() < 1.0, "scrolled: top {top}");
}

#[test]
fn a_click_at_its_centre_reaches_it() {
    let mut page = mount(page);
    let (x, y, w, h) = page.rect("#fab");
    page.click_at((x + w / 2.0) as f32, (y + h / 2.0) as f32);
    page.wait_for(|page| page.text("#count") == "1");
    assert_eq!(page.text("#count"), "1", "{}", page.tree());
}

/// A fixed bar publishes its height, and the button keeps its offset above the bar.
#[test]
fn it_sits_its_offset_above_a_fixed_navigation_bar() {
    let mut page = mount(bar);
    let window = page.window_size();
    let clear = |page: &Page| {
        let (x, y, w, h) = page.rect("#fab");
        let top = page.rect("#bar").1;
        (window.0 as f64 - (x + w), top - (y + h))
    };
    page.wait_for(|page| {
        let (end, above) = clear(page);
        end > 0.0 && (end - above).abs() < 1.0
    });
    let (end, above) = clear(&page);
    assert!(
        (end - above).abs() < 1.0,
        "end {end}, above the bar {above}: {}",
        page.tree()
    );
}
