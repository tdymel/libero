//! Todo 2663: a `radius` takes any CSS next to the size scale. A size keeps its
//! `radius-<size>` state; custom CSS keeps the theme's step and overrides its value.

use crate::common::{body, render, tags_with};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        Alert, Badge, Button, ButtonGroup, Checkbox, Chip, ColorCode, ColorSwatch, Paper,
        ProgressBar, Skeleton, Switch, TextField, Textarea,
    },
};

const OVERRIDE: &str = "--lsx-radius-override:";

/// The `style` of each element marked `data-radius`.
fn styles(html: &str) -> Vec<String> {
    tags_with(&body(html), "data-radius")
        .into_iter()
        .map(|tag| tag.get("style").cloned().unwrap_or_default())
        .collect()
}

#[test]
fn a_custom_radius_overrides_the_step_on_each_state_class_component() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button { "data-radius": "", radius: "0", "b" }
                Chip { "data-radius": "", radius: "3px", "c" }
                Paper { "data-radius": "", radius: "0", "p" }
                Skeleton { "data-radius": "", radius: "0" }
                ColorSwatch { "data-radius": "", color: ColorCode::default(), radius: "0" }
            }
        }
    }
    let html = render(app);
    let styles = styles(&html);

    assert_eq!(styles.len(), 5, "{html}");
    for style in &styles {
        assert!(style.contains(OVERRIDE), "no override in `{style}`: {html}");
    }
    assert!(
        styles[1].contains(&format!("{OVERRIDE}3px")),
        "{}",
        styles[1]
    );
    assert!(
        html.contains("var(--lsx-radius-override,var(--lsx-radius-")
            || html.contains("var(--lsx-radius-override, var(--lsx-radius-"),
        "the step reads the override: {html}"
    );
}

#[test]
fn a_size_radius_writes_no_override() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button { "data-radius": "", radius: "lg", "b" }
                Paper { "data-radius": "", radius: "lg", "p" }
            }
        }
    }
    let html = render(app);

    for tag in tags_with(&body(&html), "data-radius") {
        assert!(
            tag.get("data-state")
                .is_some_and(|state| state.split_whitespace().any(|s| s == "radius-lg")),
            "{tag:?}"
        );
        assert!(
            !tag.get("style")
                .is_some_and(|style| style.contains(OVERRIDE)),
            "{tag:?}"
        );
    }
}

#[test]
fn a_group_s_custom_radius_reaches_its_buttons() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ButtonGroup { "aria-label": "Pair", radius: "0",
                    Button { "data-radius": "", "a" }
                    Button { "data-radius": "", "b" }
                }
            }
        }
    }
    let html = render(app);
    let styles = styles(&html);

    assert_eq!(styles.len(), 2, "{html}");
    assert!(
        styles
            .iter()
            .all(|style| style.contains(&format!("{OVERRIDE}0"))),
        "{styles:?}"
    );
}

#[test]
fn a_custom_radius_fills_the_component_s_own_scale_var() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Badge { "data-radius": "", radius: "0", "b" }
                Alert { "data-radius": "", radius: "0", "a" }
                ProgressBar { "data-radius": "", value: 40.0, radius: "0" }
            }
        }
    }
    let html = render(app);

    assert!(html.contains("--lsx-badge-radius-override:0"), "{html}");
    assert!(html.contains("--lsx-alert-radius-override:0"), "{html}");
    assert!(
        html.contains(&format!("{OVERRIDE}0")),
        "progress track: {html}"
    );
}

/// Todo 2721: a field's radius takes any CSS. The wrapper sets the override; the frame and
/// the controls reset it per `radius-*` rule, so they inherit it back.
#[test]
fn a_custom_field_radius_reaches_the_frame_and_the_controls() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField { label: "Name", radius: "0" }
                Textarea { label: "Note", radius: "3px" }
                Checkbox { label: "Check", radius: "0" }
                Switch { label: "Switch", radius: "0" }
            }
        }
    }
    let html = render(app);

    assert_eq!(html.matches(&format!("{OVERRIDE}0")).count(), 3, "{html}");
    assert!(html.contains(&format!("{OVERRIDE}3px")), "{html}");
    assert!(html.contains(&format!("{OVERRIDE}inherit")), "{html}");
}

#[test]
fn a_field_radius_size_sets_no_override() {
    fn app() -> Element {
        rsx! { LiberoProvider { TextField { label: "Name", radius: "lg" } } }
    }
    let html = render(app);

    assert!(html.contains("radius-lg"), "{html}");
    assert!(!html.contains(&format!("{OVERRIDE}lg")), "{html}");
}
