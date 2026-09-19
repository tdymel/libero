//! Blitz renders inline svg without resolving `var()`: a theme colour in a
//! `fill` painted every module and the quiet zone black. The harness paints
//! no svg, so this reads what the renderer gets.

use dioxus::prelude::*;
use e2e::native::mount;
use libero::{components::QrCode, sx::sx};

fn app() -> Element {
    rsx! {
        QrCode { id: "qr", data: "https://example.com", aria_label: "Example", sx: sx().width("160px") }
    }
}

#[test]
fn its_svg_takes_the_theme_colours_without_var() {
    let page = mount(app);
    assert_eq!(
        page.attr("#qr path", "fill").as_deref(),
        Some("currentColor")
    );
    assert_eq!(page.attr("#qr rect", "fill").as_deref(), Some("none"));
    assert_eq!(page.computed("#qr svg", "color"), "rgb(0, 0, 0)");
    assert_eq!(
        page.computed("#qr", "background-color"),
        "rgb(255, 255, 255)"
    );
}
