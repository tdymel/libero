//! Scrolling natively (todo 468 N3): `ScrollApi` hears wheels and libero's scrolls, an open
//! popover follows its anchor, and `scroll_into_view` scrolls the nearest scroller.

use std::{rc::Rc, time::Duration};

use dioxus::prelude::*;
use e2e::native::{Key, Page, mount};
use libero::components::{Button, Menu, MenuItem, NavLink, use_menu};
use libero::hooks::use_element;
use libero::platform::{ElementApi, scroll};

const BOX: &str = "#box";

// The buttons sit beside the box: Blitz hits its clipped content anywhere.
fn listening() -> Element {
    let heard = use_signal(|| 0u32);
    let _subscription = use_hook(|| {
        Rc::new(scroll().map(|api| {
            api.on_scroll(Box::new(move || {
                let mut heard = heard;
                let next = *heard.peek() + 1;
                heard.set(next);
            }))
        }))
    });
    let target = use_element();
    let scroller = use_element();
    let mut measured = use_signal(String::new);
    rsx! {
        div { style: "display: flex; align-items: flex-start;",
            div {
                id: "box",
                style: "width: 200px; height: 100px; overflow-y: auto;",
                onmounted: scroller.mount(),
                div { style: "height: 400px;" }
                div { id: "target", style: "height: 20px;", onmounted: target.mount(), "Target" }
                div { style: "height: 400px;" }
            }
            button { id: "heard", "{heard}" }
            button {
                id: "near",
                onclick: move |_| {
                    let _ = scroller.scroll_to(0.0, 50.0);
                },
                "Near"
            }
            button {
                id: "far",
                onclick: move |_| {
                    let _ = scroller.scroll_to(0.0, 700.0);
                },
                "Far"
            }
            button {
                id: "measure",
                onclick: move |_| {
                    let offset = scroller.client_offset();
                    spawn(async move {
                        if let Ok((x, y)) = offset.await {
                            measured.set(format!("{x},{y}"));
                        }
                    });
                },
                "{measured}"
            }
            button {
                id: "show",
                onclick: move |_| {
                    let _ = target.scroll_into_view(false);
                },
                "Show"
            }
        }
    }
}

/// No scroller round the target: the window scrolls (todo 1546).
fn plain_page() -> Element {
    let target = use_element();
    rsx! {
        button {
            id: "show",
            onclick: move |_| {
                let _ = target.scroll_into_view(false);
            },
            "Show"
        }
        div { style: "height: 2000px;" }
        div { id: "target", style: "height: 20px;", onmounted: target.mount(), "Target" }
        div { style: "height: 400px;" }
    }
}

#[test]
fn scroll_into_view_scrolls_the_window_without_a_scroller() {
    let mut page = mount(plain_page);
    assert_eq!(page.viewport_scroll().1, 0.0);
    page.click("#show");
    assert!(page.viewport_scroll().1 > 0.0, "the window did not scroll");
}

/// A scroller padded as a capped `Table` pads its scroll area, with a nested scroller in it.
fn padded() -> Element {
    rsx! {
        div {
            id: "outer",
            style: "height: 300px; overflow-y: auto; scroll-padding-top: 50px; --lsx-scroll-padding-top: 50px;",
            div { id: "inner", style: "height: 120px; overflow-y: auto;",
                for index in 0..10 {
                    a { key: "{index}", id: "inner-{index}", href: "#", style: "display: block; height: 40px;", "Inner {index}" }
                }
            }
            for index in 0..20 {
                a { key: "{index}", id: "row-{index}", href: "#", style: "display: block; height: 40px;", "Row {index}" }
            }
        }
    }
}

