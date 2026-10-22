//! `Marquee`: Tab skips the `inert` copies of the children (todo 1501).

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::Marquee;

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
