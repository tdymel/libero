//! Responsive axis props (todo 5643): one `data-state` word per viewport run, and the range
//! copies registered on first use, so a page without a responsive value carries none.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Badge, Button, Options, Tabs},
    sx::ThemeAwareValue,
    theme::{Size, TabsPlacement, Variant, responsive},
};

/// Any range copy or visibility rule in the page.
fn has_lazy_css(html: &str) -> bool {
    html.split(r#"[data-state~=""#).skip(1).any(|rest| {
        let word = &rest[..rest.find('"').unwrap_or(0)];
        word.contains(':') || word.starts_with("hidden-from") || word.starts_with("visible-from")
    })
}

#[test]
fn a_plain_page_carries_no_range_copy() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button { size: "lg", variant: "outlined", radius: "0", "Save" }
                Badge { size: "sm", "New" }
            }
        }
    }

    let html = render(app);
    let state = &attributes_of(&body(&html), "button")["data-state"];
    assert!(state.contains("size-lg"), "{state}");
    assert!(!state.contains(':'), "{state}");
    assert!(!has_lazy_css(&html), "lazy CSS on a plain page");
}

#[test]
fn a_responsive_button_carries_one_word_per_run_and_its_copies() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button {
                    size: responsive(Size::Sm).md(Size::Xl),
                    variant: responsive(Variant::Outlined).lg(Variant::Filled),
                    "Save"
                }
            }
        }
    }

    let html = render(app);
    let button = attributes_of(&body(&html), "button");
    let state = &button["data-state"];
    for word in [
        "base-md:size-sm",
        "md-:size-xl",
        "base-lg:outlined",
        "lg-:filled",
    ] {
        assert!(state.split(' ').any(|s| s == word), "{word} in {state}");
    }
    assert!(!state.split(' ').any(|s| s == "size-sm"), "{state}");
    // Copies are rooted at the base sheet's class, under their own run's query.
    let copy = html
        .find(r#"[data-state~="md-:size-xl"]"#)
        .expect("no md copy in the page");
    assert!(html.contains("(min-width: 62rem)"));
    assert!(!html.contains(r#"[data-state~="base-md:size-xl"]"#));
    // Ahead of the base sheet, whose state rules win a tie.
    let base = html
        .find(r#"[data-state~="full-width"]"#)
        .expect("no base sheet");
    assert!(copy < base, "the copy sorts after the base sheet");
    assert!(button["style"].contains("--lsx-button-hover-v-filled"));
}

#[test]
fn two_buttons_with_one_value_share_its_copies() {
    fn one() -> Element {
        rsx! {
            LiberoProvider { Button { size: responsive(Size::Sm).md(Size::Xl), "A" } }
        }
    }
    fn two() -> Element {
        rsx! {
            LiberoProvider {
                Button { size: responsive(Size::Sm).md(Size::Xl), "A" }
                Button { size: responsive(Size::Sm).md(Size::Xl), "A" }
            }
        }
    }

    let (one, two) = (render(one), render(two));
    assert_eq!(
        one.matches(r#"[data-state~="md-:size-xl"]"#).count(),
        two.matches(r#"[data-state~="md-:size-xl"]"#).count()
    );
}

#[test]
fn a_custom_radius_per_breakpoint_rides_the_user_layer() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button { radius: responsive(ThemeAwareValue::from(Size::Sm)).md("0".into()), "Save" }
            }
        }
    }

    let html = render(app);
    let button = attributes_of(&body(&html), "button");
    // The custom range keeps the theme's step word; the CSS replaces its value.
    let state = &button["data-state"];
    assert!(state.contains("base-md:radius-sm"), "{state}");
    assert!(state.contains("md-:radius-md"), "{state}");
    assert!(
        !button
            .get("style")
            .is_some_and(|style| style.contains("override"))
    );
    assert!(
        html.contains("@layer lsx-user-static"),
        "no user-layer rule"
    );
}

#[test]
fn a_responsive_badge_renders_its_run_words() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Badge {
                    size: responsive(Size::Xs).md(Size::Lg),
                    variant: responsive(Variant::Outlined).md(Variant::Tonal),
                    "New"
                }
            }
        }
    }

    let html = render(app);
    let state = &attributes_of(&body(&html), "span")["data-state"];
    assert!(state.contains("md-:size-lg"), "{state}");
    assert!(state.contains("md-:tonal"), "{state}");
    assert!(html.contains(r#"[data-state~="md-:tonal"]"#));
}

#[derive(Clone, Copy, PartialEq, Options)]
enum Section {
    One,
    Two,
}

#[test]
fn a_responsive_tabs_size_renders_its_run_words() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tabs {
                    aria_label: "Sections",
                    size: responsive(Size::Xs).md(Size::Lg),
                    value: Section::One,
                    panel: |_: Section| rsx! { "Panel" },
                }
            }
        }
    }

    let html = render(app);
    assert!(body(&html).contains("md-:size-lg"));
    assert!(html.contains(r#"[data-state~="md-:size-lg"]"#));
}

/// Todo 5653: the CSS half is exact at SSR; the keys' half renders the base until mount.
#[test]
fn a_responsive_tabs_placement_draws_its_column_from_css_and_keys_from_the_base() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tabs {
                    aria_label: "Sections",
                    placement: responsive(TabsPlacement::Top).md(TabsPlacement::Start),
                    value: Section::One,
                    panel: |_: Section| rsx! { "Panel" },
                }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    let state = &attributes_of(&page, "div")["data-state"];
    for word in ["base-md:placement-top", "md-:placement-start"] {
        assert!(state.split(' ').any(|s| s == word), "{word} in {state}");
    }
    assert!(!state.split(' ').any(|s| s == "vertical"), "{state}");
    assert!(page.contains(r#"aria-orientation="horizontal""#));
    // `top` needs no rule; `start` gets the column under its own range.
    assert!(!html.contains(r#"[data-state~="base-md:placement-top"]"#));
    let copy = html
        .find(r#"[data-state~="md-:placement-start"]"#)
        .expect("no start copy");
    assert!(html[copy..].contains("flex-direction:column"));
}

#[test]
fn a_plain_tabs_placement_keeps_its_words_and_registers_nothing() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tabs {
                    aria_label: "Sections",
                    placement: "end",
                    value: Section::One,
                    panel: |_: Section| rsx! { "Panel" },
                }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    let state = &attributes_of(&page, "div")["data-state"];
    assert!(state.split(' ').any(|s| s == "vertical"), "{state}");
    assert!(state.split(' ').any(|s| s == "placement-end"), "{state}");
    assert!(page.contains(r#"aria-orientation="vertical""#));
    assert!(!has_lazy_css(&html));
}

#[test]
fn hidden_from_and_visible_from_register_one_rule_each() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button { hidden_from: "md", "Menu" }
                Badge { visible_from: "sm", "New" }
                Button { hidden_from: "md", "Again" }
            }
        }
    }

    let html = render(app);
    let state = &attributes_of(&body(&html), "button")["data-state"];
    assert!(
        state.split(' ').any(|word| word == "hidden-from-md"),
        "{state}"
    );
    let hidden =
        r#"@media (min-width: 62rem){[data-state~="hidden-from-md"]{display:none !important;}}"#;
    assert_eq!(html.matches(hidden).count(), 1, "one shared rule");
    assert!(html.contains(r#"@media (max-width: 47.99875rem){[data-state~="visible-from-sm"]"#));
}
