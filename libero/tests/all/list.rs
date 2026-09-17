use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{List, ListItem},
};

#[test]
fn list_renders_its_items() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                List {
                    ListItem { "one" }
                    ListItem { "two" }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("<ul"));
    assert_eq!(body.matches("<li").count(), 2);
}

/// `list-style: none` makes Safari with VoiceOver stop announcing a list, so
/// the role is explicit - and a caller's own still wins.
#[test]
fn the_list_keeps_its_semantics_through_an_explicit_role() {
    fn plain() -> Element {
        rsx! {
            LiberoProvider {
                List { ListItem { "one" } }
            }
        }
    }
    fn overridden() -> Element {
        rsx! {
            LiberoProvider {
                List { role: "presentation", ListItem { "one" } }
            }
        }
    }

    assert_eq!(attributes_of(&render(plain), "ul")["role"], "list");
    assert_eq!(
        attributes_of(&render(overridden), "ul")["role"],
        "presentation"
    );
}

/// Items whose order is the point get an `ol` with visible numbers (todo 743).
#[test]
fn an_ordered_list_is_a_numbered_ol() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                List { ordered: true,
                    ListItem { "one" }
                    ListItem { "two" }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("<ol"), "{body}");
    assert!(!body.contains("<ul"), "{body}");
    assert!(
        attributes_of(&html, "ol")["data-state"].contains("ordered"),
        "{body}"
    );
    assert!(
        html.contains(r#"[data-state~="ordered"]{list-style-type:decimal;"#),
        "{html}"
    );
}
