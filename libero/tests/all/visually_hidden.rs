use crate::common::{body, classes_of, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Checkbox, VisuallyHidden},
};

#[test]
fn visually_hidden_stays_in_the_accessibility_tree() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                VisuallyHidden { "screen reader only" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("screen reader only"));
}

/// The component needs no positioned parent: `fixed` is never part of the
/// page's scrollable overflow. Unhosted and `absolute`, its span kept a page
/// scrollable at 1189px in a 900px window (todo 63).
#[test]
fn visually_hidden_is_fixed() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                VisuallyHidden { "screen reader only" }
            }
        }
    }

    let html = render(app);
    let span = classes_of(&html, "span");
    assert!(
        span.iter()
            .any(|class| html.contains(&format!(".{class}{{position:fixed"))),
        "{span:?}"
    );
}

/// Only `focusable` unclips on focus: a default reveal would pop a hidden
/// native input out of its custom visual (todo 613).
#[test]
fn only_a_focusable_one_reveals_on_focus() {
    fn plain() -> Element {
        rsx! {
            LiberoProvider {
                VisuallyHidden { "screen reader only" }
            }
        }
    }
    fn focusable() -> Element {
        rsx! {
            LiberoProvider {
                VisuallyHidden { focusable: true, a { href: "#main", "Skip" } }
            }
        }
    }

    let reveals = |html: &str| {
        classes_of(html, "span")
            .iter()
            .any(|class| html.contains(&format!(".{class}:focus-within{{")))
    };

    assert!(!reveals(&render(plain)));
    let html = render(focusable);
    assert!(reveals(&html), "{html}");
}

/// The hosted inputs keep the shared `absolute` recipe. Tab onto one scrolls
/// its `relative` label into view; under `fixed` it scrolled nothing and the
/// label stayed off-screen (todo 63, measured on four pages).
#[test]
fn a_hosted_hidden_input_stays_absolute() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Checkbox { label: "Accept" }
            }
        }
    }

    let html = render(app);
    let input = classes_of(&html, "input");
    assert!(
        input
            .iter()
            .any(|class| html.contains(&format!(".{class}{{position:absolute"))),
        "{input:?}"
    );
}
