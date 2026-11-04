//! Responsive `orientation` (todo 5656): run words and range copies, so the layout is exact at
//! SSR, while keys and `aria-orientation` render the base until mount. A plain value adds nothing.

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        ActionIcon, Button, ButtonGroup, Carousel, DataList, DataListItem, Divider, Marquee,
        Options, Orientation, RadioGroup, RangeSlider, SegmentedControl, Slider, Sortable,
        SortableItem, Splitter, Stepper, Toolbar,
    },
    theme::{ChoiceVariant, responsive},
};

/// Any range copy in the page.
fn has_range_copy(html: &str) -> bool {
    html.split(r#"[data-state~=""#)
        .skip(1)
        .any(|rest| rest[..rest.find('"').unwrap_or(0)].contains(':'))
}

/// The CSS of the copy keyed on `word`, up to the end of its `@media` block.
fn copy<'a>(html: &'a str, word: &str) -> &'a str {
    let at = html
        .find(&format!(r#"[data-state~="{word}"]"#))
        .unwrap_or_else(|| panic!("no copy for {word}"));
    let rest = &html[at..];
    &rest[..rest.find("</style>").unwrap_or(rest.len())]
}

fn stacked() -> libero::theme::Responsive<Orientation> {
    responsive(Orientation::Vertical).md(Orientation::Horizontal)
}

#[test]
fn a_plain_orientation_keeps_its_words_and_registers_nothing() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Toolbar { "aria-label": "Tools", orientation: "vertical",
                    ActionIcon { aria_label: "Bold", "B" }
                }
                ButtonGroup { "aria-label": "Align", orientation: "vertical", Button { "Left" } }
                DataList { orientation: "horizontal",
                    DataListItem { label: rsx! { "Name" }, "Ada" }
                }
                Sortable { orientation: "horizontal", onreorder: |_| {},
                    SortableItem { index: 0, label: "One", "One" }
                }
                Splitter { aria_label: "Panes", orientation: "horizontal",
                    panel_a: rsx! { "A" }, panel_b: rsx! { "B" },
                }
                RadioGroup::<Plan> { label: "Plan", orientation: "horizontal", onchange: |_| {} }
                RadioGroup::<Plan> { label: "Tier", orientation: "horizontal", variant: "card", onchange: |_| {} }
                SegmentedControl::<Plan> { "aria-label": "Plan", orientation: "vertical", value: Plan::Free, onchange: |_| {} }
                Divider { orientation: "vertical", "or" }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    assert!(!has_range_copy(&html), "a range copy on a plain page");
    assert!(page.contains(r#"aria-orientation="vertical""#), "{page}");
}

#[test]
fn a_responsive_toolbar_draws_its_row_from_css_and_keys_from_the_base() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Toolbar { "aria-label": "Tools", orientation: stacked(),
                    ActionIcon { aria_label: "Bold", "B" }
                }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    assert!(page.contains("base-md:vertical md-:horizontal"), "{page}");
    // The server knows no breakpoint: the base's keys and `aria-orientation`.
    assert!(page.contains(r#"aria-orientation="vertical""#), "{page}");
    assert!(copy(&html, "base-md:vertical").contains("flex-direction:column"));
    assert!(copy(&html, "md-:horizontal").contains("flex-wrap:wrap"));
}

#[test]
fn a_responsive_button_group_copies_its_seams_per_range() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ButtonGroup { "aria-label": "Align", orientation: stacked(),
                    Button { "Left" }
                    Button { "Right" }
                }
            }
        }
    }

    let html = render(app);
    assert!(copy(&html, "base-md:vertical").contains("flex-direction:column"));
    assert!(copy(&html, "md-:horizontal").contains("margin-inline-start:-1px"));
}

#[test]
fn a_responsive_data_list_turns_into_a_grid_from_its_range() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DataList { orientation: stacked(),
                    DataListItem { label: rsx! { "Name" }, "Ada" }
                }
            }
        }
    }

    let html = render(app);
    assert!(body(&html).contains("md-:horizontal"));
    assert!(copy(&html, "md-:horizontal").contains("display:grid"));
    // The column is the static sheet's: its range needs no rule.
    assert!(!html.contains(r#"[data-state~="base-md:vertical"]"#));
}

#[test]
fn a_responsive_splitter_copies_its_root_bar_and_hit_zone() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Splitter {
                    aria_label: "Panes",
                    orientation: responsive(Orientation::Horizontal).md(Orientation::Vertical),
                    panel_a: rsx! { "A" },
                    panel_b: rsx! { "B" },
                }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    assert!(page.contains(r#"aria-orientation="horizontal""#), "{page}");
    assert!(
        html.matches(r#"[data-state~="md-:vertical"]"#).count() >= 3,
        "root, bar and hit copies"
    );
    assert!(html.contains("col-resize") && html.contains("row-resize"));
}

#[test]
fn a_responsive_sortable_scrolls_its_row_only_in_its_range() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Sortable { orientation: stacked(), onreorder: |_| {},
                    SortableItem { index: 0, label: "One", "One" }
                }
            }
        }
    }

    let html = render(app);
    assert!(copy(&html, "md-:horizontal").contains("overflow-x:auto"));
    assert!(copy(&html, "base-md:vertical").contains("flex-direction:column"));
}

