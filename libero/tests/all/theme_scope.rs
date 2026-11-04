//! `ThemeScope` shows its subtree in another theme of the set (todo 5638).

use crate::common::{attributes_of, body, css_rules, render, rule_of};

use dioxus::prelude::*;
use libero::{
    LiberoProvider, ThemeScope,
    components::Box,
    hooks::{DrawerOptions, ModalScope, use_drawer, use_theme},
    sx::sx,
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

/// Todo 5710: a portalled overlay mounts under the outlet, yet reads its scope's theme, and an
/// unknown inner scope carries the scope around it.
#[test]
fn an_overlay_opened_inside_reads_the_scopes_theme() {
    #[component]
    fn Probe() -> Element {
        let surface = use_theme().surface.to_string();
        rsx! { span { id: "probe", "{surface}" } }
    }
    #[component]
    fn Opener() -> Element {
        let options = DrawerOptions {
            aria_label: Some("Settings".into()),
            ..Default::default()
        };
        let drawer = use_drawer(options, |_: ModalScope<()>| rsx! { Probe {} });
        use_hook(move || drawer.open());
        rsx! {}
    }
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ThemeScope { theme: "dark",
                    ThemeScope { theme: "sepia", Opener {} }
                }
            }
        }
    }
    let html = body(&render(app));
    let probe = html.find(r#"id="probe""#).expect(&html);
    let rest = &html[probe..];
    assert_eq!(
        &rest[rest.find('>').unwrap() + 1..rest.find("</span>").unwrap()],
        Theme::DARK.surface.to_string()
    );
    let carried = html[..probe].rfind("data-lsx-theme-scope").expect(&html);
    let tag = &html[html[..carried].rfind('<').unwrap()..probe];
    assert!(tag.contains(r#"data-lsx-theme-scope="dark""#), "{tag}");
    assert!(tag.contains(r#"data-lsx-scheme="dark""#), "{tag}");
}

/// Todo 5681: a scope carries its theme's scheme, which `Sx::dark` and `Sx::light` read.
#[test]
fn a_scope_writes_its_theme_scheme() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ThemeScope { theme: "dark", "inside" }
                ThemeScope { theme: "light", "inside" }
                ThemeScope { theme: "sepia", "inside" }
            }
        }
    }
    let html = body(&render(app));
    assert_eq!(
        html.matches(r#"data-lsx-scheme="dark""#).count(),
        1,
        "{html}"
    );
    assert_eq!(
        html.matches(r#"data-lsx-scheme="light""#).count(),
        1,
        "{html}"
    );
    assert_eq!(
        html.matches("data-lsx-scheme").count(),
        2,
        "an unknown scope writes none: {html}"
    );
}

#[test]
fn use_theme_follows_the_nearest_scope() {
    #[component]
    fn Probe(id: &'static str) -> Element {
        let surface = use_theme().surface.to_string();
        rsx! { span { id, "{surface}" } }
    }
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Probe { id: "page" }
                ThemeScope { theme: "dark",
                    Probe { id: "dark" }
                    ThemeScope { theme: "sepia", Probe { id: "unknown" } }
                }
            }
        }
    }
    let html = body(&render(app));
    let text = |id: &str| {
        let from = html.find(&format!(r#"id="{id}""#)).expect(id);
        let rest = &html[from..];
        rest[rest.find('>').unwrap() + 1..rest.find("</span>").unwrap()].to_string()
    };
    assert_eq!(text("page"), Theme::DEFAULT.surface.to_string());
    assert_eq!(text("dark"), Theme::DARK.surface.to_string());
    assert_eq!(
        text("unknown"),
        Theme::DARK.surface.to_string(),
        "an unknown scope keeps the scope around it"
    );
}

/// Todo 5681: the root arms skip a pinned scope and the scope's own arms follow them, so the
/// page's scheme never leaks in.
#[test]
fn scheme_arms_skip_a_pinned_scope_and_follow_it() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Box { sx: sx().background_color("#ff0000").dark(sx().background_color("#008000")), "x" }
            }
        }
    }
    let html = render(app);
    let scope = "[data-lsx-theme-scope][data-lsx-scheme]";
    let green = html.find("#008000").expect(&html);
    let from = html[..green]
        .rfind(r#":where(:root[data-lsx-scheme="dark"]) "#)
        .expect(&html);
    let sheet = &html[from..];
    for arm in [
        format!(":not(:where({scope}, {scope} *))"),
        r#":where([data-lsx-theme-scope][data-lsx-scheme="dark"]) ."#.to_string(),
        r#":where([data-lsx-theme-scope][data-lsx-scheme="dark"]){"#.to_string(),
    ] {
        assert!(
            sheet.contains(&arm),
            "{arm} in {}",
            &sheet[..600.min(sheet.len())]
        );
    }
    assert!(
        sheet.find(":not(:where(").unwrap()
            < sheet
                .find(r#":where([data-lsx-theme-scope][data-lsx-scheme="dark"]) ."#)
                .unwrap(),
        "the scope arms come after the root arms"
    );
}

/// Todo 5681: one level only, a scope inside a scope pinned the other way matches both
/// schemes' arms, and the later arm of a rule wins.
#[test]
fn nested_scopes_of_opposite_schemes_are_not_resolved() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ThemeScope { theme: "dark", ThemeScope { theme: "light", "inside" } }
            }
        }
    }
    let html = body(&render(app));
    assert_eq!(
        html.matches(r#"data-lsx-scheme="dark""#).count(),
        1,
        "{html}"
    );
    assert_eq!(
        html.matches(r#"data-lsx-scheme="light""#).count(),
        1,
        "{html}"
    );
}
