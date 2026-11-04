//! Responsive props of the data_display family (todo 5654): axis props carry run words and
//! their range copies, value props declare their var per breakpoint in the user layer.

use std::collections::BTreeMap;

use crate::common::{body, css_rules_for, render, tag_with};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        Accordion, AccordionOpen, AvatarGroup, AvatarSpec, Badge, Carousel, Icon, Image, ImageItem,
        ImageList, Indicator, List, ListItem, Marquee, Options, Timeline, TimelineEvent,
    },
    sx::ThemeAwareValue,
    theme::{Size, TimelineAlign, Variant, responsive},
};

/// Whether the page holds a run word or a user-layer rule.
fn has_responsive_css(html: &str) -> bool {
    html.contains("@layer lsx-user-static")
        || html
            .split(r#"[data-state~=""#)
            .skip(1)
            .any(|rest| rest[..rest.find('"').unwrap_or(0)].contains(':'))
}

/// `var` as `element`'s rules declare it: the unconditional value, then each `@media` one.
fn declared(html: &str, element: &BTreeMap<String, String>, var: &str) -> Vec<String> {
    css_rules_for(html, element)
        .into_iter()
        .filter_map(|rule| {
            let value = rule.declarations.get(var)?;
            let media = rule.at.iter().find(|at| at.starts_with("@media"));
            Some(match media {
                Some(media) => format!("{media} {value}"),
                None => value.clone(),
            })
        })
        .collect()
}

fn words(element: &BTreeMap<String, String>) -> Vec<&str> {
    element["data-state"].split(' ').collect()
}

#[derive(Clone, Copy, PartialEq, Options)]
enum Pane {
    One,
    Two,
}

fn events() -> Vec<TimelineEvent> {
    vec![TimelineEvent::new("Ordered"), TimelineEvent::new("Shipped")]
}

#[test]
fn plain_values_carry_no_responsive_css() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Badge { id: "badge", radius: "0", "New" }
                Icon { size: "lg", radius: "0", variant: "filled", "x" }
                Indicator { size: "lg", radius: "xs" }
                Image { src: "/a.svg", alt: "A", radius: "0" }
                Timeline { align: "end", bullet_size: "lg", radius: "0", gap: "sm", items: events() }
                List { size: "lg", ListItem { "one" } }
                Marquee { gap: "0", "x" }
                AvatarGroup { spacing: "md", people: vec![AvatarSpec::from("Ada")] }
                Carousel { aria_label: "Offers", gap: "0", height: "240px", slides: vec![rsx! { "a" }] }
                Accordion { open: AccordionOpen::One(Some(Pane::One)), size: "lg", panel: |_: Pane| rsx! { "x" } }
            }
        }
    }

    let html = render(app);
    assert!(!has_responsive_css(&html), "responsive CSS on a plain page");
    let badge = tag_with(&body(&html), r#"id="badge""#);
    assert!(badge["style"].contains("--lsx-badge-radius-override:0"));
}

