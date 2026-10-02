use std::collections::BTreeMap;

use crate::common::{body, css_rules_for, render, tag_with};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Splitter};

fn two_panes() -> Element {
    rsx! {
        LiberoProvider {
            Splitter {
                initial_size: 50.0,
                panel_a: rsx! { div { id: "a-content", "left" } },
                panel_b: rsx! { div { id: "b-content", "right" } },
            }
        }
    }
}

/// The `<div>` opening just before the tag carrying `marker`: its wrapper or previous sibling.
fn div_before(html: &str, marker: &str) -> BTreeMap<String, String> {
    let at = html.find(marker).expect("the marker");
    let open = html[..at].rfind('<').expect("the marked tag");
    let before = html[..open].rfind("<div").expect("a div before it");
    tag_with(&html[before..], "<div")
}

/// Every declaration `property` the element's rules set, at-rules included.
fn declared(html: &str, element: &BTreeMap<String, String>, property: &str) -> Vec<String> {
    css_rules_for(html, element)
        .into_iter()
        .filter_map(|rule| rule.declarations.get(property).cloned())
        .collect()
}

#[test]
fn both_panes_scroll_their_own_overflow() {
    // A pane at its floor otherwise paints over the divider and the other pane (todo 1581).
    let html = render(two_panes);

    for content in [r#"id="a-content""#, r#"id="b-content""#] {
        let pane = div_before(&html, content);
        assert_eq!(declared(&html, &pane, "overflow"), ["auto"], "{pane:?}");
    }
}

#[test]
fn the_default_divider_line_is_the_3_to_1_boundary_shade() {
    // muted.4 was about 1.5:1 on the page; muted.6 is 3:1 (WCAG 1.4.11, todo 1580).
    let html = render(two_panes);
    let bar = div_before(&html, r#"role="separator""#);

    let background = declared(&html, &bar, "background").join(" ");
    assert!(background.contains("muted-6"), "{background}");
}

#[test]
fn splitter_renders_both_panes_around_a_divider() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Splitter {
                    initial_size: 50.0,
                    panel_a: rsx! { div { "left" } },
                    panel_b: rsx! { div { "right" } },
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("left"));
    assert!(body.contains("right"));
    assert!(body.contains("--lsx-splitter-a:50%;"));
}

/// `touch-action: none` on the drag target is what lets a touch drag start
/// at all - without it the browser claims the gesture for scrolling and no
/// `pointermove` ever arrives. Nothing visible regresses if it goes missing,
/// so assert it reaches the rendered CSS. The drag itself needs real pointer
/// input, which SSR cannot produce.
#[test]
fn splitter_hit_target_opts_out_of_browser_touch_gestures() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Splitter {
                    initial_size: 50.0,
                    panel_a: rsx! { div { "left" } },
                    panel_b: rsx! { div { "right" } },
                }
            }
        }
    }

    assert!(render(app).contains("touch-action:none"));
}

/// `min_size` floors *both* panes, so anything past 50 leaves `f64::clamp`
/// with `min > max` and panicked the whole subtree.
#[test]
fn splitter_survives_a_min_size_past_the_midpoint() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Splitter {
                    initial_size: 50.0,
                    min_size: 70.0,
                    panel_a: rsx! { div { "left" } },
                    panel_b: rsx! { div { "right" } },
                }
            }
        }
    }

    let body = body(&render(app));

    assert!(body.contains("left") && body.contains("right"));
    assert!(body.contains("aria-valuemin=\"50\""));
    assert!(body.contains("--lsx-splitter-a:50%;"));
}

/// `f64::clamp` panics on a `NaN` bound, and keeps a `NaN` receiver - so a
/// `min_size` of `NaN` took the page down and an `initial_size` of `NaN`
/// (`a / (a + b)` with both at 0) wrote `--lsx-splitter-a:NaN%`, leaving both
/// panes without a size. Both fall back with a warning instead.
#[test]
fn a_non_finite_size_falls_back_instead_of_panicking() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Splitter {
                    min_size: f64::NAN,
                    initial_size: f64::NAN,
                    aria_label: "Resize",
                    panel_a: rsx! { div { "left" } },
                    panel_b: rsx! { div { "right" } },
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("left"));
    assert!(body.contains("--lsx-splitter-a:50%;"));
    assert!(!body.contains("NaN"));
}
