//! Responsive `orientation` (todo 5656): run words and range copies, so the layout is exact at
//! SSR, while keys and `aria-orientation` render the base until mount. A plain value adds nothing.

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        ActionIcon, Button, ButtonGroup, DataList, DataListItem, Options, Orientation, Sortable,
        SortableItem, Splitter, Stepper, Toolbar,
    },
    theme::responsive,
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
