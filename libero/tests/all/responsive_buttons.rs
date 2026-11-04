//! Responsive props of the buttons family, Paper, Alert and Avatar (todo 5648): run words and
//! range copies for a responsive value, nothing for a plain one.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{ActionIcon, Alert, Avatar, Button, Chip, Paper},
    sx::ThemeAwareValue,
    theme::{Size, Variant, responsive},
};

/// Any range copy or visibility rule in the page.
fn has_lazy_css(html: &str) -> bool {
    html.split(r#"[data-state~=""#).skip(1).any(|rest| {
        let word = &rest[..rest.find('"').unwrap_or(0)];
        word.contains(':') || word.starts_with("hidden-from") || word.starts_with("visible-from")
    })
}

#[test]
fn a_responsive_action_icon_sizes_per_range_in_the_user_layer() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ActionIcon {
                    aria_label: "Close",
                    size: responsive(ThemeAwareValue::from(Size::Sm)).md(Size::Xl.into()),
                    variant: responsive(Variant::Outlined).md(Variant::Filled),
                    hidden_from: "xl",
                    "x"
                }
            }
        }
    }

    let html = render(app);
    let button = attributes_of(&body(&html), "button");
    let state = &button["data-state"];
    for word in [
        "base-md:compact",
        "md-:grows",
        "base-md:outlined",
        "md-:filled",
        "hidden-from-xl",
    ] {
        assert!(state.split(' ').any(|s| s == word), "{word} in {state}");
    }
    // The size is no inline var: the user layer carries it per range.
    let style = button.get("style").cloned().unwrap_or_default();
    assert!(!style.contains("action-icon-size-override"), "{style}");
    assert!(
        style.contains("--lsx-action-icon-hover-v-filled"),
        "{style}"
    );
    assert!(html.contains(r#"[data-state~="md-:filled"]"#));
    assert!(html.contains(r#"[data-state~="base-md:compact"]"#));
    assert!(!html.contains("touch-fixed"));
}

#[test]
fn a_plain_action_icon_and_chip_carry_no_range_copy() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ActionIcon { aria_label: "Close", size: "sm", variant: "filled", "x" }
                Chip { size: "lg", variant: "outlined", radius: "0", "Tag" }
            }
        }
    }

    let html = render(app);
    assert!(!has_lazy_css(&html), "lazy CSS on a plain page");
    let style = attributes_of(&body(&html), "button")
        .get("style")
        .cloned()
        .unwrap_or_default();
    assert!(style.contains("action-icon-size-override"), "{style}");
}

#[test]
fn a_responsive_chip_renders_its_run_words_and_copies() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Chip {
                    size: responsive(Size::Xs).md(Size::Lg),
                    variant: responsive(Variant::Outlined).md(Variant::Filled),
                    radius: responsive(ThemeAwareValue::from(Size::Sm)).md(Size::Xl.into()),
                    "Tag"
                }
            }
        }
    }

    let html = render(app);
    let state = &attributes_of(&body(&html), "span")["data-state"];
    for word in [
        "md-:size-lg",
        "md-:filled",
        "md-:radius-xl",
        "base-md:size-xs",
    ] {
        assert!(state.split(' ').any(|s| s == word), "{word} in {state}");
    }
    assert!(html.contains(r#"[data-state~="md-:size-lg"]"#));
    assert!(html.contains("--lsx-chip-hover-v-filled"));
}

#[test]
fn paper_alert_and_avatar_render_their_run_words_and_copies() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Paper {
                    radius: responsive(ThemeAwareValue::from(Size::Xs)).md(Size::Xl.into()),
                    shadow: responsive(ThemeAwareValue::from(Size::Sm)).lg(Size::Lg.into()),
                    "Card"
                }
                Alert { variant: responsive(Variant::Outlined).md(Variant::Filled), "Note" }
                Avatar {
                    size: responsive(Size::Sm).md(Size::Xl),
                    variant: responsive(Variant::Outlined).md(Variant::Filled),
                    name: "Ada",
                    initials: "AL",
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    for word in [
        "md-:radius-xl",
        "lg-:shadow-lg",
        "md-:filled",
        "md-:size-xl",
        "base-md:outlined",
    ] {
        assert!(body.contains(word), "{word} in the markup");
        assert!(
            html.contains(&format!(r#"[data-state~="{word}"]"#)),
            "no copy for {word}"
        );
    }
}

#[test]
fn plain_paper_alert_and_avatar_carry_no_range_copy() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Paper { radius: "lg", shadow: "sm", "Card" }
                Alert { variant: "filled", radius: "0", "Note" }
                Avatar { size: "lg", variant: "outlined", name: "Ada", initials: "AL" }
            }
        }
    }

    assert!(!has_lazy_css(&render(app)), "lazy CSS on a plain page");
}

/// Todo 5649: a responsive start or end loader is sized per range by the button's copies.
#[test]
fn a_responsive_inline_loader_takes_half_the_button_height_per_range() {
    fn responsive_app() -> Element {
        rsx! {
            LiberoProvider {
                Button {
                    loading: true,
                    loading_position: "start",
                    size: responsive(Size::Xs).md(Size::Xl),
                    "Save"
                }
            }
        }
    }
    fn plain_app() -> Element {
        rsx! {
            LiberoProvider {
                Button { loading: true, loading_position: "end", size: "xl", "Save" }
            }
        }
    }

    let html = render(responsive_app);
    let rule = "{--lsx-loader-size:calc(var(--lsx-button-height-xl) / 2);}";
    assert!(
        html.split(r#"[data-state~="md-:size-xl"]"#)
            .skip(1)
            .any(|rest| rest.contains("[data-slot='button-icon'] > span") && rest.contains(rule)),
        "no md copy for the inline loader"
    );
    assert!(
        !html.contains("@layer lsx-user-static"),
        "the base sx outranks the copies"
    );

    let html = render(plain_app);
    assert!(!has_lazy_css(&html), "lazy CSS on a plain page");
    assert!(html.contains(rule), "the plain loader lost its sx");
}
