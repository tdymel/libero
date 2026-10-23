use crate::{
    common::{attributes_of, body, nth_attributes, render},
    recorder::mounted_listeners,
};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{ActionIcon, Anchor, Button, NavLink},
    theme::{AnchorDefaults, Theme},
};

#[derive(Clone, PartialEq, Routable)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/other")]
    Other {},
}

#[component]
fn Home() -> Element {
    rsx! {
        Anchor { to: Route::Home {}, "Home" }
        Anchor { to: Route::Other {}, "Other" }
        Anchor { to: "https://example.com", "Example" }
        NavLink { to: Route::Other {}, "Nav" }
    }
}

#[component]
fn Other() -> Element {
    rsx! {}
}

fn routed() -> Element {
    rsx! {
        LiberoProvider { Router::<Route> {} }
    }
}

/// Todo 2986: a routed link renders as the router's `Link` would.
#[test]
fn a_routed_anchor_keeps_the_links_markup() {
    let html = render(routed);
    let home = nth_attributes(&html, "a", 0);
    let other = nth_attributes(&html, "a", 1);
    let external = nth_attributes(&html, "a", 2);

    assert_eq!(home["href"], "/", "{home:?}");
    assert_eq!(home["aria-current"], "page", "{home:?}");
    assert_eq!(other["href"], "/other", "{other:?}");
    assert!(!other.contains_key("aria-current"), "{other:?}");
    assert_eq!(external["rel"], "noopener noreferrer", "{external:?}");
}

/// Todo 2986: the router's `Link` added an `onmounted` per anchor, a blocking
/// WebView round trip each.
#[test]
fn routed_links_register_no_onmounted() {
    #[component]
    fn Empty() -> Element {
        rsx! {}
    }
    #[derive(Clone, PartialEq, Routable)]
    enum Bare {
        #[route("/")]
        Empty {},
    }
    fn empty() -> Element {
        rsx! {
            LiberoProvider { Router::<Bare> {} }
        }
    }

    assert_eq!(mounted_listeners(routed), mounted_listeners(empty));
}

/// Todo 3996: `not-hover` underlines at rest and drops the line under the pointer.
#[test]
fn a_not_hover_anchor_is_underlined_at_rest_and_bare_under_the_pointer() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Anchor { to: "https://example.com", underline: "not-hover", "Example" }
            }
        }
    }

    let html = render(app);
    let state = attributes_of(&html, "a")["data-state"].clone();
    let state = state
        .split(' ')
        .find(|token| token.contains("not-hover"))
        .unwrap_or_else(|| panic!("no not-hover state in {state}"));
    let selector = format!(r#"[data-state~="{state}"]"#);
    let rest = &html[html.find(&selector).expect("the not-hover rule")..];

    assert!(
        rest.starts_with(&format!("{selector}{{text-decoration:underline")),
        "{}",
        &rest[..120]
    );
    assert!(
        rest.contains(&format!("{selector}:hover{{text-decoration:none")),
        "{}",
        &rest[..240]
    );
}

#[test]
fn an_external_anchor_renders_a_plain_link() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Anchor { to: "https://example.com", "Example" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "a");

    assert_eq!(attributes["href"], "https://example.com");
    assert!(body(&html).contains("Example"));
}

/// Todos 579, 3324: a new tab is announced, drawn only on request, and the caller can opt out.
#[test]
fn a_blank_target_adds_the_new_tab_hint_unless_opted_out() {
    fn blank() -> Element {
        rsx! {
            LiberoProvider {
                Anchor { to: "https://example.com", target: "_blank", new_tab_icon: true, "Example" }
            }
        }
    }
    fn opted_out() -> Element {
        rsx! {
            LiberoProvider {
                Anchor { to: "https://example.com", target: "_blank", new_tab_hint: false, "Example" }
            }
        }
    }
    fn same_tab() -> Element {
        rsx! {
            LiberoProvider {
                Anchor { to: "https://example.com", "Example" }
            }
        }
    }

    let html = body(&render(blank));
    assert!(html.contains("(opens in a new tab)"), "{html}");
    assert!(html.contains("data-slot=\"new-tab\""), "{html}");
    assert!(html.contains("<svg"), "{html}");
    for app in [opted_out as fn() -> Element, same_tab] {
        let html = body(&render(app));
        assert!(!html.contains("new tab"), "{html}");
        assert!(!html.contains("<svg"), "{html}");
    }
}

/// Todo 3324: the icon is off by default and on by the theme, the prop wins; the hint text stays.
#[test]
fn the_new_tab_icon_is_off_unless_the_theme_or_the_prop_turns_it_on() {
    static ICONS: Theme = Theme {
        anchor: AnchorDefaults {
            new_tab_icon: true,
            ..Theme::DEFAULT.anchor
        },
        ..Theme::DEFAULT
    };
    fn plain() -> Element {
        rsx! {
            LiberoProvider {
                Anchor { to: "https://example.com", target: "_blank", "Example" }
            }
        }
    }
    fn themed() -> Element {
        rsx! {
            LiberoProvider { themes: &ICONS,
                Anchor { to: "https://example.com", target: "_blank", "Example" }
            }
        }
    }
    fn themed_but_off() -> Element {
        rsx! {
            LiberoProvider { themes: &ICONS,
                Anchor { to: "https://example.com", target: "_blank", new_tab_icon: false, "Example" }
            }
        }
    }

    for (app, icon) in [
        (plain as fn() -> Element, false),
        (themed, true),
        (themed_but_off, false),
    ] {
        let html = body(&render(app));
        assert!(html.contains("(opens in a new tab)"), "{html}");
        assert_eq!(html.contains("<svg"), icon, "{html}");
        assert_eq!(html.contains("data-slot=\"new-tab\""), icon, "{html}");
    }
}

