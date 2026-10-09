//! Todo 2663: a `gap` takes any CSS next to the size scale; on `Grid` and `DataList` it
//! is responsive too. A size keeps its state class; custom CSS replaces only the value.

use crate::common::{body, nth_attributes, render, rules_for, tags_with};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Carousel, DataList, DataListItem, Grid, GridArea, GridTemplate, Marquee},
    sx::ThemeAwareValue,
    theme::{Size, responsive},
};

#[derive(Clone, Copy, PartialEq)]
struct Only;

impl GridArea for Only {
    fn name(&self) -> &'static str {
        "only"
    }
}

#[test]
fn a_grid_gap_takes_css_and_breakpoints() {
    fn app() -> Element {
        let template = GridTemplate::new()
            .row(|row| row.cell(Only))
            .build()
            .expect("a template");
        rsx! {
            LiberoProvider {
                Grid { template, gap: responsive(ThemeAwareValue::from("0")).md(Size::Lg.into()), "g" }
            }
        }
    }
    let html = render(app);
    let rules = rules_for(&html, &nth_attributes(&body(&html), "div", 0));

    assert!(rules.contains("gap:0"), "{rules}");
    assert!(html.contains("gap:var(--lsx-spacing-lg)"), "{html}");
}

#[test]
fn a_data_list_gap_takes_css() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DataList { gap: "3px", DataListItem { label: rsx! { "a" }, "b" } }
            }
        }
    }
    let html = render(app);

    assert!(html.contains("gap:3px"), "{html}");
}

#[test]
fn a_custom_gap_fills_the_spacing_var() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel { "data-gap": "", aria_label: "C", gap: "7px", slides: vec![rsx! { "a" }] }
                Marquee { "data-gap": "", gap: "7px", "m" }
            }
        }
    }
    let html = render(app);
    let tags = tags_with(&body(&html), "data-gap");

    assert_eq!(tags.len(), 2, "{html}");
    for tag in tags {
        assert!(
            tag.get("style").is_some_and(|style| style.contains(":7px")),
            "{tag:?}"
        );
    }
}