#[test]
fn axis_props_carry_run_words_and_copies() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Indicator { id: "dot", size: responsive(Size::Xs).md(Size::Lg) }
                Icon { id: "glyph", variant: responsive(Variant::Standard).md(Variant::Filled), "x" }
                List { id: "list", size: responsive(Size::Xs).md(Size::Lg), ListItem { "one" } }
                Accordion {
                    id: "accordion",
                    open: AccordionOpen::One(Some(Pane::One)),
                    size: responsive(Size::Sm).md(Size::Lg),
                    panel: |_: Pane| rsx! { "x" },
                }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    for (id, word) in [
        ("dot", "md-:size-lg"),
        ("glyph", "md-:filled"),
        ("list", "md-:size-lg"),
        ("accordion", "md-:size-lg"),
    ] {
        let element = tag_with(&page, &format!(r#"id="{id}""#));
        assert!(words(&element).contains(&word), "{id}: {element:?}");
        assert!(
            html.contains(&format!(r#"[data-state~="{word}"]"#)),
            "{id}: no copy"
        );
    }
    let glyph = tag_with(&page, r#"id="glyph""#);
    assert!(words(&glyph).contains(&"base-md:standard"), "{glyph:?}");
}

#[test]
fn a_responsive_timeline_copies_root_item_and_bullet_blocks() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Timeline {
                    id: "events",
                    align: responsive(TimelineAlign::Start).md(TimelineAlign::Alternate),
                    bullet_size: responsive(Size::Sm).md(Size::Lg),
                    radius: responsive(ThemeAwareValue::from(Size::Xs)).md("0".into()),
                    gap: responsive(ThemeAwareValue::from(Size::Sm)).md(Size::Xl.into()),
                    items: events(),
                }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    let root = tag_with(&page, r#"id="events""#);
    for word in [
        "base-md:align-start",
        "md-:align-alternate",
        "md-:size-lg",
        "base-md:gap-sm",
        "md-:gap-xl",
        // A custom range keeps the theme's step word; the override var replaces its value.
        "base-md:radius-xs",
        "md-:radius-xl",
    ] {
        assert!(words(&root).contains(&word), "{word} in {root:?}");
    }
    assert!(
        !root
            .get("style")
            .is_some_and(|style| style.contains("override"))
    );
    let radius = declared(&html, &root, "--lsx-radius-override");
    assert!(
        radius
            .iter()
            .any(|value| value.starts_with("@media") && value.ends_with(" 0")),
        "{radius:?}"
    );
    let item = tag_with(&page, r#"data-slot="item""#);
    assert!(words(&item).contains(&"md-:align-alternate"), "{item:?}");
    let bullet = tag_with(&page, r#"data-slot="bullet""#);
    assert!(
        words(&bullet).contains(&"md-:align-alternate"),
        "{bullet:?}"
    );
    for copy in ["md-:align-alternate", "md-:gap-xl", "md-:size-lg"] {
        assert!(
            html.contains(&format!(r#"[data-state~="{copy}"]"#)),
            "{copy}"
        );
    }
}

#[test]
fn value_props_declare_their_var_per_breakpoint() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Badge { id: "badge", radius: responsive(ThemeAwareValue::from(Size::Xs)).md("0".into()), "New" }
                Icon { id: "glyph", size: responsive(ThemeAwareValue::from(Size::Sm)).md("48px".into()), "x" }
                Indicator { id: "dot", radius: responsive(ThemeAwareValue::from("0")).md(Size::Xl.into()) }
                Image { id: "image", src: "/a.svg", alt: "A", radius: responsive(ThemeAwareValue::from("0")).md("12px".into()) }
                Marquee { id: "marquee", gap: responsive(ThemeAwareValue::from("0")).md("32px".into()), "x" }
                AvatarGroup {
                    id: "group",
                    spacing: responsive(ThemeAwareValue::from("0")).md("16px".into()),
                    people: vec![AvatarSpec::from("Ada")],
                }
                Carousel {
                    id: "carousel",
                    aria_label: "Offers",
                    gap: responsive(ThemeAwareValue::from("0")).md("24px".into()),
                    height: responsive(ThemeAwareValue::from("200px")).md("400px".into()),
                    slides: vec![rsx! { "a" }],
                }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    for (id, var, base, md) in [
        ("badge", "--lsx-badge-radius-override", None, "0"),
        ("glyph", "--lsx-icon-size-override", None, "48px"),
        ("dot", "--lsx-indicator-radius-override", Some("0"), ""),
        ("image", "--lsx-image-radius-override", Some("0"), "12px"),
        ("marquee", "--lsx-marquee-gap", Some("0"), "32px"),
        ("group", "--lsx-avatar-group-spacing", Some("0"), "16px"),
        ("carousel", "--lsx-carousel-gap-override", Some("0"), "24px"),
        ("carousel", "--lsx-carousel-height", Some("200px"), "400px"),
    ] {
        let element = tag_with(&page, &format!(r#"id="{id}""#));
        assert!(
            !element
                .get("style")
                .is_some_and(|style| style.contains(var)),
            "{id}: {var} inline beats the breakpoints"
        );
        let values = declared(&html, &element, var);
        if let Some(base) = base {
            assert!(values.iter().any(|value| value == base), "{id}: {values:?}");
        }
        assert!(
            values
                .iter()
                .any(|value| value.starts_with("@media") && value.ends_with(md)),
            "{id}: {values:?}"
        );
    }
}

#[test]
fn a_responsive_image_list_radius_rides_the_cells_override_var() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ImageList {
                    radius: responsive(ThemeAwareValue::from(Size::Xs)).md("0".into()),
                    items: vec![ImageItem::new(rsx! { Image { src: "/a.svg", alt: "A" } })],
                }
            }
        }
    }

    let html = render(app);
    let cell = tag_with(&body(&html), r#"data-slot="item""#);
    let values = declared(&html, &cell, "--lsx-radius-override");
    assert!(
        values
            .iter()
            .any(|value| value.starts_with("@media") && value.ends_with(" 0")),
        "{values:?}"
    );
}
