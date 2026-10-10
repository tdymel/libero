use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Anchor};

#[test]
fn an_external_anchor_renders_a_plain_link() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Anchor { to: "https://example.com", "Example" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "a");

    assert_eq!(attributes["href"], "https://example.com");
    assert!(body(&html).contains("Example"));
}

/// Todo 579: a new tab is announced and drawn, and the caller can opt out.
#[test]
fn a_blank_target_adds_the_new_tab_hint_unless_opted_out() {
    fn blank() -> Element {
        rsx! {
            LiberoProvider {
                Anchor { to: "https://example.com", target: "_blank", "Example" }
            }
        }
    }
    fn opted_out() -> Element {
        rsx! {
            LiberoProvider {
                Anchor { to: "https://example.com", target: "_blank", new_tab_hint: false, "Example" }
            }
        }
    }
    fn same_tab() -> Element {
        rsx! {
            LiberoProvider {
                Anchor { to: "https://example.com", "Example" }
            }
        }
    }

    let html = body(&render(blank));
    assert!(html.contains("(opens in a new tab)"), "{html}");
    assert!(html.contains("data-slot=\"new-tab\""), "{html}");
    assert!(html.contains("<svg"), "{html}");
    for app in [opted_out as fn() -> Element, same_tab] {
        let html = body(&render(app));
        assert!(!html.contains("new tab"), "{html}");
        assert!(!html.contains("<svg"), "{html}");
    }
}

/// Todo 3009: the target keyword matches case-insensitively, as in HTML.
#[test]
fn an_uppercase_blank_target_adds_the_new_tab_hint() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Anchor { to: "https://example.com", target: "_BLANK", "Example" }
            }
        }
    }

    let html = body(&render(app));
    assert!(html.contains("(opens in a new tab)"), "{html}");
}

#[derive(Clone, PartialEq, Routable)]
enum Route {
    #[route("/")]
    Home {},
}

#[component]
fn Home() -> Element {
    rsx! {
        LiberoProvider {
            Anchor { to: "/elsewhere", target: "_BLANK", "Elsewhere" }
        }
    }
}

/// Todo 3008: with a router mounted, an internal `_blank` link still opens a new tab.
#[test]
fn an_internal_blank_link_under_a_router_opens_a_new_tab() {
    fn app() -> Element {
        rsx! { Router::<Route> {} }
    }

    let html = body(&render(app));
    let attributes = attributes_of(&html, "a");

    assert_eq!(attributes["href"], "/elsewhere", "{html}");
    assert_eq!(attributes["target"], "_blank", "{html}");
    assert!(html.contains("(opens in a new tab)"), "{html}");
}
