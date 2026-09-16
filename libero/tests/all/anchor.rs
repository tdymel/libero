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
    assert!(html.contains("data-anchor-new-tab"), "{html}");
    assert!(html.contains("<svg"), "{html}");
    for app in [opted_out as fn() -> Element, same_tab] {
        let html = body(&render(app));
        assert!(!html.contains("new tab"), "{html}");
        assert!(!html.contains("<svg"), "{html}");
    }
}
