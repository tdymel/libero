//! Responsive feedback, navigation and overlay props (todo 5655): axis props write run words and
//! their copies, value props declare each breakpoint in the user layer, plain values add nothing.

use crate::common::{attributes_of, body, render, tags_with};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        Burger, CircularProgress, Dialog, Loader, MenuItem, Menubar, MenubarMenu, Options,
        Pagination, ProgressBar, Skeleton, Stepper,
    },
    sx::ThemeAwareValue,
    theme::{BURGER_SIZE, DIALOG_SIZE, SKELETON_HEIGHT, Size, SizeCss, responsive},
};

const MD: &str = "@media (min-width: 62rem)";

#[derive(Clone, Debug, PartialEq, Options)]
enum Stage {
    Account,
    Review,
}

/// The run words and range copies in the page.
fn run_words(html: &str) -> Vec<&str> {
    let after = |marker| html.split(marker).skip(1);
    after(r#"data-state=""#)
        .chain(after(r#"[data-state~=""#))
        .map(|rest| &rest[..rest.find('"').unwrap_or(0)])
        .filter(|words| {
            words
                .split(' ')
                .any(|word| word.contains('-') && word.contains(':'))
        })
        .collect()
}

/// `word` is in the markup and has its copy in a sheet.
fn has_copy(html: &str, word: &str) {
    assert!(body(html).contains(word), "{word} in the markup");
    assert!(
        html.contains(&format!(r#"[data-state~="{word}"]"#)),
        "{word} has no copy"
    );
}

fn menus() -> Vec<MenubarMenu> {
    vec![MenubarMenu::new("File", vec![MenuItem::new("New").into()])]
}

#[test]
fn a_plain_feedback_navigation_and_overlay_page_carries_no_run_word() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Loader { size: "lg" }
                ProgressBar { aria_label: "Upload", value: 3.0, size: "lg", radius: "0" }
                CircularProgress { aria_label: "Upload", value: 3.0, size: "lg" }
                Skeleton { height: "40px", width: "8rem", radius: "md" }
                Pagination { total: 10, page: 4, aria_label: "Results", size: "lg", radius: "xs", onchange: |_| {} }
                Menubar { aria_label: "Main", size: "lg", radius: "0", menus: menus() }
                Stepper::<Stage> { aria_label: "Checkout", size: "lg", value: Some(Stage::Account) }
                Burger { size: "lg" }
                Dialog { aria_label: "Inline", size: "lg", radius: "0", "content" }
            }
        }
    }

    let html = render(app);
    let words = run_words(&html);
    assert!(words.is_empty(), "run words on a plain page: {words:?}");
    // Plain values keep their inline vars.
    let page = body(&html);
    for var in [
        SKELETON_HEIGHT.name().to_string(),
        DIALOG_SIZE.override_var().name().to_string(),
        BURGER_SIZE.override_var().name().to_string(),
    ] {
        assert!(page.contains(&format!("{var}:")), "{var} inline");
    }
    assert!(!html.contains("--lsx-pagination-arrow-size"));
}

#[test]
fn responsive_sizes_write_run_words_and_copies() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Loader { size: responsive(Size::Sm).md(Size::Lg) }
                ProgressBar { aria_label: "Upload", value: 3.0, size: responsive(Size::Xs).md(Size::Xl) }
                CircularProgress { aria_label: "Upload", value: 3.0, size: responsive(Size::Sm).md(Size::Xxl) }
                Stepper::<Stage> { aria_label: "Checkout", size: responsive(Size::Xs).md(Size::Md), value: Some(Stage::Account) }
                Menubar { aria_label: "Main", size: responsive(Size::Sm).md(Size::Xl), menus: menus() }
            }
        }
    }

    let html = render(app);
    for word in [
        "base-md:size-sm",
        "md-:size-lg",
        "base-md:size-xs",
        "md-:size-xl",
        "md-:size-xxl",
        "md-:size-md",
    ] {
        has_copy(&html, word);
    }
}

#[test]
fn a_responsive_radius_steps_per_range_and_declares_custom_css_per_breakpoint() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ProgressBar {
                    aria_label: "Upload",
                    value: 3.0,
                    radius: responsive(ThemeAwareValue::from(Size::Xs)).md("0".into()),
                }
            }
        }
    }

    let html = render(app);
    let track = attributes_of(&body(&html), "div");
    let state = &track["data-state"];
    // The custom md radius keeps the theme's step word (`xl`).
    for word in ["base-md:radius-xs", "md-:radius-xl"] {
        assert!(state.contains(word), "{word} in {state}");
        has_copy(&html, word);
    }
    let radius = SizeCss::RADIUS.override_var();
    assert!(
        !track
            .get("style")
            .is_some_and(|style| style.contains(radius.name()))
    );
    assert!(html.contains(&format!("{}:0", radius.name())));
}

#[test]
fn value_props_declare_each_breakpoint_instead_of_an_inline_var() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Skeleton { height: responsive(ThemeAwareValue::from("40px")).md("64px".into()) }
                Dialog { aria_label: "Inline", size: responsive(ThemeAwareValue::from(Size::Sm)).md("40rem".into()), "content" }
                Burger { size: responsive(ThemeAwareValue::from(Size::Sm)).md(Size::Lg.into()) }
                Pagination {
                    total: 10,
                    page: 4,
                    aria_label: "Results",
                    size: responsive(Size::Sm).md(Size::Lg),
                    radius: responsive(ThemeAwareValue::from(Size::Xs)).md(Size::Xl.into()),
                    onchange: |_| {},
                }
            }
        }
    }

    let html = render(app);
    let page = body(&html);
    let dialog_size = DIALOG_SIZE.override_var();
    for var in [
        SKELETON_HEIGHT.name(),
        dialog_size.name(),
        "--lsx-pagination-radius",
    ] {
        let inline = tags_with(&page, &format!("{var}:"));
        assert!(inline.is_empty(), "{var} inline: {inline:?}");
    }
    assert!(html.contains(MD));
    assert!(html.contains(&format!("{}:64px", SKELETON_HEIGHT.name())));
    assert!(html.contains(&format!("{}:40rem", dialog_size.name())));
    assert!(html.contains(&format!("{}:", BURGER_SIZE.override_var().name())));
    // The arrows read one var, declared per breakpoint; the page buttons take copies.
    assert!(html.contains("--lsx-pagination-arrow-size:"));
    has_copy(&html, "md-:size-lg");
}
