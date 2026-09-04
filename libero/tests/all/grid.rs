use crate::common::{attributes_of, body, has_rule_for, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Grid, GridArea, GridItem, GridSpan, GridTemplate, GridZone, sp},
    theme::Size,
};

/// `attributes_of` reads the first matching tag, and a grid nests three deep.
fn nth_div(html: &str, skip: usize) -> String {
    let body = body(html);
    let mut rest = body.as_str();
    for _ in 0..skip {
        let start = rest.find("<div").expect("another nested div") + 4;
        rest = &rest[start..];
    }
    rest[rest.find("<div").expect("a div")..].to_string()
}

#[derive(Clone, Copy, PartialEq)]
enum TestArea {
    Header,
    Body,
}

impl GridArea for TestArea {
    fn name(&self) -> &'static str {
        match self {
            Self::Header => "header",
            Self::Body => "body",
        }
    }
}

fn test_template() -> GridTemplate {
    GridTemplate::new()
        .row(|row| row.cell(TestArea::Header))
        .row(|row| row.cells(TestArea::Body, 4))
        .build()
        .expect("a rectangular template")
}

#[test]
fn a_grid_publishes_its_areas_and_column_count_as_custom_properties() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Grid { template: test_template(), span { "zone" } }
            }
        }
    }

    let html = render(app);
    let style = attributes_of(&html, "div")
        .get("style")
        .expect("the areas variable")
        .clone();

    // The quotes `grid-template-areas` needs are HTML-escaped on the way into
    // the `style` attribute.
    assert!(style.contains("--lsx-grid-areas:&#34;header header header header&#34;"));
    assert!(style.contains("--lsx-grid-columns:4"));
}

#[test]
fn a_zone_carries_its_area_name_and_its_packing_states() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Grid { template: test_template(),
                    GridZone { area: TestArea::Body, dense: true, masonry: true, span { "item" } }
                }
            }
        }
    }

    let html = render(app);
    let zone = attributes_of(&nth_div(&html, 1), "div");

    assert!(
        zone.get("style")
            .expect("the zone area variable")
            .contains("--lsx-grid-zone-area:body")
    );
    let state = zone.get("data-state").expect("the zone states");
    assert!(state.contains("dense"));
    assert!(state.contains("masonry"));
}

#[test]
fn a_zone_works_without_a_grid_and_then_names_no_area() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                GridZone { masonry: true, GridItem { span: GridSpan::Half, "card" } }
            }
        }
    }

    let html = render(app);
    let zone = attributes_of(&nth_div(&html, 0), "div");

    // Every zone publishes its resolved gap, but only one naming an area
    // declares the area var.
    let style = zone.get("style").expect("the resolved gap");
    assert!(!style.contains("--lsx-grid-zone-area"));
    assert!(style.contains("--lsx-grid-zone-gap"));
    assert!(body(&html).contains("card"));
}

#[test]
fn an_item_writes_the_grid_item_token_the_zone_selects_on() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                GridZone { GridItem { span: GridSpan::TwoThirds, "card" } }
            }
        }
    }

    let html = render(app);
    let item = attributes_of(&nth_div(&html, 1), "div");
    let state = item.get("data-state").expect("the item states");

    assert!(state.contains("grid-item"));
    assert!(state.contains("span-two-thirds"));
    // Nothing measures without a browser, so no span is claimed.
    assert!(!state.contains("measured"));
}

#[test]
fn a_responsive_span_queries_its_zone_not_the_viewport() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Grid { template: test_template(),
                    GridZone { area: TestArea::Body,
                        GridItem { span: sp().base(GridSpan::Full).md(GridSpan::Half), "card" }
                    }
                }
            }
        }
    }

    let html = render(app);
    let zone = attributes_of(&nth_div(&html, 1), "div");

    // The zone names itself so an item has something to query.
    assert!(
        zone.get("style")
            .expect("the zone container variable")
            .contains("--lsx-grid-zone-container:lsx-zone-body")
    );

    // The base span still rides the recycled framework class; only the
    // breakpoint needs a rule of its own, and it is a container query.
    let item = attributes_of(&nth_div(&html, 2), "div");
    assert!(
        item.get("data-state")
            .expect("the item states")
            .contains("span-full")
    );
    assert!(html.contains("@container lsx-zone-body (min-width: 62rem){"));
    assert!(html.contains("grid-column:span 6;"));
    // A viewport query would be the bug this replaces.
    assert!(!html.contains("@media (min-width: 62rem){"));
}

