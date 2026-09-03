//! `Float`'s `fixed` arm: one state token that swaps `position`, and nothing
//! else about the placement changes.

mod common;

use common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Float};

fn fixed_app() -> Element {
    rsx! {
        LiberoProvider {
            Float { fixed: true, placement: "bottom-center", "bar" }
        }
    }
}

fn anchored_app() -> Element {
    rsx! {
        LiberoProvider {
            Float { placement: "bottom-center", "bar" }
        }
    }
}

fn tokens(html: &str) -> Vec<String> {
    attributes_of(&body(html), "div")["data-state"]
        .split(' ')
        .map(str::to_string)
        .collect()
}

#[test]
fn fixed_adds_its_token_beside_the_placement() {
    let tokens = tokens(&render(fixed_app));
    for token in ["fixed", "vertical-bottom", "horizontal-center"] {
        assert!(tokens.iter().any(|t| t == token), "{tokens:?}");
    }
}

#[test]
fn a_float_is_not_fixed_by_default() {
    let tokens = tokens(&render(anchored_app));
    assert!(!tokens.iter().any(|t| t == "fixed"), "{tokens:?}");
}

/// The token is only useful if a rule keys on it, and it has to outrank the
/// base `position: absolute`: a `when` block is 0-2-0 against the base 0-1-0.
#[test]
fn the_fixed_token_switches_position_over_the_base() {
    let html = render(fixed_app);
    let class = attributes_of(&body(&html), "div")["class"]
        .split(' ')
        .next()
        .unwrap()
        .to_string();
    assert!(
        html.contains(&format!(".{class}{{position:absolute;")),
        "{html}"
    );
    assert!(
        html.contains(&format!(".{class}[data-state~=\"fixed\"]{{position:fixed;")),
        "{html}"
    );
}
