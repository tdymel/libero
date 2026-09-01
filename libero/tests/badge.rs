//! `Badge`'s rendered contract: one bare `<span>`, chrome that never grows a
//! hover, and a `circle` that finds the active size's height on its own.

mod common;

use common::{attributes_of, body, classes_of, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Badge};

fn app() -> Element {
    rsx! {
        LiberoProvider { Badge { color: "success", "Shipped" } }
    }
}

/// The badge's own framework class - the first one on its root, which is what
/// every rule below is keyed on.
fn badge_class(html: &str) -> String {
    classes_of(&body(html), "span")
        .first()
        .expect("a class on the badge")
        .clone()
}

/// A badge is visible text, read in document order, so its content already is
/// its accessible name. A `role="status"` here would make every badge announce
/// itself on mount - a live region is the caller's own, around the badge.
#[test]
fn it_renders_one_span_with_no_role_and_no_aria() {
    let html = body(&render(app));

    assert!(html.contains("<span"), "{html}");
    assert_eq!(html.matches("<span").count(), 1, "{html}");
    assert!(html.contains(">Shipped</span>"), "{html}");

    let attributes = attributes_of(&html, "span");
    assert!(!attributes.contains_key("role"), "{attributes:?}");
    assert!(
        attributes.keys().all(|name| !name.starts_with("aria-")),
        "{attributes:?}"
    );
}

/// `variant_chrome_sx`, not `button_variant_sx`: a static label that changed
/// colour under the pointer would be claiming to be interactive. The rule is
/// `Icon`'s, and the mistake is one `Chip`-shaped copy-paste away.
#[test]
fn the_variant_chrome_has_no_hover() {
    let html = render(app);
    let class = badge_class(&html);

    assert!(
        !html.contains(&format!(".{class}[data-state~=\"filled\"]:hover")),
        "{html}"
    );
    assert!(!html.contains(&format!(".{class}:hover")), "{html}");
}

/// `circle` has to floor the width at *the active size's* height, and the rule
/// that does it cannot know which size token is on. So each size step
/// republishes its height unsuffixed and `circle` reads that - hardcode a step
/// here instead and every other size draws an oval.
#[test]
fn circle_floors_the_width_at_the_active_size_height() {
    fn circle_app() -> Element {
        rsx! {
            LiberoProvider { Badge { circle: true, size: "xl", "9" } }
        }
    }

    let html = render(circle_app);
    let class = badge_class(&html);

    assert!(
        body(&html).contains(r#"data-state="filled size-xl circle""#),
        "{}",
        body(&html)
    );
    assert!(
        html.contains(&format!(
            r#".{class}[data-state~="circle"]{{min-width:var(--lsx-badge-box);"#
        )),
        "{html}"
    );
    assert!(
        html.contains(
            r#"[data-state~="size-xl"]{--lsx-badge-font:var(--lsx-badge-font-size-xl);--lsx-badge-box:var(--lsx-badge-height-xl);"#
        ),
        "{html}"
    );
}

/// A tinted surface owes its label the tint's own `-contrast` twin, never an
/// accent shade - the rule that has now been broken three times. Both halves
/// come out of one resolution here, so they cannot disagree.
#[test]
fn a_theme_colour_resolves_a_shade_and_its_contrast_twin() {
    let style = attributes_of(&body(&render(app)), "span")["style"].clone();

    assert!(
        style.contains("--lsx-badge-color:var(--lsx-success-6);"),
        "{style}"
    );
    assert!(
        style.contains("--lsx-badge-contrast:var(--lsx-success-contrast-6);"),
        "{style}"
    );
}

/// The default is off the radius scale - a pill at every height - so it lives
/// in the theme rather than as a `Size`, and an unset prop must leave the
/// override var alone. A caller naming a scale step still resolves through it.
#[test]
fn radius_defaults_to_the_theme_pill_and_a_step_resolves_through_the_scale() {
    let style = attributes_of(&body(&render(app)), "span")["style"].clone();
    assert!(!style.contains("--lsx-badge-radius-override"), "{style}");

    fn square_app() -> Element {
        rsx! {
            LiberoProvider { Badge { radius: "sm", "Beta" } }
        }
    }
    let style = attributes_of(&body(&render(square_app)), "span")["style"].clone();
    assert!(
        style.contains("--lsx-badge-radius-override:var(--lsx-radius-sm);"),
        "{style}"
    );
}