#[derive(Clone, Copy, PartialEq, Options)]
enum Stage {
    Account,
    Review,
}

/// Laid out in Rust: the server draws the base, a vertical rail with content under its step.
#[test]
fn a_responsive_stepper_renders_its_base_on_the_server() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Stepper {
                    value: Some(Stage::Account),
                    orientation: stacked(),
                    panel: |_: Stage| rsx! { "Body" },
                }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    assert!(page.contains("vertical"), "{page}");
    assert!(!has_range_copy(&html));
}

#[derive(Clone, Copy, PartialEq, Options)]
enum Plan {
    Free,
    Pro,
}

#[test]
fn a_responsive_radio_group_wraps_its_row_only_in_its_range() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                RadioGroup::<Plan> { label: "Plan", orientation: stacked(), variant: "card", onchange: |_| {} }
            }
        }
    }

    let html = render(app);
    assert!(body(&html).contains("base-md:vertical md-:horizontal"));
    let row = copy(&html, "md-:horizontal");
    assert!(row.contains("flex-wrap:wrap"), "{row}");
    // `horizontal && cards` keeps its cards half inside the copy.
    assert!(
        row.contains(r#"[data-state~="cards"] > *{flex:1 1 12rem;}"#),
        "{row}"
    );
    // The column is the static sheet's: its range needs no rule.
    assert!(!html.contains(r#"[data-state~="base-md:vertical"]"#));
}

#[test]
fn a_responsive_divider_turns_its_line_per_range_and_names_the_base() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Divider { orientation: stacked(), "or" }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    assert!(page.contains("base-md:vertical md-:horizontal"), "{page}");
    assert!(page.contains(r#"aria-orientation="vertical""#), "{page}");
    let column = copy(&html, "base-md:vertical");
    assert!(column.contains("flex-direction:column"), "{column}");
    // The label span's class is the base's; the row copy restyles it.
    let row = copy(&html, "md-:horizontal");
    assert!(
        row.contains("[data-slot='label']") && row.contains("overflow-wrap:anywhere"),
        "{row}"
    );
}

#[test]
fn a_responsive_marquee_scrolls_on_the_axis_of_its_range() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Marquee { orientation: stacked(), span { "Rust" } }
            }
        }
    }

    let html = render(app);
    assert!(body(&html).contains("base-md:vertical md-:horizontal"));
    assert!(copy(&html, "base-md:vertical").contains("translateY("));
    assert!(copy(&html, "md-:horizontal").contains("translateX("));
}

/// Laid out in Rust (controls, scrollbars, drag axis): the server draws the base.
#[test]
fn a_responsive_carousel_renders_its_base_on_the_server() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel {
                    aria_label: "Photos",
                    orientation: stacked(),
                    height: "240px",
                    slides: vec![rsx! { "One" }, rsx! { "Two" }],
                }
            }
        }
    }

    let html = render(app);
    assert!(body(&html).contains("vertical"));
    assert!(!has_range_copy(&html));
}

/// Laid out in Rust (bars, segments, drag axis): the server draws the base.
#[test]
fn a_responsive_slider_renders_its_base_on_the_server() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Slider::<f64> { label: "Volume", value: 40.0, orientation: stacked() }
                RangeSlider::<f64> { label: "Price", value: (20.0, 60.0), orientation: stacked() }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    assert_eq!(
        page.matches(r#"aria-orientation="vertical""#).count(),
        3,
        "{page}"
    );
    assert!(!has_range_copy(&html));
}

/// Both axes per breakpoint: a third run word keys the cards' row on the ranges they share.
#[test]
fn a_radio_group_with_both_axes_responsive_grows_its_cards_where_they_meet() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                RadioGroup::<Plan> {
                    label: "Plan",
                    orientation: stacked(),
                    variant: responsive(ChoiceVariant::Plain).lg(ChoiceVariant::Card),
                    onchange: |_| {},
                }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    assert!(page.contains("base-lg:no-card-row lg-:card-row"), "{page}");
    let row = copy(&html, "lg-:card-row");
    assert!(
        row.starts_with(r#"[data-state~="lg-:card-row"] > *{flex:1 1 12rem;}"#),
        "{row}"
    );
    assert!(!html.contains(r#"[data-state~="base-lg:no-card-row"]"#));
}

#[test]
fn a_responsive_segmented_control_stacks_and_joins_per_range() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                SegmentedControl::<Plan> {
                    "aria-label": "Plan",
                    orientation: stacked(),
                    full_width: true,
                    value: Plan::Free,
                    onchange: |_| {},
                }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    assert!(page.contains("base-md:vertical md-:horizontal"), "{page}");
    let column = copy(&html, "base-md:vertical");
    assert!(column.contains("flex-direction:column"), "{column}");
    assert!(column.contains("margin-top:-1px"), "{column}");
    let row = copy(&html, "md-:horizontal");
    assert!(row.contains("margin-inline-start:-1px"), "{row}");
    assert!(row.contains("flex:1 1 0"), "{row}");
}
