//! `Skeleton`'s rendered contract: covered content leaves the accessibility
//! tree and the tab order while visible, `circle` derives its width from
//! `height` or else its children, and the theme's colour and duration reach the CSS.

use crate::common::{attributes_of, body, classes_of, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Button, Skeleton},
    theme::{Color, ColorShade, ColorValue, SkeletonDefaults, Theme},
};

/// The skeleton's own framework class - the first one on its root.
fn skeleton_class(html: &str) -> String {
    classes_of(&body(html), "div")
        .first()
        .expect("a class on the skeleton")
        .clone()
}

/// `aria-hidden` alone would leave the covered button tabbable - a focusable
/// node with no name. `inert` is what takes it out of the tab order.
#[test]
fn a_visible_skeleton_hides_and_disables_its_children() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Skeleton { Button { "Save" } } }
        }
    }

    let html = body(&render(app));
    let attributes = attributes_of(&html, "div");

    assert_eq!(attributes["aria-hidden"], "true", "{attributes:?}");
    assert!(html.contains("inert"), "{html}");
    assert_eq!(
        attributes["data-state"], "visible animate radius-sm",
        "the defaults: {attributes:?}"
    );
    assert!(
        html.contains("Save"),
        "the children render underneath: {html}"
    );
}

#[test]
fn a_hidden_skeleton_leaves_its_children_alone() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Skeleton { visible: false, animate: false, Button { "Save" } } }
        }
    }

    let html = body(&render(app));
    let attributes = attributes_of(&html, "div");

    assert!(!attributes.contains_key("aria-hidden"), "{attributes:?}");
    assert!(!html.contains("inert"), "{html}");
    assert_eq!(attributes["data-state"], "radius-sm", "{attributes:?}");
}

#[test]
fn height_and_width_are_per_instance_vars() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Skeleton { height: "12px", width: "180px" } }
        }
    }

    let style = attributes_of(&body(&render(app)), "div")["style"].clone();

    assert!(style.contains("--lsx-skeleton-height:12px;"), "{style}");
    assert!(style.contains("--lsx-skeleton-width:180px;"), "{style}");
}

/// One number draws a round placeholder: the width follows the height, and
/// the radius token is replaced rather than competing with the circle's.
#[test]
fn circle_takes_its_width_from_height_and_drops_the_radius_token() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Skeleton { circle: true, height: "40px", width: "180px", radius: "xl" } }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&body(&html), "div");
    let class = skeleton_class(&html);

    assert!(
        attributes["style"].contains("--lsx-skeleton-width:40px;"),
        "{attributes:?}"
    );
    assert_eq!(attributes["data-state"], "visible animate circle");
    assert!(
        html.contains(&format!(
            r#".{class}[data-state~="circle"]{{--lsx-skeleton-radius:1000px;"#
        )),
        "{html}"
    );
}

/// A circle wrapping content, with no `height`: the width must not fall back
/// to the shape's `100%`, or an avatar placeholder spans the whole row.
#[test]
fn a_circle_without_height_is_as_wide_as_its_children() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Skeleton { circle: true, Button { "A" } } }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&body(&html), "div");
    let class = skeleton_class(&html);

    assert!(
        !attributes
            .get("style")
            .is_some_and(|style| style.contains("--lsx-skeleton-width")),
        "{attributes:?}"
    );
    let rule = html
        .split(&format!(r#".{class}[data-state~="circle"]{{"#))
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .expect("a circle rule");
    assert!(
        rule.contains("width:var(--lsx-skeleton-width, fit-content);"),
        "{rule}"
    );
}

/// The pulse runs on `::after` only, stops under reduced motion, and settles
/// between its two ends rather than on the dim one.
#[test]
fn reduced_motion_stops_the_pulse_at_a_mid_opacity() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Skeleton {} }
        }
    }

    let html = render(app);
    let class = skeleton_class(&html);

    assert!(
        html.contains(&format!(
            r#".{class}[data-state~="animate"]::after{{animation:lsx-skeleton-pulse var(--lsx-skeleton-duration) linear infinite;}}"#
        )),
        "{html}"
    );
    let rule = format!(r#".{class}[data-state~="animate"]::after{{animation:none;opacity:0.7;}}"#);
    assert!(
        html.split("@media (prefers-reduced-motion: reduce)")
            .skip(1)
            .any(|block| block.contains(&rule)),
        "missing {rule} under the reduced-motion query: {html}"
    );
    assert!(html.contains("@keyframes lsx-skeleton-pulse"), "{html}");
}

static SLOW_ERROR: Theme = Theme {
    skeleton: SkeletonDefaults {
        color: ColorValue::Shade(Color::Error, ColorShade::S2),
        duration: "3s",
        ..Theme::DEFAULT.skeleton
    },
    ..Theme::DEFAULT
};

/// Both themed fields reach `:root` - a field nothing reads would be dead
/// configuration.
#[test]
fn the_theme_colour_and_duration_reach_the_stylesheet() {
    fn default() -> Element {
        rsx! {
            LiberoProvider { Skeleton {} }
        }
    }
    fn themed() -> Element {
        rsx! {
            LiberoProvider { theme: &SLOW_ERROR, Skeleton {} }
        }
    }

    let html = render(default);
    assert!(
        html.contains("--lsx-skeleton-color:var(--lsx-grey-3);"),
        "{html}"
    );
    assert!(html.contains("--lsx-skeleton-duration:1500ms;"), "{html}");

    let html = render(themed);
    assert!(
        html.contains("--lsx-skeleton-color:var(--lsx-error-2);"),
        "{html}"
    );
    assert!(html.contains("--lsx-skeleton-duration:3s;"), "{html}");
}

/// The children are hidden rather than covered, so nothing paints through
/// whatever its z-index, and the grey is the only thing that opts back in.
#[test]
fn a_visible_skeleton_hides_its_children_and_shows_only_the_grey() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Skeleton { "x" } }
        }
    }

    let html = render(app);
    let class = skeleton_class(&html);

    assert!(
        html.contains(&format!(
            r#".{class}[data-state~="visible"]{{position:relative;overflow:hidden;visibility:hidden;}}"#
        )),
        "{html}"
    );
    assert!(
        html.contains(&format!(
            r#".{class}[data-state~="visible"]::after{{content:"";position:absolute;inset:0;visibility:visible;background:var(--lsx-skeleton-color);}}"#
        )),
        "{html}"
    );
    assert!(
        !html.contains(&format!(".{class}[data-state~=\"visible\"]::before")),
        "no cover layer: {html}"
    );
}

static STILL: Theme = Theme {
    skeleton: SkeletonDefaults {
        animate: false,
        ..Theme::DEFAULT.skeleton
    },
    ..Theme::DEFAULT
};

/// An unset `animate` takes the theme's.
#[test]
fn an_unset_animate_follows_the_theme() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { theme: &STILL, Skeleton { Button { "Save" } } }
        }
    }

    let attributes = attributes_of(&body(&render(app)), "div");
    assert_eq!(
        attributes["data-state"], "visible radius-sm",
        "{attributes:?}"
    );
}
