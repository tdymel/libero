//! Responsive layout and typography props (todo 5651): axis props write run words and their
//! copies, value props declare each breakpoint in the user layer, and plain values add nothing.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        Anchor, Blockquote, Container, Divider, Flex, FlexWrap, Grid, GridArea, GridTemplate,
        GridZone, Header, Kbd, Sidebar, Text, Title,
    },
    sx::ThemeAwareValue,
    theme::{
        CONTAINER_GUTTERS, CONTAINER_SIZE, GRID_AREAS_VAR, GRID_ZONE_GAP, HEADER_HEIGHT, Size,
        SizeCss, responsive,
    },
};

const MD: &str = "@media (min-width: 62rem)";

/// The run words and range copies in the page.
fn run_words(html: &str) -> Vec<&str> {
    let after = |marker| html.split(marker).skip(1);
    after(r#"data-state=""#)
        .chain(after(r#"[data-state~=""#))
        .map(|rest| &rest[..rest.find('"').unwrap_or(0)])
        .filter(|words| {
            words
                .split(' ')
                .any(|word| word.contains('-') && word.contains(':'))
        })
        .collect()
}

#[test]
fn a_plain_layout_and_typography_page_carries_no_run_word() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Header { size: "lg", "Banner" }
                Container { size: "md", gutters: "lg",
                    Flex { wrap: true,
                        Sidebar { size: "sm", aria_label: "Filters", "Filters" }
                        GridZone { gap: "lg", "Zone" }
                    }
                    Title { size: "lg", line_clamp: 2, "Title" }
                    Text { size: "sm", line_clamp: 2, "Text" }
                    Kbd { size: "lg", "K" }
                    Anchor { size: "sm", to: "/x", "Link" }
                    Blockquote { size: "lg", radius: "md", "Quote" }
                    Divider { size: "sm", spacing: "md" }
                }
            }
        }
    }

    let html = render(app);
    let words = run_words(&html);
    assert!(words.is_empty(), "run words on a plain page: {words:?}");
    // Plain values keep their inline vars.
    assert!(body(&html).contains(&format!("{}:", CONTAINER_SIZE.override_var().name())));
}

#[test]
fn responsive_type_sizes_write_run_words_and_copies() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Title { size: responsive(Size::Lg).md(Size::Xxl), "Title" }
                Text { size: responsive(Size::Sm).md(Size::Lg), "Text" }
                Kbd { size: responsive(Size::Xs).md(Size::Xl), "K" }
                Anchor { size: responsive(Size::Xs).md(Size::Md), to: "/x", "Link" }
                Divider { size: responsive(Size::Xs).md(Size::Md) }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    // The tag follows the base size.
    let title = attributes_of(&page, "h3");
    assert!(title["data-state"].contains("md-:size-xxl"), "{title:?}");
    for word in [
        "md-:size-xxl",
        "md-:size-lg",
        "md-:size-xl",
        "md-:size-md",
        "base-md:size-xs",
    ] {
        assert!(page.contains(word), "{word} in the markup");
        assert!(
            html.contains(&format!(r#"[data-state~="{word}"]"#)),
            "{word} has no copy"
        );
    }
}

#[test]
fn a_blockquote_takes_size_and_radius_per_breakpoint() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Blockquote {
                    size: responsive(Size::Sm).md(Size::Lg),
                    radius: responsive(ThemeAwareValue::from(Size::Xs)).md("0".into()),
                    attribution: rsx! { "Someone" },
                    "Quote"
                }
            }
        }
    }

    let html = render(app);
    let quote = attributes_of(&body(&html), "blockquote");
    let state = &quote["data-state"];
    // The custom md radius keeps the theme's step word (`sm`).
    for word in ["md-:size-lg", "base-md:radius-xs", "md-:radius-sm"] {
        assert!(state.contains(word), "{word} in {state}");
    }
    assert!(
        !quote
            .get("style")
            .is_some_and(|style| { style.contains(SizeCss::RADIUS.override_var().name()) })
    );
    // The caption's own size copy.
    let caption = attributes_of(&body(&html), "figcaption");
    assert!(caption["data-state"].contains("md-:size-lg"));
    assert_eq!(
        html.matches(r#"[data-state~="md-:size-lg"]"#).count(),
        2,
        "a copy for the quote and one for the caption"
    );
}

#[test]
fn a_line_clamp_per_breakpoint_rides_the_user_layer() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Text { line_clamp: responsive(0).md(3), "Text" } }
        }
    }

    let html = render(app);
    let text = attributes_of(&body(&html), "p");
    assert!(text["data-state"].contains("line-clamp"));
    assert!(
        !text
            .get("style")
            .is_some_and(|style| style.contains("--lsx-lines"))
    );
    assert!(html.contains("--lsx-lines:none"));
    assert!(html.contains(&format!("{MD}{{")));
    assert!(html.contains("--lsx-lines:3"));
}

#[derive(Clone, Copy, PartialEq)]
enum Area {
    Nav,
    Main,
}

impl GridArea for Area {
    fn name(&self) -> &'static str {
        match self {
            Self::Nav => "nav",
            Self::Main => "main",
        }
    }
}

#[test]
fn value_props_declare_each_breakpoint_instead_of_an_inline_var() {
    fn app() -> Element {
        let stacked = GridTemplate::new()
            .row(|row| row.cell(Area::Nav))
            .row(|row| row.cell(Area::Main))
            .build()
            .unwrap();
        let side = GridTemplate::new()
            .row(|row| row.cell(Area::Nav).cells(Area::Main, 3))
            .build()
            .unwrap();
        rsx! {
            LiberoProvider {
                Header { size: responsive(ThemeAwareValue::from("48px")).md("72px".into()), publish_height: true, "Banner" }
                Container {
                    size: responsive(Size::Sm).md(Size::Lg),
                    gutters: responsive(Size::Xs).md(Size::Lg),
                    Flex { wrap: responsive(FlexWrap::Wrap).md(FlexWrap::NoWrap),
                        Sidebar { size: responsive(ThemeAwareValue::from(Size::Sm)).md("20rem".into()), aria_label: "Filters", "Filters" }
                    }
                    Grid { template: responsive(stacked).md(side),
                        GridZone { area: Area::Nav, gap: responsive(Size::Xs).md(Size::Xl), "Nav" }
                        GridZone { area: Area::Main, "Main" }
                    }
                    Divider { spacing: responsive(Size::Xs).md(Size::Xl) }
                }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    let header = attributes_of(&page, "header");
    assert!(
        !header
            .get("style")
            .is_some_and(|style| style.contains(HEADER_HEIGHT.override_var().name()))
    );
    // The published height follows the breakpoints too.
    assert!(html.contains(&format!("{MD}{{:root[data-lsx-header=")));
    for var in [
        CONTAINER_SIZE.override_var().name().to_string(),
        CONTAINER_GUTTERS.override_var().name().to_string(),
        "--lsx-flex-wrap".to_string(),
        GRID_AREAS_VAR.name().to_string(),
        GRID_ZONE_GAP.override_var().name().to_string(),
        "--lsx-divider-spacing".to_string(),
    ] {
        assert!(html.contains(&format!("{var}:")), "{var} not declared");
    }
    // The custom md width keeps the theme's step word, and its copy.
    let aside = attributes_of(&page, "aside");
    assert!(aside["data-state"].contains("base-md:size-sm"), "{aside:?}");
    assert!(
        html.contains(r#"[data-state~="base-md:size-sm"]"#),
        "no sidebar copy"
    );
    assert!(html.contains("20rem"), "the sidebar's custom md width");
}