/// Todo 3009: the target keyword matches case-insensitively, as in HTML.
#[test]
fn an_uppercase_blank_target_adds_the_new_tab_hint() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Anchor { to: "https://example.com", target: "_BLANK", "Example" }
            }
        }
    }

    let html = body(&render(app));
    assert!(html.contains("(opens in a new tab)"), "{html}");
}

#[derive(Clone, PartialEq, Routable)]
enum BlankRoute {
    #[route("/")]
    Blank {},
}

#[component]
fn Blank() -> Element {
    rsx! {
        LiberoProvider {
            Anchor { to: "/elsewhere", target: "_BLANK", "Elsewhere" }
        }
    }
}

/// Todo 3008: with a router mounted, an internal `_blank` link still opens a new tab.
#[test]
fn an_internal_blank_link_under_a_router_opens_a_new_tab() {
    fn app() -> Element {
        rsx! { Router::<BlankRoute> {} }
    }

    let html = body(&render(app));
    let attributes = attributes_of(&html, "a");

    assert_eq!(attributes["href"], "/elsewhere", "{html}");
    assert_eq!(attributes["target"], "_blank", "{html}");
    assert!(html.contains("(opens in a new tab)"), "{html}");
}

/// Todo 2996: a link-mode `Button` and `ActionIcon` announce a new tab as `Anchor` does.
#[test]
fn a_blank_button_and_action_icon_say_they_open_a_new_tab() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button { to: "https://example.com", target: "_blank", "Docs" }
                ActionIcon { aria_label: "Docs", to: "https://example.com", target: "_blank", "?" }
                Button { to: "https://example.com", "Same" }
                ActionIcon { aria_label: "Same", to: "https://example.com", "?" }
                Button { to: "https://example.com", target: "_blank", new_tab_icon: true, "Icon" }
            }
        }
    }

    let html = render(app);
    let action_icon = nth_attributes(&html, "a", 1);
    assert_eq!(action_icon["aria-label"], "Docs (opens in a new tab)");
    assert_eq!(nth_attributes(&html, "a", 3)["aria-label"], "Same");
    assert_eq!(html.matches("opens in a new tab").count(), 3, "{html}");
    // Todo 3324: only the button that asks for it draws the icon.
    assert_eq!(html.matches("data-slot=\"new-tab\"").count(), 1, "{html}");
}

/// Todo 3745: a link-mode `Button` can opt out of the new-tab hint, as `Anchor` can.
#[test]
fn a_blank_button_can_drop_the_new_tab_hint() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button {
                    to: "https://example.com",
                    target: "_blank",
                    new_tab_hint: false,
                    new_tab_icon: true,
                    "Docs"
                }
            }
        }
    }

    let html = render(app);
    assert!(!html.contains("opens in a new tab"), "{html}");
    assert!(!html.contains("data-slot=\"new-tab\""), "{html}");
}

/// Todo 3620: every link built on the shared anchor path drops a script `href` and keeps the element.
#[test]
fn a_script_target_renders_an_anchor_without_an_href() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Anchor { to: "javascript:alert(1)", "Script" }
                Anchor { to: " Java\tScript:alert(1)", "Spaced" }
                Anchor { to: "data:text/html,<b>x</b>", "Data" }
                NavLink { to: "VBScript:msgbox(1)", "Nav" }
                Button { to: "javascript:alert(1)", "Button" }
                ActionIcon { aria_label: "Icon", to: "javascript:alert(1)", "?" }
                Anchor { to: "https://example.com", "Ordinary" }
            }
        }
    }

    let html = render(app);
    for index in 0..6 {
        let attributes = nth_attributes(&html, "a", index);
        assert!(!attributes.contains_key("href"), "{index}: {attributes:?}");
    }
    assert_eq!(
        nth_attributes(&html, "a", 6)["href"],
        "https://example.com",
        "{html}"
    );
    assert!(body(&html).contains("Script"), "{html}");
}

/// Todo 3054: without a router a relative path is `External`, but not a foreign origin.
#[test]
fn a_relative_link_without_a_router_gets_no_rel() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Anchor { to: "/docs", "Docs" }
                Anchor { to: "//example.com", "Scheme-relative" }
            }
        }
    }

    let html = render(app);
    assert!(!nth_attributes(&html, "a", 0).contains_key("rel"), "{html}");
    assert_eq!(nth_attributes(&html, "a", 1)["rel"], "noopener noreferrer");
}