/// Todo 1521: a scroller's own padding var holds a focus scroll below it; a nested
/// scroller inherits the var but not the padding.
#[test]
fn a_scrollers_padding_var_is_its_own_not_its_nested_scrollers() {
    let mut page = mount(padded);
    page.hover("#outer");
    page.wheel("#outer", 400.0);
    page.wait_for(|page| page.scroll_top("#outer") == 400.0);
    // Row 7 sits at 120 + 280 = 400: at the outer box's top edge, inside its padding.
    page.focus("#row-8");
    page.shift_tab();
    page.settle();
    assert!(page.is_focused("#row-7"), "{}", page.focus_owner());
    assert_eq!(
        page.scroll_top("#outer"),
        350.0,
        "the outer box ignored its padding"
    );

    page.wheel("#outer", -400.0);
    page.wait_for(|page| page.scroll_top("#outer") == 0.0);
    page.hover("#inner");
    page.wheel("#inner", 80.0);
    page.wait_for(|page| page.scroll_top("#inner") == 80.0);
    page.focus("#inner-3");
    page.shift_tab();
    page.settle();
    assert!(page.is_focused("#inner-2"), "{}", page.focus_owner());
    assert_eq!(
        page.scroll_top("#inner"),
        80.0,
        "the inner box took the outer's padding"
    );
}

/// A link far below the first, with `--lsx-scroll-margin` on the link itself or on its wrapper.
fn margined(own: bool) -> Element {
    let margin = "--lsx-scroll-margin: 40px;";
    let link = if own { margin } else { "" };
    rsx! {
        div { id: "box", style: "height: 100px; overflow-y: auto;",
            a { id: "first", href: "#", style: "display: block; height: 20px;", "First" }
            div { style: "height: 300px;" }
            div { style: if own { "" } else { margin },
                a {
                    id: "far",
                    href: "#",
                    style: "display: block; height: 20px; {link}",
                    "Far"
                }
            }
            div { style: "height: 300px;" }
        }
    }
}

fn own_margin() -> Element {
    margined(true)
}

fn wrapper_margin() -> Element {
    margined(false)
}

/// Todo 1831: a scroll margin is the target's own, as `scroll-margin` is, not its descendants'.
#[test]
fn a_scroll_margin_var_holds_for_its_own_element_only() {
    for (app, margin) in [(own_margin as fn() -> Element, 40.0), (wrapper_margin, 0.0)] {
        let mut page = mount(app);
        page.focus("#first");
        page.tab();
        page.settle();
        assert!(page.is_focused("#far"), "{}", page.focus_owner());
        // The link's bottom, plus its margin, meets the box's bottom.
        assert_eq!(
            page.scroll_top("#box"),
            340.0 + margin - 100.0,
            "margin {margin}"
        );
    }
}

#[test]
fn a_wheel_tells_scroll_subscribers() {
    let mut page = mount(listening);
    assert_eq!(page.text("#heard"), "0");

    page.hover(BOX);
    page.wheel(BOX, 50.0);
    assert_eq!(page.scroll_top(BOX), 50.0);
    assert_ne!(page.text("#heard"), "0", "the wheel went unheard");
}

#[test]
fn scroll_to_sets_the_offset_and_tells_subscribers() {
    let mut page = mount(listening);
    page.click("#near");
    assert_eq!(page.scroll_top(BOX), 50.0);
    assert_ne!(page.text("#heard"), "0", "the scroll went unheard");
}

#[test]
fn scroll_into_view_scrolls_the_nearest_scroller_just_far_enough() {
    let mut page = mount(listening);
    page.click("#show");
    // From above: the target's bottom meets the box's.
    assert_eq!(page.scroll_top(BOX), 320.0);
    assert_ne!(page.text("#heard"), "0", "the scroll went unheard");

    // Already shown: a second call moves nothing.
    page.click("#show");
    assert_eq!(page.scroll_top(BOX), 320.0);

    // From below: the target's top meets the box's.
    page.click("#far");
    assert_eq!(page.scroll_top(BOX), 700.0);
    page.click("#show");
    assert_eq!(page.scroll_top(BOX), 400.0);
}

#[test]
fn a_scrolled_box_reports_where_its_box_is() {
    let mut page = mount(listening);
    page.click("#measure");
    let unscrolled = page.text("#measure");
    assert_eq!(unscrolled, "0,0");

    page.click("#far");
    page.click("#measure");
    assert_eq!(
        page.text("#measure"),
        unscrolled,
        "its scroll moved its box"
    );
}

