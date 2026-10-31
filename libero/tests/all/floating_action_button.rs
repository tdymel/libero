//! `FloatingActionButton`'s rendered contract: a fixed `Float` at the header layer
//! around an `ActionIcon`, or an extended `Button` with `children`, held off the
//! edges by logical margins and lifted by a shadow that leaves the focus ring alone.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{FloatingActionButton, FloatingActionButtonAction},
};

fn icon_only() -> Element {
    rsx! {
        LiberoProvider {
            FloatingActionButton { aria_label: "New message", icon: rsx! { svg {} } }
        }
    }
}

fn extended() -> Element {
    rsx! {
        LiberoProvider {
            FloatingActionButton { icon: rsx! { svg {} }, "Compose" }
        }
    }
}

fn tokens(html: &str, tag: &str) -> Vec<String> {
    attributes_of(&body(html), tag)["data-state"]
        .split(' ')
        .map(str::to_string)
        .collect()
}

#[test]
fn icon_only_is_a_named_button_in_a_fixed_bottom_end_float() {
    let html = render(icon_only);
    assert_eq!(
        attributes_of(&body(&html), "button")["aria-label"],
        "New message"
    );
    let float = tokens(&html, "div");
    for token in ["fixed", "vertical-bottom", "horizontal-end"] {
        assert!(float.iter().any(|t| t == token), "{float:?}");
    }
    let button = tokens(&html, "button");
    assert!(!button.iter().any(|t| t == "extended"), "{button:?}");
    // The `xl` box grows to 44px under a coarse pointer, as an `md`+ ActionIcon does.
    assert!(button.iter().any(|t| t == "grows"), "{button:?}");
}

#[test]
fn children_render_an_extended_button_named_by_its_label() {
    let html = render(extended);
    let button = attributes_of(&body(&html), "button");
    assert!(!button.contains_key("aria-label"), "{html}");
    assert!(body(&html).contains("Compose"), "{html}");
    assert!(
        tokens(&html, "button").iter().any(|t| t == "extended"),
        "{html}"
    );
    assert!(
        html.contains(":where([data-slot='button-icon'] > svg){width:"),
        "the icon is sized: {html}"
    );
}

#[test]
fn it_sits_at_the_header_layer_and_clears_a_docked_bar() {
    let html = render(icon_only);
    assert!(
        html.contains("--lsx-z-index-float-override:var(--lsx-z-index-header)"),
        "{html}"
    );
    assert!(
        html.contains("margin-block-end:calc(var(--lsx-navigation-bar-height, env(safe-area-inset-bottom, 0px)) + var(--lsx-spacing-md))"),
        "{html}"
    );
    assert!(
        html.contains("margin-inline-end:var(--lsx-spacing-md)"),
        "{html}"
    );
}

/// The shadow skips `:focus-visible`, where the ring folds it in through `--lsx-own-shadow`.
#[test]
fn the_shadow_leaves_the_focus_ring_to_the_button() {
    let html = render(icon_only);
    assert!(
        html.contains(":where(:not(:focus-visible)){box-shadow:var(--lsx-shadow-md);}"),
        "{html}"
    );
    assert!(
        html.contains("--lsx-own-shadow:var(--lsx-shadow-md)"),
        "{html}"
    );
}

#[test]
fn contained_it_drops_fixed_and_the_bar_clearance() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FloatingActionButton { aria_label: "Add", fixed: false, placement: "top-start", offset: "8px", icon: rsx! { svg {} } }
            }
        }
    }
    let html = render(app);
    let float = tokens(&html, "div");
    assert!(!float.iter().any(|t| t == "fixed"), "{float:?}");
    assert!(float.iter().any(|t| t == "horizontal-start"), "{float:?}");
    assert!(html.contains("margin-block-start:8px"), "{html}");
    assert!(html.contains("margin-inline-start:8px"), "{html}");
    assert!(!html.contains("navigation-bar-height"), "{html}");
}

#[test]
fn a_fixed_top_one_clears_the_top_safe_area_only() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FloatingActionButton { aria_label: "Add", placement: "top-center", offset: "8px", icon: rsx! { svg {} } }
            }
        }
    }
    let html = render(app);
    assert!(
        html.contains("margin-block-start:calc(env(safe-area-inset-top, 0px) + 8px)"),
        "{html}"
    );
    assert!(!html.contains("margin-inline"), "{html}");
}

#[test]
fn hidden_fades_it_out_of_the_tab_order_and_the_tree() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FloatingActionButton { aria_label: "Add", hidden: true, icon: rsx! { svg {} } }
            }
        }
    }
    let html = render(app);
    assert!(
        attributes_of(&body(&html), "div").contains_key("inert"),
        "{html}"
    );
    assert!(html.contains("visibility:hidden"), "{html}");
    assert!(html.contains("scale:0.6"), "{html}");
    assert!(
        html.contains("visibility 0s linear var(--lsx-transition-duration)"),
        "visibility flips after the fade: {html}"
    );
    assert!(html.contains("prefers-reduced-motion"), "{html}");
    assert!(!attributes_of(&body(&render(icon_only)), "div").contains_key("inert"));
}

fn speed_dial(opened: bool) -> Element {
    let open = use_signal(|| opened);
    rsx! {
        LiberoProvider {
            FloatingActionButton {
                aria_label: "Create",
                open,
                to: "/ignored",
                icon: rsx! { svg {} },
                actions: rsx! {
                    FloatingActionButtonAction { label: "Message", icon: rsx! { svg {} } }
                    FloatingActionButtonAction { label: "Event", icon: rsx! { svg {} } }
                },
            }
        }
    }
}

#[test]
fn a_closed_speed_dial_is_a_collapsed_button_without_its_actions() {
    let html = body(&render(|| speed_dial(false)));
    let button = attributes_of(&html, "button");
    assert_eq!(button["aria-expanded"], "false", "{html}");
    assert!(!button.contains_key("aria-controls"), "{html}");
    assert!(!html.contains("Message"), "{html}");
    assert!(!html.contains("<a"), "`to` is ignored with actions: {html}");
}

#[test]
fn an_open_speed_dial_controls_the_actions_after_it() {
    let html = body(&render(|| speed_dial(true)));
    let button = attributes_of(&html, "button");
    assert_eq!(button["aria-expanded"], "true", "{html}");
    let list = format!("id=\"{}\"", button["aria-controls"]);
    let at = html
        .find(&list)
        .unwrap_or_else(|| panic!("no list: {html}"));
    let at = html[..at].rfind("<div").unwrap();
    let group = format!("aria-labelledby=\"{}\"", button["id"]);
    assert!(html[at..].contains(&group), "named by the button: {html}");
    assert!(html[at..].contains("role=\"group\""), "{html}");
    assert!(
        at > html.find("aria-expanded").unwrap(),
        "the list follows the button: {html}"
    );
    assert!(
        html[at..].contains("Message") && html[at..].contains("Event"),
        "{html}"
    );
}

#[test]
fn to_renders_a_link() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FloatingActionButton { aria_label: "Help", to: "https://libero-ui.dev", icon: rsx! { svg {} } }
            }
        }
    }
    let html = body(&render(app));
    assert_eq!(attributes_of(&html, "a")["aria-label"], "Help", "{html}");
    assert!(!html.contains("<button"), "{html}");
}
