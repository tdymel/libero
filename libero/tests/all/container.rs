use crate::common::{CssRule, body, css_rules_for, render, tag_with};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Center, Container},
};

#[test]
fn container_and_center_render_their_children() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Container {
                    Center { inline: true, "centred" }
                }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("centred"));
    assert!(body(&html).contains("--lsx-center-display-override:inline-flex;"));
}

fn container_rules() -> Vec<CssRule> {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Container { id: "page", tabindex: "-1", "content" }
            }
        }
    }

    let html = render(app);
    css_rules_for(&html, &tag_with(&html, r#"id="page""#))
}

#[test]
fn a_container_sets_no_height() {
    // A forced `height: 100%` squeezes siblings in a sized column (todo 1314).
    assert!(
        container_rules()
            .iter()
            .all(|rule| !rule.declarations.contains_key("height"))
    );
}

#[test]
fn a_focused_container_draws_its_ring_inset() {
    // A full-width container's outer ring is clipped at the viewport sides (todo 1315).
    // Every matching ring: the box's outer one would draw too.
    let rules = container_rules();
    let rings: Vec<_> = rules
        .iter()
        .filter(|rule| rule.selector.contains(":focus-visible"))
        .filter_map(|rule| rule.declarations.get("box-shadow"))
        .collect();
    assert!(!rings.is_empty(), "a focus-visible ring");
    assert!(
        rings.iter().all(|ring| ring.starts_with("inset ")),
        "{rings:?}"
    );
}
