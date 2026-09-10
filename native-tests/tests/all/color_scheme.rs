//! `use_color_scheme` follows the window theme natively (todo 393).

use dioxus::prelude::*;
use libero::hooks::use_color_scheme;
use native_tests::{ColorScheme, Key, mount, mount_in};

const SCHEME: &str = "#scheme";

fn app() -> Element {
    let scheme = use_color_scheme();
    rsx! {
        button { id: "scheme", "{scheme.resolved().as_str()}" }
    }
}

#[test]
fn a_dark_window_resolves_dark_from_the_first_settled_frame() {
    let page = mount_in(app, ColorScheme::Dark);
    assert_eq!(page.text(SCHEME), "dark");
}

#[test]
fn a_light_window_resolves_light() {
    let page = mount(app);
    assert_eq!(page.text(SCHEME), "light");
}

/// Pins a gap: Blitz sends no theme-change event, and libero reads the scheme
/// once at mount. Flips once Blitz notifies.
#[test]
fn a_live_theme_change_does_not_reach_rust() {
    let mut page = mount(app);
    page.set_color_scheme(ColorScheme::Dark);
    page.click(SCHEME);
    page.press(Key::Tab);
    assert_eq!(page.text(SCHEME), "light");
}
