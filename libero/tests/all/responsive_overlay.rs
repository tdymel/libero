//! Responsive overlay behaviour (todo 5657): Dialog `full_screen` per breakpoint in CSS, Drawer
//! `anchor` and popover sides in Rust, base on the server.

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Dialog, DialogFullScreen, DrawerAnchor, Shortcut, ShortcutHelp},
    hooks::{DrawerOptions, ModalScope, use_drawer, use_modal},
    theme::responsive,
};

const UNDER_MD: &str = "(max-width: 61.99875rem)";

/// The `data-state` of the dialog named `name`.
fn dialog_state<'a>(html: &'a str, name: &str) -> &'a str {
    let tag = html
        .split("<div")
        .find(|tag| {
            let head = tag.split('>').next().unwrap_or_default();
            head.contains("role=\"dialog\"") && head.contains(&format!("aria-label=\"{name}\""))
        })
        .unwrap_or_else(|| panic!("no dialog {name}: {html}"));
    tag.split("data-state=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .filter(|_| {
            tag.split('>')
                .next()
                .unwrap_or_default()
                .contains("data-state")
        })
        .unwrap_or("")
}

#[test]
fn the_three_full_screen_words_stay_as_they_were() {
    #[component]
    fn Opener() -> Element {
        let modal = use_modal(|_: ModalScope<()>| {
            rsx! {
                Dialog { aria_label: "Phone", full_screen: "phone", "one" }
                Dialog { aria_label: "Enum", full_screen: DialogFullScreen::Always, "two" }
                Dialog { aria_label: "Never", full_screen: false, "three" }
                Dialog { aria_label: "Shaped", full_screen: responsive(true).sm(false), "four" }
            }
        });
        use_hook(move || modal.open());
        rsx! {}
    }
    fn app() -> Element {
        rsx! { LiberoProvider { Opener {} } }
    }

    let html = render(app);
    assert_eq!(dialog_state(&html, "Phone"), "full-screen-phone");
    assert_eq!(dialog_state(&html, "Enum"), "full-screen-always");
    assert_eq!(dialog_state(&html, "Never"), "");
    assert_eq!(dialog_state(&html, "Shaped"), "full-screen-phone");
    assert!(
        !html.contains("[data-state~=\"base-"),
        "no range copy: {html}"
    );
}

#[test]
fn a_custom_full_screen_shape_fills_the_screen_per_range() {
    #[component]
    fn Opener() -> Element {
        let modal = use_modal(|_: ModalScope<()>| {
            rsx! {
                Dialog { aria_label: "Custom", full_screen: responsive(true).md(false), "one" }
                ShortcutHelp { full_screen: responsive(true).md(false), shortcuts: vec![Shortcut::new("mod+b", "Bold")] }
            }
        });
        use_hook(move || modal.open());
        rsx! {}
    }
    fn app() -> Element {
        rsx! { LiberoProvider { Opener {} } }
    }

    let html = render(app);
    assert_eq!(
        dialog_state(&html, "Custom"),
        "base-md:full-screen md-:centred"
    );
    let page = body(&html);
    assert!(!page.contains("full-screen-phone"), "{page}");
    // One copy for the dialog and one for the shortcut list, under `md`; `centred` has no rule.
    let copies: Vec<_> = html
        .split("<style")
        .filter(|sheet| sheet.contains("[data-state~=\"base-md:full-screen\"]"))
        .collect();
    assert_eq!(copies.len(), 2, "{html}");
    assert!(copies.iter().all(|copy| copy.contains(UNDER_MD)), "{html}");
    assert!(
        copies.iter().any(|copy| copy.contains("border-radius:0")),
        "{html}"
    );
    assert!(
        copies.iter().any(|copy| copy.contains("max-height:none")),
        "{html}"
    );
    assert!(!html.contains("[data-state~=\"md-:centred\"]"), "{html}");
}

#[test]
fn a_responsive_drawer_anchor_renders_its_base_on_the_server() {
    #[component]
    fn Opener() -> Element {
        let drawer = use_drawer(
            DrawerOptions {
                anchor: responsive(DrawerAnchor::Bottom)
                    .md(DrawerAnchor::End)
                    .into(),
                aria_label: Some("Panel".into()),
                ..Default::default()
            },
            |_: ModalScope<()>| rsx! { "drawer content" },
        );
        use_hook(move || drawer.open());
        rsx! {}
    }
    fn app() -> Element {
        rsx! { LiberoProvider { Opener {} } }
    }

    let html = render(app);
    let state = dialog_state(&html, "Panel");
    assert!(state.contains("anchor-bottom"), "{state}");
    assert!(
        body(&html).contains("data-swipe-handle"),
        "the base's strip"
    );
}
