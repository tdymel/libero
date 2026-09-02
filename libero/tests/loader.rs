//! `Loader`'s rendered contract: silent by default, a live region only when it
//! is given a label, and a reduced-motion arm that leaves every shape visible.

mod common;

use common::{attributes_of, body, classes_of, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Loader};

/// The loader's own framework class - the first one on its root.
fn loader_class(html: &str) -> String {
    classes_of(&body(html), "span")
        .first()
        .expect("a class on the loader")
        .clone()
}

/// The default is beside-its-own-text or inside a named control, so the loader
/// is silent: `aria-hidden`, no role. `role="presentation"` would be a no-op on
/// a `<span>`, which has no implicit role to strip.
#[test]
fn without_a_label_it_is_hidden_and_has_no_role() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Loader {} }
        }
    }

    let html = body(&render(app));
    let attributes = attributes_of(&html, "span");

    assert_eq!(attributes["aria-hidden"], "true", "{attributes:?}");
    assert!(!attributes.contains_key("role"), "{attributes:?}");
    assert!(
        html.contains(r#"data-state="oval size-md""#),
        "the theme defaults: {html}"
    );
    assert_eq!(
        html.matches("<span").count(),
        1,
        "oval takes no children: {html}"
    );
}

/// A live region announces its content, so the label is a text node inside the
/// root, not `aria-label`. And a live region must not also be `aria-hidden`.
#[test]
fn a_label_makes_it_a_status_region_with_a_text_node() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Loader { variant: "dots", label: "Loading results" } }
        }
    }

    let html = body(&render(app));
    let attributes = attributes_of(&html, "span");

    assert_eq!(attributes["role"], "status", "{attributes:?}");
    assert!(!attributes.contains_key("aria-hidden"), "{attributes:?}");
    assert!(!attributes.contains_key("aria-label"), "{attributes:?}");
    assert!(html.contains("Loading results"), "{html}");
}

/// `bars` and `dots` are three real boxes - three independently delayed
/// animations need three elements.
#[test]
fn bars_and_dots_render_three_children() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Loader { variant: "bars" }
                Loader { variant: "dots" }
            }
        }
    }

    let html = body(&render(app));
    assert_eq!(html.matches("<span").count(), 8, "{html}");
    assert!(html.contains(r#"data-state="bars size-md""#), "{html}");
    assert!(html.contains(r#"data-state="dots size-md""#), "{html}");
}

/// Cancelling the animation drops a bar onto its static style, and the bars'
/// keyframes start at `opacity: 0`. So the reduced-motion arm has to put the
/// visible end back, per child - a bare `animation: none` would leave three
/// invisible bars for exactly the readers who asked for less motion.
#[test]
fn reduced_motion_stops_every_bar_on_its_visible_end() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Loader { variant: "bars" } }
        }
    }

    let html = render(app);
    let class = loader_class(&html);

    for n in 1..=3 {
        let rule = format!(
            r#".{class}[data-state~="bars"] > span:nth-child({n}){{transform:scale(1);opacity:1;animation:none;}}"#
        );
        let media = html
            .split("@media (prefers-reduced-motion: reduce)")
            .skip(1)
            .any(|block| block.contains(&rule));
        assert!(
            media,
            "missing {rule} under the reduced-motion query: {html}"
        );
    }
}

/// The size resolves on the root and the children inherit it, and the ink is
/// per instance - never on `:root`.
#[test]
fn size_and_colour_resolve_on_the_root() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Loader { size: "xl", color: "success" } }
        }
    }

    let html = render(app);
    let style = attributes_of(&body(&html), "span")["style"].clone();

    assert!(
        style.contains("--lsx-loader-color:var(--lsx-success-6);"),
        "{style}"
    );
    assert!(
        html.contains(r#"[data-state~="size-xl"]{--lsx-loader-size:var(--lsx-loader-size-xl);"#),
        "{html}"
    );
    assert!(html.contains("--lsx-loader-size-xxl:72px;"), "{html}");
    assert!(html.contains("@keyframes lsx-loader-bars"), "{html}");
}
