use crate::common::{attributes_of, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Icon, SvgData},
    theme::Color,
};

#[test]
fn icon_renders_its_variant_and_colour_variables() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Icon { color: Color::Success, "★" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "span");

    assert_eq!(attributes["data-state"], "filled");
    assert!(attributes["style"].contains("--lsx-icon-color:var(--lsx-success-text-6);"));
}

/// An unnamed icon is decoration: a caller svg inside would read as an
/// unnamed image otherwise (todo 612).
#[test]
fn an_unnamed_icon_is_hidden_from_assistive_technology() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Icon { svg {} }
            }
        }
    }

    let attributes = attributes_of(&render(app), "span");

    assert_eq!(attributes["aria-hidden"], "true");
    assert!(!attributes.contains_key("role"));
}

#[test]
fn a_labelled_icon_is_an_image() {
    fn labelled() -> Element {
        rsx! {
            LiberoProvider {
                Icon { aria_label: "Verified", svg {} }
            }
        }
    }
    fn labelledby() -> Element {
        rsx! {
            LiberoProvider {
                Icon { aria_labelledby: "caption", svg {} }
            }
        }
    }

    for app in [labelled, labelledby] {
        let attributes = attributes_of(&render(app), "span");
        assert_eq!(attributes["role"], "img");
        assert!(!attributes.contains_key("aria-hidden"));
    }
}

/// A caller's own `role` or `aria-hidden` wins over the default.
#[test]
fn a_callers_role_is_kept() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Icon { aria_label: "Status", role: "status", svg {} }
            }
        }
    }

    let attributes = attributes_of(&render(app), "span");

    assert_eq!(attributes["role"], "status");
}

const BAR: SvgData = SvgData::new(r#"<svg viewBox="0 0 16 16"><path d="M2 8h12"/></svg>"#);

/// Todo 1094: `src` > `svg` > `children`.
#[test]
fn an_svg_draws_a_pictogram_and_a_src_wins_over_it() {
    fn svg() -> Element {
        rsx! {
            LiberoProvider {
                Icon { svg: BAR, "children" }
            }
        }
    }
    fn src() -> Element {
        rsx! {
            LiberoProvider {
                Icon { src: "/a.svg", svg: BAR }
            }
        }
    }

    let html = render(svg);
    assert!(html.contains(r#"<path d="M2 8h12"/>"#), "{html}");
    assert!(!html.contains("children"), "{html}");
    let html = render(src);
    assert!(html.contains("mask-image"), "{html}");
    assert!(!html.contains("<svg"), "{html}");
}
