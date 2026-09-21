//! `use_color_scheme` follows the window theme natively (todo 393).

use std::time::Duration;

use dioxus::prelude::*;
use e2e::native::{ColorScheme, mount, mount_in};
use libero::{
    hooks::{use_color_scheme, use_theme_set},
    theme::ThemeSet,
};

const SCHEME: &str = "#scheme";
// libero's re-read interval, and a margin for a loaded machine.
const TICK: Duration = Duration::from_millis(800);

fn app() -> Element {
    let scheme = use_color_scheme();
    rsx! {
        button { id: "scheme", onclick: move |_| scheme.toggle(), "{scheme.resolved().as_str()}" }
    }
}

/// A pinned scheme flips the root's attribute, as on the web, instead of
/// rebuilding the theme sheet (todo 480).
#[test]
fn a_pinned_scheme_flips_the_root_attribute() {
    let mut page = mount(app);
    let light = page.computed("#scheme", "--lsx-ink");
    page.click(SCHEME);
    assert_eq!(page.text(SCHEME), "dark");
    assert_eq!(page.attr("html", "data-lsx-theme").as_deref(), Some("dark"));
    assert_ne!(page.computed("#scheme", "--lsx-ink"), light);

    page.click(SCHEME);
    assert_eq!(page.attr("html", "data-lsx-theme"), None);
    assert_eq!(page.computed("#scheme", "--lsx-ink"), light);
}

/// Todo 635: libero kept the first document it saw in a `thread_local`, so a
/// second mount on the thread flipped the attribute on the first one's root.
#[test]
fn a_second_mount_on_the_thread_flips_its_own_root() {
    drop(mount(app));
    let mut page = mount(app);
    page.click(SCHEME);
    assert_eq!(page.text(SCHEME), "dark");
    assert_eq!(page.attr("html", "data-lsx-theme").as_deref(), Some("dark"));
}

/// Todo 702: two live documents on one thread each keep their own state, so
/// the older one's toggle and theme switch reach its own root and app.
#[test]
fn two_live_mounts_on_the_thread_keep_their_own_scheme() {
    let mut older = mount(app);
    let mut newer = mount(app);
    older.click(SCHEME);
    assert_eq!(
        older.attr("html", "data-lsx-theme").as_deref(),
        Some("dark")
    );
    assert_eq!(newer.attr("html", "data-lsx-theme"), None);

    older.click(SCHEME);
    assert_eq!(older.attr("html", "data-lsx-theme"), None);

    older.set_color_scheme(ColorScheme::Dark);
    older.wait(TICK);
    assert_eq!(older.text(SCHEME), "dark");
    newer.wait(TICK);
    assert_eq!(newer.text(SCHEME), "light");
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

fn swap_app() -> Element {
    let themes = use_theme_set();
    rsx! {
        button { id: "swap", onclick: move |_| themes.set(ThemeSet::DRACULA.clone()), "Swap" }
    }
}

/// Todo 871: a set swap rebuilds the theme sheet in the click's focus frame. Its `:root` rules
/// invalidate stylo fully, so todo 837's panic cannot occur; `SheetWatch` guards it anyway.
#[test]
fn a_theme_set_swap_in_a_focusing_click_restyles() {
    let mut page = mount(swap_app);
    let ink = page.computed("#swap", "--lsx-ink");
    page.click("#swap");
    assert!(
        page.is_focused("#swap"),
        "focus is on {}",
        page.focus_owner()
    );
    assert_ne!(page.computed("#swap", "--lsx-ink"), ink);
}
