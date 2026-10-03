use crate::common::{body, drive_once, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Header, Paper},
};

/// Turning `glass` on at runtime keeps the hook order: the release docs demo panicked
/// (wasm `unreachable`) on its Glass switch (todo 2134).
#[test]
fn toggling_glass_at_runtime_renders() {
    fn app() -> Element {
        let mut glass = use_signal(|| false);
        use_hook(|| spawn(async move { glass.set(true) }));
        rsx! {
            LiberoProvider {
                Header { glass: glass(), "site header" }
                Paper { glass: glass(), "card" }
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let html = drive_once(&mut dom);

    assert_eq!(html.matches(r#"data-state="glass""#).count(), 2, "{html}");
}

#[test]
fn header_renders_as_a_banner_landmark() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Header { "site header" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("<header"));
    assert!(body(&html).contains("site header"));
}

/// An uncoloured header is the surface, and says so through the one token
/// that spells it rather than a `white` of its own.
#[test]
fn an_uncoloured_header_falls_back_to_the_surface() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Header { "site header" } }
        }
    }

    let html = render(app);

    assert!(
        html.contains("background:var(--lsx-header-background, var(--lsx-paper-background));"),
        "{html}"
    );
}

/// Only an opted-in banner puts its height and scroll padding on `:root`, so a
/// Header shown elsewhere (a demo, a card) leaves the page alone.
#[test]
fn only_publish_height_puts_the_height_on_the_root() {
    fn quiet() -> Element {
        rsx! {
            LiberoProvider { Header { "demo" } }
        }
    }
    fn banner() -> Element {
        rsx! {
            LiberoProvider { Header { publish_height: true, "site header" } }
        }
    }

    assert!(!render(quiet).contains(":root[data-lsx-header="));
    let html = render(banner);
    assert!(html.contains(":root[data-lsx-header="), "{html}");
    assert!(
        html.contains("scroll-padding-top:var(--lsx-header-height)"),
        "{html}"
    );
}
