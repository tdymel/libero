use crate::common::render;

use dioxus::prelude::*;
use libero::{
    ColorSchemeHandle, LiberoProvider,
    components::Text,
    hooks::use_color_scheme,
    theme::{ColorScheme, ColorSchemeSetting, Theme, ThemeSet},
};

/// Reads the scheme the way an app's own toggle would, so the assertions
/// measure what a caller sees rather than the context behind it.
#[component]
fn Reader() -> Element {
    let scheme = use_color_scheme();
    let theme = libero::hooks::use_theme();

    rsx! {
        Text {
            "setting={scheme.setting():?} resolved={scheme.resolved():?} surface={theme.surface}"
        }
    }
}

/// Runs `act` once, after the first render, with the handle a component in
/// the tree would have. The hook is called here rather than in the caller's
/// closure: a hook belongs to a component's render, not to an effect.
#[component]
fn Act(act: EventHandler<ColorSchemeHandle>) -> Element {
    let scheme = use_color_scheme();
    use_effect(move || act.call(scheme.clone()));

    rsx! {}
}

fn scheme_line(html: &str) -> String {
    let start = html.find("setting=").expect("the reader rendered");
    let end = html[start..].find('<').expect("a closing tag") + start;
    html[start..end].to_string()
}

/// Nothing stored, and a platform that cannot be asked: the app follows the
/// system, and the system reads light. The sheet carries the pair, so the
/// *page* still follows `prefers-color-scheme` on its own - this is the
/// Rust-side answer, which is only ever consulted for values read off the
/// theme.
#[test]
fn an_untouched_app_follows_the_system() {
    fn app() -> Element {
        rsx! { LiberoProvider { Reader {} } }
    }

    let html = render(app);

    assert_eq!(
        scheme_line(&html),
        format!(
            "setting={:?} resolved={:?} surface={}",
            ColorSchemeSetting::System,
            ColorScheme::Light,
            Theme::DEFAULT.surface
        )
    );
}

/// A toggle pins the other scheme, and `use_theme()` follows it: a component
/// reading a `HexColor` off the theme has to agree with the page it is
/// painted on.
#[test]
fn a_toggle_pins_the_other_scheme_and_the_theme_follows() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Reader {}
                Act {
                    act: move |scheme: ColorSchemeHandle| scheme.toggle(),
                }
            }
        }
    }

    let html = render(app);

    assert_eq!(
        scheme_line(&html),
        format!(
            "setting={:?} resolved={:?} surface={}",
            ColorSchemeSetting::Dark,
            ColorScheme::Dark,
            Theme::DARK.surface
        )
    );
}

/// Toggling back to the scheme the platform is in drops the pin rather than
/// pinning that scheme too. Otherwise one round trip through the toggle
/// leaves the app deaf to the platform - and to a devtools emulation of it -
/// until storage is cleared.
#[test]
fn toggling_back_to_the_system_scheme_follows_the_system_again() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Reader {}
                Act {
                    act: move |scheme: ColorSchemeHandle| {
                        scheme.toggle();
                        scheme.toggle();
                    },
                }
            }
        }
    }

    let html = render(app);

    assert_eq!(
        scheme_line(&html),
        format!(
            "setting={:?} resolved={:?} surface={}",
            ColorSchemeSetting::System,
            ColorScheme::Light,
            Theme::DEFAULT.surface
        )
    );
}

/// Off the web there is no document root to carry the attribute, so the
/// switch falls back to rebuilding the sheet - with the pinned theme alone,
/// because the media block would otherwise keep answering for a choice the
/// app has already made.
#[test]
fn a_platform_with_no_reachable_root_rebuilds_the_sheet() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Text { "hello" }
                Act {
                    act: move |scheme: ColorSchemeHandle| scheme.set(ColorScheme::Dark),
                }
            }
        }
    }

    let html = render(app);

    assert!(html.contains(":root{--lsx-spacing-xs"));
    assert!(html.contains(&format!("--lsx-surface:{};", Theme::DARK.surface)));
    assert!(!html.contains("@media (prefers-color-scheme: dark)"));
    assert!(!html.contains("[data-lsx-theme"));
}

