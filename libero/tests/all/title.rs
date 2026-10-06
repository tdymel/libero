use crate::common::{attributes_of, body, css_rules_for, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::Title,
    theme::{Gradient, Size, Theme, TitleDefaults},
};

#[test]
fn title_renders_as_its_heading_level() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Title { component: "h2", "Heading" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("<h2"));
    assert!(attributes_of(&html, "h2").contains_key("data-state"));
}

static SMALL_TITLES: Theme = Theme {
    title: TitleDefaults {
        size: Size::Sm,
        ..Theme::DEFAULT.title
    },
    ..Theme::DEFAULT
};

/// The theme's step is the look only: a bare `Title` keeps its h1, so a theme
/// cannot move it in the document outline.
#[test]
fn an_unsized_title_takes_the_themes_look_but_keeps_its_h1() {
    fn app() -> Element {
        rsx! { LiberoProvider { themes: &SMALL_TITLES, Title { "Heading" } } }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "h1")["data-state"], "size-sm");
}

/// Todo 2546: each size's own tag, and the props beat the theme's size.
#[test]
fn each_size_renders_its_heading_level() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { themes: &SMALL_TITLES,
                Title { size: "xxl", "One" }
                Title { size: "xl", "Two" }
                Title { size: "lg", "Three" }
                Title { size: "md", "Four" }
                Title { size: "sm", "Five" }
                Title { size: "xs", "Six" }
            }
        }
    }

    let html = render(app);

    for (tag, size) in [
        ("h1", "xxl"),
        ("h2", "xl"),
        ("h3", "lg"),
        ("h4", "md"),
        ("h5", "sm"),
        ("h6", "xs"),
    ] {
        assert_eq!(
            attributes_of(&html, tag)["data-state"],
            format!("size-{size}")
        );
    }
}

#[test]
fn an_explicit_component_beats_the_sizes_tag() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { themes: &SMALL_TITLES,
                Title { size: "lg", component: "h2", "Heading" }
            }
        }
    }

    let html = render(app);

    assert!(!body(&html).contains("<h3"));
    assert_eq!(attributes_of(&html, "h2")["data-state"], "size-lg");
}

/// Todo 2543: a long word at `xxl` must break inside a 320px column.
#[test]
fn a_title_breaks_a_long_word() {
    fn app() -> Element {
        rsx! { LiberoProvider { Title { "Donaudampfschifffahrt" } } }
    }

    let html = render(app);
    let rules = css_rules_for(&html, &attributes_of(&html, "h1"));

    assert!(
        rules.iter().any(|rule| rule.unconditional()
            && rule.declarations.get("overflow-wrap").map(String::as_str) == Some("break-word")),
        "no overflow-wrap on the title: {rules:?}"
    );
}

/// Todo 2544: print drops the clipped background, so the glyphs go solid there.
#[test]
fn a_gradient_title_prints_in_its_first_stop() {
    fn app() -> Element {
        rsx! { LiberoProvider { Title { gradient: Gradient::default(), "Heading" } } }
    }

    let html = render(app);
    let rules = css_rules_for(&html, &attributes_of(&html, "h1"));

    assert!(
        rules.iter().any(|rule| {
            rule.at.iter().any(|at| at == "@media print")
                && rule
                    .declarations
                    .get("background-image")
                    .map(String::as_str)
                    == Some("none")
                && rule
                    .declarations
                    .get("color")
                    .is_some_and(|color| color.contains("--lsx-gradient-from"))
        }),
        "no print fallback on the gradient title: {rules:?}"
    );
}