/// Regression: `container-type: inline-size` zeroes an element's intrinsic
/// contribution, so on a standalone zone - a shrink-to-fit flex item, as in
/// the docs preview - it collapsed the zone to zero width and stacked every
/// item at x=0. A zone with no area has no container name to be queried by
/// either, so it must not be a container at all.
#[test]
fn only_a_zone_filling_an_area_is_a_query_container() {
    fn standalone() -> Element {
        rsx! {
            LiberoProvider {
                GridZone { GridItem { span: GridSpan::Half, "loose" } }
            }
        }
    }

    fn placed() -> Element {
        rsx! {
            LiberoProvider {
                Grid { template: test_template(),
                    GridZone { area: TestArea::Body, GridItem { span: GridSpan::Half, "placed" } }
                }
            }
        }
    }

    let loose = render(standalone);
    assert!(
        !attributes_of(&nth_div(&loose, 0), "div")
            .get("data-state")
            .is_some_and(|state| state.contains("container"))
    );

    let html = render(placed);
    assert!(
        attributes_of(&nth_div(&html, 1), "div")
            .get("data-state")
            .expect("the zone states")
            .contains("container")
    );
    assert!(has_rule_for(&html, "lsx-",));
    assert!(html.contains("[data-state~=\"container\"]{container-type:inline-size;}"));
}

#[test]
fn a_plain_span_emits_no_container_query() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                GridZone { GridItem { span: GridSpan::Half, "card" } }
            }
        }
    }

    assert!(!render(app).contains("@container"));
}

#[test]
fn an_orphan_item_still_renders_its_children() {
    fn app() -> Element {
        rsx! { LiberoProvider { GridItem { "loose" } } }
    }

    assert!(body(&render(app)).contains("loose"));
}

/// The zone places its children from *its* stylesheet while the item sets its
/// own `grid-column` from another. Both are on the framework layer, which the
/// registry emits in hash order, so only specificity settles the two - (0,3,0)
/// against (0,2,0). Loosening the zone selector to `& > *` would invert it.
#[test]
fn the_zone_places_items_at_a_higher_specificity_than_the_item_styles_itself() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                GridZone { masonry: true, GridItem { span: GridSpan::Half, "card" } }
            }
        }
    }

    let html = render(app);

    assert!(html.contains(
        "[data-state~=\"masonry\"] > [data-state~=\"grid-item\"][data-state~=\"measured\"]"
    ));
    assert!(html.contains("[data-state~=\"span-half\"]"));
}

/// `rows` is the caller's own row span, and it writes a **different variable**
/// from the masonry engine's: one property, one writer. Outside a masonry zone
/// the item sets `grid-row` from its own stylesheet.
#[test]
fn an_item_can_span_rows_of_its_own() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                GridZone { GridItem { rows: 2u8, "tall" } }
            }
        }
    }

    let html = render(app);
    let item = attributes_of(&nth_div(&html, 1), "div");

    assert!(
        item.get("data-state")
            .is_some_and(|state| state.contains("rows")),
        "{html}"
    );
    assert!(
        item.get("style")
            .expect("the row span")
            .contains("--lsx-grid-item-row-span:2")
    );
    assert!(html.contains("grid-row:span var(--lsx-grid-item-row-span, 1)"));
}

/// And the masonry engine wins by the item simply not writing it: the span
/// there is derived from a measured height, so a manual one would leave the
/// item overlapping its neighbours. The two rules never meet on an element.
#[test]
fn a_masonry_zone_drops_an_items_own_row_span() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                GridZone { masonry: true, GridItem { rows: 2u8, "tall" } }
            }
        }
    }

    let html = render(app);
    let item = attributes_of(&nth_div(&html, 1), "div");

    assert!(
        !item
            .get("data-state")
            .is_some_and(|state| state.split_whitespace().any(|token| token == "rows")),
        "{html}"
    );
    assert!(
        !item
            .get("style")
            .is_some_and(|style| style.contains("--lsx-grid-item-row-span")),
        "{html}"
    );
}

/// `gap` has to reach the item's `margin-bottom` and the zone's cancelling
/// margin, not just the `gap` shorthand - in masonry the row gap is zero and
/// the vertical spacing is entirely that margin. A per-size class could only
/// change the shorthand, so the resolved value is published as the var itself.
#[test]
fn a_zones_gap_reaches_the_vertical_spacing_masonry_actually_uses() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                GridZone { masonry: true, gap: Size::Xl, GridItem { "card" } }
            }
        }
    }

    let html = render(app);
    let zone = attributes_of(&nth_div(&html, 0), "div");

    assert!(
        zone.get("style")
            .expect("the resolved gap")
            .contains("--lsx-grid-zone-gap:var(--lsx-spacing-xl)")
    );
    assert!(html.contains("margin-bottom:var(--lsx-grid-zone-gap)"));
    assert!(html.contains("margin-bottom:calc(-1 * var(--lsx-grid-zone-gap))"));
}

/// Twelve tracks carry eleven gaps whatever spans them, and a length gap does
/// not shrink - so a zone narrower than `11 * gap` would overflow its area.
/// The percentage cap resolves against the zone's own width and removes the
/// floor; `min-width: 0` is what lets the zone shrink at all, since a grid
/// item defaults to `min-width: auto`.
#[test]
fn a_zone_can_shrink_below_the_width_its_twelve_tracks_would_demand() {
    fn app() -> Element {
        rsx! { LiberoProvider { GridZone { GridItem { "card" } } } }
    }

    let html = render(app);

    assert!(html.contains("min-width:0"));
    assert!(html.contains("column-gap:min(var(--lsx-grid-zone-gap), 4%)"));
    assert!(html.contains("row-gap:var(--lsx-grid-zone-gap)"));
}
