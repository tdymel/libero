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
