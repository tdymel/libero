//! `ThemeScope` shows its subtree in another theme of the set (todo 5638).

use crate::common::{attributes_of, body, css_rules, render, rule_of};

use dioxus::prelude::*;
use libero::{
    LiberoProvider, ThemeScope,
    hooks::{DrawerOptions, ModalScope, use_drawer},
    theme::Theme,
};

const DARK_SCOPE: &str = r#"[data-lsx-theme-scope="dark"]"#;

#[test]
fn a_page_without_a_scope_ships_no_scope_rules() {
    fn app() -> Element {
        rsx! { LiberoProvider { "x" } }
    }
    let html = render(app);
    assert!(!html.contains("data-lsx-theme-scope"), "{html}");
}

#[test]
fn a_dark_scope_repaints_only_the_vars_the_pair_disagrees_on() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ThemeScope { theme: "dark", "inside" }
            }
        }
    }
    let html = render(app);
    let scope = attributes_of(&body(&html), "div");
    assert_eq!(scope["data-lsx-theme-scope"], "dark", "{html}");

    let rule = rule_of(&html, DARK_SCOPE);
    assert_eq!(
        rule["--lsx-surface"],
        Theme::DARK.surface.to_string(),
        "{rule:?}"
    );
    assert_eq!(rule["color-scheme"], "dark");
    assert_eq!(rule["background-color"], "var(--lsx-surface)");
    // Shared by both themes, so inherited from the root as it is.
    assert!(!rule.contains_key("--lsx-spacing-md"), "{rule:?}");
    let scope_rules = css_rules(&html)
        .into_iter()
        .filter(|rule| rule.selector == DARK_SCOPE)
        .count();
    assert_eq!(scope_rules, 1, "one block, however many scopes");
}

#[test]
fn an_unknown_theme_leaves_the_children_unscoped() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ThemeScope { theme: "sepia", "inside" }
            }
        }
    }
    let html = render(app);
    assert!(!html.contains("data-lsx-theme-scope"), "{html}");
    assert!(html.contains("inside"), "{html}");
}

#[test]
fn an_overlay_opened_inside_carries_the_scope_out_of_the_tree() {
    #[component]
    fn Opener() -> Element {
        let options = DrawerOptions {
            aria_label: Some("Settings".into()),
            ..Default::default()
        };
        let drawer = use_drawer(options, |_: ModalScope<()>| rsx! { "drawer content" });
        use_hook(move || drawer.open());
        rsx! {}
    }
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ThemeScope { theme: "dark", Opener {} }
            }
        }
    }
    let html = body(&render(app));
    let scopes = html.matches(r#"data-lsx-theme-scope="dark""#).count();
    assert_eq!(scopes, 2, "the box and the portalled drawer: {html}");
    let portalled = html.rfind(r#"data-lsx-theme-scope="dark""#).unwrap();
    assert!(
        html[portalled..].contains("drawer content"),
        "the drawer sits inside the carried scope: {html}"
    );
}
