//! `Marquee`: Tab skips the `inert` copies of the children (todo 1501).

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{Anchor, Button, Flex, Marquee};
use libero::sx::sx;

fn app() -> Element {
    rsx! {
        Marquee { button { "Item" } }
        button { id: "after", "After" }
    }
}

#[test]
fn tab_skips_the_inert_copies() {
    let mut page = mount(app);
    let copies = "[data-slot=group][inert] button";
    assert!(page.exists(copies), "no inert copy:\n{}", page.tree());
    for _ in 0..6 {
        page.tab();
        assert!(!page.is_focused(copies), "Tab entered a copy");
        if page.is_focused("#after") {
            return;
        }
    }
    panic!("Tab never reached #after, at {}", page.focus_owner());
}

fn linked() -> Element {
    rsx! {
        // The docs preview's column: Blitz clipped the ring's top and bottom there.
        div { style: "padding: 40px",
            Flex { direction: "column", gap: "md", sx: sx().width("100%").min_width("0"),
                Button { "Pause the ticker" }
                Marquee {
                    Anchor { id: "rust", to: "https://www.rust-lang.org", "Rust" }
                    Anchor { to: "https://dioxuslabs.com", "Dioxus" }
                }
            }
        }
        p { style: "padding: 20px", "Before ", Anchor { id: "inline", to: "https://a.example", "a link" }, " after" }
    }
}

/// The painted pixel 3px out from each edge's middle, on the ring's stripe: top, bottom, left, right.
fn ring(page: &Page, selector: &str) -> [String; 4] {
    let (x, y, width, height) = page.rect(selector);
    let (cx, cy) = ((x + width / 2.0) as u32, (y + height / 2.0) as u32);
    [
        (cx, (y - 3.0) as u32),
        (cx, (y + height + 2.0) as u32),
        ((x - 3.0) as u32, cy),
        ((x + width + 2.0) as u32, cy),
    ]
    .map(|(px, py)| page.painted_pixel(px, py))
}

fn tab_to(page: &mut Page, selector: &str) {
    for _ in 0..8 {
        if page.is_focused(selector) {
            return;
        }
        page.tab();
    }
    panic!("Tab never reached {selector}, at {}", page.focus_owner());
}

/// Todo 5203: the strip's root clips both axes on Blitz, so it pads the cross axis
/// while focus is inside; only the link's sides were drawn.
#[test]
fn a_focused_link_paints_its_whole_ring() {
    let mut page = mount(linked);
    tab_to(&mut page, "#rust");
    let stripe = page.computed("#rust", "outline-color");
    assert_eq!(ring(&page, "#rust"), [0; 4].map(|_| stripe.clone()));
}

#[test]
#[ignore = "5203: Blitz paints no outline or box-shadow around an inline box"]
fn a_focused_inline_link_paints_its_ring() {
    let mut page = mount(linked);
    tab_to(&mut page, "#inline");
    let stripe = page.computed("#inline", "outline-color");
    assert_eq!(ring(&page, "#inline"), [0; 4].map(|_| stripe.clone()));
}
