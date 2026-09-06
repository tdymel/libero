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
