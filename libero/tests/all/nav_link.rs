use crate::common::{attributes_of, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::NavLink};

#[test]
fn nav_link_renders_a_link_with_its_active_background() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NavLink { to: "https://example.com", active: true, "Docs" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "a");

    assert_eq!(attributes["href"], "https://example.com");
    assert!(attributes["style"].contains("--lsx-nav-link-active-background"));
    assert!(attributes["data-state"].contains("active"));
    assert_eq!(attributes["aria-current"], "page");
}

/// Disabling the current page's link still marks it as the current page.
#[test]
fn an_active_disabled_nav_link_keeps_aria_current() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NavLink { to: "https://example.com", active: true, disabled: true, "Docs" }
            }
        }
    }

    let attributes = attributes_of(&render(app), "a");

    assert_eq!(attributes["aria-current"], "page");
    assert_eq!(attributes["aria-disabled"], "true");
}

/// The description is the link's description, not part of its name (580).
#[test]
fn a_description_describes_the_link() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NavLink { to: "/docs", description: "Guides and API", "aria-describedby": "hint", "Docs" }
            }
        }
    }

    let html = render(app);
    let link = attributes_of(&html, "a");
    let (caller, ours) = link["aria-describedby"].split_once(' ').unwrap();

    assert_eq!(caller, "hint");
    assert!(html.contains(&format!(r#"id="{ours}""#)), "{html}");
    assert!(html.contains(r#"aria-hidden="true""#), "{html}");
    assert!(html.contains("Guides and API"));
}

/// APG disclosure: a button beside the link, named after it, controlling the
/// nested links' panel.
#[test]
fn nested_links_sit_behind_a_disclosure_button() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NavLink { to: "/docs", id: "docs",
                    nested: rsx! { NavLink { to: "/docs/install", "Install" } },
                    "Docs"
                }
            }
        }
    }

    let html = render(app);
    let button = attributes_of(&html, "button");
    let panel = &button["aria-controls"];

    assert_eq!(button["aria-expanded"], "false");
    assert_eq!(button["aria-label"], "Show links");
    assert_eq!(button["aria-labelledby"], format!("{} docs", button["id"]));
    assert!(html.contains(&format!(r#"id="{panel}""#)), "{html}");
    assert_eq!(attributes_of(&html, "a")["href"], "/docs");
}

#[test]
fn default_opened_and_opened_expand_the_nested_links() {
    fn uncontrolled() -> Element {
        rsx! {
            LiberoProvider {
                NavLink { to: "/docs", default_opened: true, nested: rsx! { "x" }, "Docs" }
            }
        }
    }
    fn controlled() -> Element {
        rsx! {
            LiberoProvider {
                NavLink { to: "/docs", default_opened: true, opened: false, nested: rsx! { "x" }, "Docs" }
            }
        }
    }

    let expanded =
        |app: fn() -> Element| attributes_of(&render(app), "button")["aria-expanded"].clone();
    assert_eq!(expanded(uncontrolled), "true");
    assert_eq!(expanded(controlled), "false");
}