fn above_a_scroller() -> Element {
    let scroller = use_element();
    rsx! {
        button {
            id: "far",
            onclick: move |_| {
                let _ = scroller.scroll_to(0.0, 700.0);
            },
            "Far"
        }
        div {
            id: "box",
            style: "height: 100px; overflow-y: auto;",
            onmounted: scroller.mount(),
            div { style: "height: 1000px;" }
        }
    }
}

/// Pins a gap: Blitz's hit test ignores `overflow` clipping, so a scrolled
/// box's hidden content takes the hits of what is drawn over it.
#[test]
#[ignore = "needs Blitz: hit testing ignores overflow clipping"]
fn a_scrolled_box_keeps_its_hidden_content_out_of_hits_above_it() {
    let mut page = mount(above_a_scroller);
    page.click("#far");
    page.click("#far");
    assert!(
        page.is_focused("#far"),
        "the click landed on {}",
        page.focus_owner()
    );
}

fn tall_page() -> Element {
    let mut clicks = use_signal(|| 0u32);
    rsx! {
        div { id: "spacer", style: "height: 1500px;" }
        button { id: "go", onclick: move |_| clicks += 1, "{clicks}" }
        div { style: "height: 1000px;" }
    }
}

/// Pointer events carry page coordinates, so a press still lands once the
/// window itself has scrolled (todo 655).
#[test]
fn a_click_lands_on_a_scrolled_page() {
    let mut page = mount(tall_page);
    page.hover("#spacer");
    page.wheel("#spacer", 1200.0);
    assert!(
        page.rect("#go").1 < 768.0,
        "the page did not scroll: {:?}",
        page.rect("#go")
    );
    assert!(page.hits("#go"));
    page.click("#go");
    assert_eq!(page.text("#go"), "1", "the click landed elsewhere");
}

fn sidebar() -> Element {
    rsx! {
        div { id: "box", style: "height: 400px; overflow-y: auto;",
            div { style: "height: 1000px;" }
            NavLink {
                id: "here",
                to: "https://example.com",
                active: true,
                scroll_into_view: true,
                "Here"
            }
            div { style: "height: 1000px;" }
        }
    }
}

#[test]
fn an_active_nav_link_scrolls_its_sidebar_on_mount() {
    let mut page = mount(sidebar);
    // Its effect runs before the first layout, so it waits a frame for one.
    page.wait(Duration::from_millis(100));
    let link = page
        .doc
        .inner
        .borrow()
        .get_client_bounding_rect(page.node("#here"))
        .unwrap();
    // Its bottom stops 8rem above the box's, as the web's `scroll-margin`
    // leaves it; servo's stylo has none, libero's var stands in (todo 789).
    assert_eq!(page.scroll_top(BOX), 1000.0 + link.height + 128.0 - 400.0);
}

fn menu_in_a_scroller() -> Element {
    let menu = use_menu();
    rsx! {
        div { id: "box", style: "height: 200px; overflow-y: auto;",
            div { style: "height: 50px;" }
            Menu {
                state: menu,
                items: vec![MenuItem::new("Copy").onselect(|_| {}).into()],
                Button { attributes: menu.a11y_attributes(), "Actions" }
            }
            div { style: "height: 1000px;" }
        }
    }
}

fn menu_top(page: &Page) -> String {
    page.computed("[role=menu]", "top")
}

#[test]
fn an_open_menu_follows_its_anchor_on_a_wheel() {
    let mut page = mount(menu_in_a_scroller);
    page.focus("[aria-haspopup]");
    page.press(Key::Enter);
    let before = menu_top(&page);

    page.hover(BOX);
    page.wheel(BOX, 30.0);
    assert_eq!(page.scroll_top(BOX), 30.0);
    assert_ne!(
        menu_top(&page),
        before,
        "the menu stayed where the anchor was"
    );
}