/// Handing the choice back to the platform restores the pair's sheet, media
/// block and all: that block is what makes the system case work with no
/// JavaScript, and a rebuilt one-theme sheet has thrown it away.
#[test]
fn following_the_system_again_restores_the_pair() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Text { "hello" }
                Act {
                    act: move |scheme: ColorSchemeHandle| {
                        scheme.set(ColorScheme::Dark);
                        scheme.set(ColorSchemeSetting::System);
                    },
                }
            }
        }
    }

    let html = render(app);

    assert!(html.contains("@media (prefers-color-scheme: dark){:root:not([data-lsx-theme]){"));
    assert!(html.contains("color-scheme:light dark;"));
}

/// A set with one theme has no dark half to pin to, so a toggle warns and
/// changes nothing - including the setting, which would otherwise claim a
/// scheme the page is not painted in.
#[test]
fn a_set_with_no_dark_half_cannot_be_toggled_into_one() {
    static SOLO: Theme = Theme {
        primary: libero::theme::HexColor::new(0x22_E6_8B),
        ..Theme::DEFAULT
    };

    fn app() -> Element {
        rsx! {
            LiberoProvider {
                themes: &SOLO,
                Reader {}
                Act {
                    act: move |scheme: ColorSchemeHandle| scheme.toggle(),
                }
            }
        }
    }

    let html = render(app);

    assert_eq!(
        scheme_line(&html),
        format!(
            "setting={:?} resolved={:?} surface={}",
            ColorSchemeSetting::System,
            ColorScheme::Light,
            Theme::DEFAULT.surface
        )
    );
}

/// Swapping the whole set rebuilds the sheet with the new palette, and keeps
/// the pinned scheme: a reader who chose dark stays in dark through the swap.
#[test]
fn a_set_swap_rebuilds_the_sheet_and_keeps_the_scheme() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Reader {}
                SwapSet {}
            }
        }
    }

    // Both writes in one effect: `render` renders once after the effects
    // run, so a second effect's write would need a second pass to be seen.
    #[component]
    fn SwapSet() -> Element {
        let scheme = use_color_scheme();
        let themes = libero::hooks::use_theme_set();
        use_effect(move || {
            scheme.set(ColorScheme::Dark);
            themes.set(ThemeSet::GRUVBOX);
        });

        rsx! {}
    }

    let html = render(app);

    assert!(scheme_line(&html).contains(&format!("setting={:?}", ColorSchemeSetting::Dark)));
    assert!(
        scheme_line(&html).contains(&format!("surface={}", libero::theme::GRUVBOX_DARK.surface))
    );
    assert!(html.contains(&format!(
        "--lsx-surface:{};",
        libero::theme::GRUVBOX_DARK.surface
    )));
    // The old palette is gone, not merely overridden further down.
    assert!(!html.contains(&format!("--lsx-surface:{};", Theme::DARK.surface)));
}

/// Every set in the catalogue is a real pair with a name a picker can show.
#[test]
fn the_catalogue_is_all_named_pairs() {
    assert!(ThemeSet::CATALOGUE.len() >= 2);

    for set in ThemeSet::CATALOGUE {
        assert!(!set.name().is_empty());
        let dark = set
            .dark_theme()
            .unwrap_or_else(|| panic!("{} has no dark half", set.name()));
        assert_ne!(dark.surface, set.light_theme().surface, "{}", set.name());
        // By value: `&ThemeSet::DEFAULT` is a promoted temporary at every
        // use site, so `ptr::eq` on two of them proves nothing.
        assert_eq!(ThemeSet::from_catalogue(set.name()), Some(*set));
    }
}
