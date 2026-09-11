//! `use_color_scheme` follows the window theme natively (todo 393).

use std::time::Duration;

use dioxus::prelude::*;
use libero::hooks::use_color_scheme;
use native_tests::{ColorScheme, mount, mount_in};

const SCHEME: &str = "#scheme";
// libero's re-read interval, and a margin for a loaded machine.
const TICK: Duration = Duration::from_millis(800);

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

/// Blitz sends no theme-change event, so libero re-reads the viewport on a
/// timer: a switch reaches Rust within half a second, with no input.
#[test]
fn a_live_theme_change_reaches_rust_without_input() {
    let mut page = mount(app);
    page.set_color_scheme(ColorScheme::Dark);
    page.wait(TICK);
    assert_eq!(page.text(SCHEME), "dark");

    page.set_color_scheme(ColorScheme::Light);
    page.wait(TICK);
    assert_eq!(page.text(SCHEME), "light");
}
