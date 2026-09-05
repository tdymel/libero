//! The `default` tier renders its own element without `full-polymorphism`.
//! Everything outside the tier falls back to `<div>`, so a tag that is meant
//! to be usable on default features has to be asserted on default features.

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Box};

#[test]
fn the_default_tier_renders_its_own_element() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Box { component: "nav", "nav" }
                Box { component: "ol", "ol" }
                Box { component: "blockquote", "blockquote" }
                Box { component: "cite", "cite" }
                Box { component: "figure", "figure" }
                Box { component: "figcaption", "figcaption" }
            }
        }
    }

    let html = body(&render(app));

    for tag in ["nav", "ol", "blockquote", "cite", "figure", "figcaption"] {
        assert!(
            html.contains(&format!("<{tag} ")),
            "<{tag}> missing in {html}"
        );
    }
}

/// Todo 278: `Box { component: "footer" }` used to render a `<div>`, and in a
/// release build it did so silently. These are the semantic tags the default
/// tier grew on 2026-09-19, one per family, so a regression that pushed any
/// of them back behind `full-polymorphism` fails here.
#[test]
fn the_promoted_semantic_tags_render_on_default_features() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Box { component: "footer", "footer" }
                Box { component: "address", "address" }
                Box { component: "strong", "strong" }
                Box { component: "em", "em" }
                Box { component: "small", "small" }
                Box { component: "time", "time" }
                Box { component: "abbr", "abbr" }
                Box { component: "del", "del" }
                Box { component: "details", "details" }
                Box { component: "summary", "summary" }
                Box { component: "dialog", "dialog" }
                Box { component: "progress" }
                Box { component: "hr" }
            }
        }
    }

    let html = body(&render(app));

    for tag in [
        "footer", "address", "strong", "em", "small", "time", "abbr", "del", "details", "summary",
        "dialog", "progress", "hr",
    ] {
        assert!(
            html.contains(&format!("<{tag} ")),
            "<{tag}> missing in {html}"
        );
    }
}
