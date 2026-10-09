use crate::common::{attributes_of, body, classes_of, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Flex, FlexDirection},
    sx::ThemeAwareValue,
    theme::{Size, responsive},
};

#[test]
fn flex_renders_its_children_in_order() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Flex {
                    span { "first" }
                    span { "second" }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let first = body.find("first").expect("the first child");
    let second = body.find("second").expect("the second child");

    assert!(first < second);
    assert!(!classes_of(&html, "div").is_empty());
}

/// The rules inside one viewport breakpoint, whitespace stripped.
fn media_block<'a>(css: &'a str, width: &str) -> &'a str {
    let open = format!("@media(min-width:{width}){{");
    let start = css
        .find(&open)
        .unwrap_or_else(|| panic!("no {open} in the sheet"))
        + open.len();
    &css[start..start + css[start..].find("}}").expect("a closed block")]
}

/// Whether one of the `div`'s classes opens a rule starting with `rule` in `css`.
fn has_rule(body: &str, css: &str, rule: &str) -> bool {
    classes_of(body, "div")
        .iter()
        .any(|class| css.contains(&format!(".{class}{{{rule}")))
}

fn has_state(body: &str, name: &str) -> bool {
    attributes_of(body, "div")
        .get("data-state")
        .is_some_and(|state| state.split_whitespace().any(|state| state == name))
}

#[test]
fn a_gap_word_rides_the_state_class() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Flex { gap: "sm", span { "a" } } }
        }
    }
    let html = render(app);

    assert!(has_state(&body(&html), "size-sm"), "{html}");
}

#[test]
fn a_gap_takes_any_css_value() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Flex { gap: "0", span { "a" } } }
        }
    }
    let html = render(app);
    let css = html.replace(char::is_whitespace, "");

    assert!(has_rule(&body(&html), &css, "gap:0"), "{css}");
}

#[test]
fn a_responsive_gap_overrides_from_each_breakpoint() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Flex {
                    gap: responsive(ThemeAwareValue::from(Size::Xs))
                        .md(Size::Lg.into())
                        .xl("3px".into()),
                    span { "a" }
                }
            }
        }
    }
    let html = render(app);
    let body = body(&html);
    let css = html.replace(char::is_whitespace, "");

    assert!(
        has_state(&body, "size-xs"),
        "the base word keeps its state: {html}"
    );
    assert!(has_rule(&body, media_block(&css, "62rem"), "gap:"), "{css}");
    assert!(
        has_rule(&body, media_block(&css, "88rem"), "gap:3px"),
        "{css}"
    );
}

/// The size-only form, the common one, needs no `ThemeAwareValue`.
#[test]
fn a_responsive_size_gap_converts() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Flex { gap: responsive(Size::Xs).sm(Size::Xl), span { "a" } }
            }
        }
    }
    let html = render(app);
    let css = html.replace(char::is_whitespace, "");

    assert!(
        has_rule(&body(&html), media_block(&css, "48rem"), "gap:"),
        "{css}"
    );
}

/// Todo 2689: a column on a phone, a row from `md`, keeping the explicit gap there.
#[test]
fn a_responsive_direction_turns_at_its_breakpoint() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Flex {
                    direction: responsive(FlexDirection::Column).md(FlexDirection::Row),
                    gap: "xs",
                    span { "a" }
                }
            }
        }
    }
    let html = render(app);
    let body = body(&html);
    let css = html.replace(char::is_whitespace, "");
    let md = media_block(&css, "62rem");

    assert!(!has_state(&body, "row"), "the base is a column: {html}");
    assert!(has_rule(&body, md, "flex-direction:row"), "{css}");
    assert!(
        !md.contains("--lsx-flex-row-spacing"),
        "the explicit gap replaces the row default: {md}"
    );
}

#[test]
fn a_responsive_direction_without_a_gap_takes_the_axis_spacing() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Flex {
                    direction: responsive(FlexDirection::Row).sm(FlexDirection::Column),
                    span { "a" }
                }
            }
        }
    }
    let html = render(app);
    let body = body(&html);
    let css = html.replace(char::is_whitespace, "");
    let sm = media_block(&css, "48rem");

    assert!(has_state(&body, "row"), "{html}");
    assert!(has_rule(&body, sm, "flex-direction:column"), "{css}");
    assert!(sm.contains("--lsx-flex-column-spacing"), "{sm}");
}
