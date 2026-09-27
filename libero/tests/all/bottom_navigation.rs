use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{BottomNavigation, BottomNavigationItem},
};

#[test]
fn the_bar_is_a_named_nav_and_the_selected_link_is_the_current_page() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                BottomNavigation { "aria-label": "Main",
                    BottomNavigationItem { to: "https://example.com/home", selected: true, icon: rsx! { "H" }, "Home" }
                    BottomNavigationItem { to: "https://example.com/search", icon: rsx! { "S" }, "Search" }
                }
            }
        }
    }

    let html = body(&render(app));
    let nav = attributes_of(&html, "nav");
    assert_eq!(nav["aria-label"], "Main");
    let home = attributes_of(&html, "a");
    assert_eq!(home["href"], "https://example.com/home");
    assert_eq!(home["aria-current"], "page");
    assert!(home["data-state"].contains("selected"));
    assert_eq!(html.matches("aria-current").count(), 1, "{html}");
    assert!(
        html.contains("<span data-slot=\"icon\" aria-hidden=\"true\">"),
        "{html}"
    );
}

#[test]
fn an_item_without_to_is_a_button_and_a_disabled_one_is_disabled() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                BottomNavigation { "aria-label": "Main",
                    BottomNavigationItem { onclick: |_| {}, disabled: true, icon: rsx! { "H" }, "Home" }
                }
            }
        }
    }

    let attributes = attributes_of(&body(&render(app)), "button");
    assert_eq!(attributes["type"], "button");
    assert_eq!(attributes["disabled"], "true");
}

#[test]
fn a_docked_bar_states_its_position_and_label_visibility() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                BottomNavigation { "aria-label": "Main", position: "fixed", show_labels: "selected",
                    BottomNavigationItem { onclick: |_| {}, icon: rsx! { "H" }, "Home" }
                }
            }
        }
    }

    let nav = attributes_of(&body(&render(app)), "nav");
    assert!(nav["data-state"].contains("fixed"), "{nav:?}");
    assert!(nav["data-state"].contains("labels-selected"), "{nav:?}");
}
