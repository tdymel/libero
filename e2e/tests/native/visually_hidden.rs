//! A focused skip link natively: it paints above a sticky `Header` (2.4.11).

use dioxus::prelude::*;
use e2e::native::mount;
use libero::{
    components::{Header, VisuallyHidden},
    sx::sx,
};

fn skip_link_under_banner() -> Element {
    rsx! {
        VisuallyHidden { focusable: true,
            a { id: "skip-link", href: "#main", "Skip to content" }
        }
        Header { id: "banner", sx: sx().background("rgb(255, 0, 0)"), "Libero" }
        main { id: "main", tabindex: "-1", "Content" }
    }
}

#[test]
fn a_focused_skip_link_paints_over_the_header() {
    let mut page = mount(skip_link_under_banner);
    page.tab();
    page.wait_for(|page| page.rect("#skip-link").2 > 20.0);
    let (x, y, width, height) = page.rect("#skip-link");
    let (x, y) = ((x + width / 2.0) as u32, (y + height / 2.0) as u32);
    assert_ne!(
        page.painted_pixel(x, y),
        "rgb(255, 0, 0)",
        "{}",
        page.tree()
    );
}
