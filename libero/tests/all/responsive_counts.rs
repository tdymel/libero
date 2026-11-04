//! Responsive counts and layouts (todos 5688-5692): Pagination `siblings`/`boundaries`, Carousel
//! `per_view`/`slides_to_scroll`, Cascader and BarChart `layout`. Rust reads them, so the server
//! renders the base; Carousel's slide widths also ride a var per breakpoint.

use crate::common::{body, css_rules_for, render, tag_with};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        BarChart, Carousel, Cascader, CascaderLayout, CascaderOption, Pagination, Series,
    },
    theme::{BarLayout, responsive},
};

fn data() -> Vec<CascaderOption<String>> {
    vec![CascaderOption::new("food", "Food").children(vec![CascaderOption::new("leek", "Leek")])]
}

fn slides() -> Vec<Element> {
    (0..6).map(|n| rsx! { "slide {n}" }).collect()
}

#[component]
fn Counts(responsive_values: bool) -> Element {
    let siblings = match responsive_values {
        true => responsive(0).md(2),
        false => 0.into(),
    };
    let boundaries = match responsive_values {
        true => responsive(2).lg(1),
        false => 2.into(),
    };
    let step = match responsive_values {
        true => responsive(1).md(3),
        false => 1.into(),
    };
    let layout = match responsive_values {
        true => responsive(CascaderLayout::Paths).md(CascaderLayout::Columns),
        false => CascaderLayout::Paths.into(),
    };
    let bars = match responsive_values {
        true => responsive(BarLayout::Stacked).md(BarLayout::Grouped),
        false => BarLayout::Stacked.into(),
    };
    rsx! {
        Pagination { total: 20, page: 10, aria_label: "Pages", siblings, boundaries }
        Carousel { aria_label: "Offers", per_view: 2.0, slides_to_scroll: step, slides: slides() }
        Cascader { label: "Aisle", data: data(), layout }
        BarChart {
            title: "Sales",
            layout: bars,
            x: ["a", "b"],
            series: vec![Series::new("A", [1.0, 3.0]), Series::new("B", [3.0, 1.0])],
        }
    }
}

#[test]
fn a_responsive_count_or_layout_renders_its_base_on_the_server() {
    fn plain() -> Element {
        rsx! { LiberoProvider { Counts { responsive_values: false } } }
    }
    fn per_breakpoint() -> Element {
        rsx! { LiberoProvider { Counts { responsive_values: true } } }
    }

    let html = render(plain);
    assert_eq!(render(per_breakpoint), html);
    // Base siblings 0 and boundaries 2 around page 10 of 20: 1 2 … 10 … 19 20.
    let pages = body(&html).matches(r#"data-slot="page""#).count();
    assert_eq!(pages, 5, "{html}");
}

#[test]
fn a_responsive_per_view_sizes_the_slides_per_breakpoint() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel { id: "plain", aria_label: "Plain", per_view: 3.0, slides: slides() }
                Carousel { id: "ranged", aria_label: "Ranged", per_view: responsive(1.0).md(3.0), slides: slides() }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    let var = "--lsx-carousel-per-view-override";
    let plain = tag_with(&page, r#"id="plain""#);
    assert!(plain["style"].contains(&format!("{var}:3")), "{plain:?}");

    let ranged = tag_with(&page, r#"id="ranged""#);
    assert!(
        !ranged.get("style").is_some_and(|style| style.contains(var)),
        "an inline var beats the breakpoints: {ranged:?}"
    );
    let declared: Vec<String> = css_rules_for(&html, &ranged)
        .into_iter()
        .filter_map(|rule| {
            let value = rule.declarations.get(var)?;
            let media = rule.at.iter().find(|at| at.starts_with("@media"));
            Some(media.map_or(value.clone(), |media| format!("{media} {value}")))
        })
        .collect();
    assert!(declared.iter().any(|value| value == "1"), "{declared:?}");
    assert!(
        declared
            .iter()
            .any(|value| value.starts_with("@media") && value.ends_with(" 3")),
        "{declared:?}"
    );
}
