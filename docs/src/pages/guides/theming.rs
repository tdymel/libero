use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Anchor, Code, CodeBlock, Text};

// snippet: item #[derive(Clone, PartialEq, Routable)] enum Route { #[route("/")] Home {} }
// snippet: item #[component] fn Home() -> Element { rsx! {} }
const CUSTOM_THEME: &str = r#"static THEME: Theme = Theme {
    primary: HexColor::new(0x7C3AED),
    spacing: Sizes::new(4, 8, 12, 16, 20, 24),
    ..Theme::DEFAULT
};

fn App() -> Element {
    rsx! {
        LiberoProvider {
            themes: &THEME,
            Router::<Route> {}
        }
    }
}"#;

const USING_COLORS: &str = r#"Text { sx: sx().color("error"), "Payment failed" }
Box { sx: sx().background("primary.1").padding("md").border_radius("md"), "Saved" }"#;

const COMPONENT_DEFAULTS: &str = r#"static THEME: Theme = Theme {
    button: ButtonDefaults {
        radius: Size::Xl,
        ..ButtonDefaults::DEFAULT
    },
    ..Theme::DEFAULT
};"#;

// snippet: ignore - the themes and `Route` are the app's own
const THEME_SET: &str = r#"LiberoProvider {
    themes: ThemeSet::new().light(&LIGHT).dark(&DARK),
    Router::<Route> {}
}"#;

// snippet: ignore - the icons are the docs site's own
const SCHEME_TOGGLE: &str = r#"let scheme = use_color_scheme();

rsx! {
    ActionIcon {
        aria_label: match scheme.resolved() {
            ColorScheme::Dark => "Switch to the light theme",
            ColorScheme::Light => "Switch to the dark theme",
        },
        onclick: move |_| scheme.toggle(),
        if scheme.resolved() == ColorScheme::Dark { SunIcon {} } else { MoonIcon {} }
    }
}"#;

const READING_THE_THEME: &str = r#"let theme = use_theme();
let gap = theme.spacing.get(Size::Md);"#;

#[component]
pub fn ThemingPage() -> Element {
    rsx! {
        DocPage {
            title: "Theming",
            markdown: "/md/theming.md",
            lead: rsx! {
                Text {
                    "A theme is one struct: a color per role, the spacing and radius scales, "
                    "and a defaults struct per component. Hand yours to "
                    Code { source: "LiberoProvider" }
                    " and every component follows it."
                }
            },

            DocSection {
                title: "A custom theme",
                Text {
                    "Name the fields you change. The rest come from "
                    Code { source: "Theme::DEFAULT" }
                    "."
                }
                CodeBlock { source: CUSTOM_THEME, language: "rust" }
            }

            DocSection {
                title: "Colors",
                Text {
                    "Each role ("
                    Code { source: "primary" }
                    ", "
                    Code { source: "secondary" }
                    ", "
                    Code { source: "success" }
                    ", "
                    Code { source: "error" }
                    ", "
                    Code { source: "warning" }
                    ", "
                    Code { source: "info" }
                    ") is one hex value. Libero derives nine shades from it and keeps text "
                    "legible on each, so a brand color is one line. In "
                    Code { source: "sx" }
                    ", a role name is its base color and "
                    Code { source: "primary.1" }
                    " to "
                    Code { source: "primary.9" }
                    " are the shades."
                }
                CodeBlock { source: USING_COLORS, language: "rust" }
            }

            DocSection {
                title: "Scales",
                Text {
                    Code { source: "spacing" }
                    " and "
                    Code { source: "radius" }
                    " hold a pixel value per step, "
                    Code { source: "xs" }
                    " to "
                    Code { source: "xxl" }
                    ". Every "
                    Code { source: "size" }
                    ", "
                    Code { source: "gap" }
                    " and "
                    Code { source: "radius" }
                    " prop, and the size words in "
                    Code { source: "sx" }
                    ", step along them. "
                    Code { source: "elevation" }
                    " does the same for shadows."
                }
            }

            DocSection {
                title: "Component defaults",
                Text {
                    "Every prop a caller leaves unset comes from the component's struct on "
                    "the theme. Pill-shaped buttons everywhere is one change here, not a "
                    "prop at every call site."
                }
                CodeBlock { source: COMPONENT_DEFAULTS, language: "rust" }
            }

            DocSection {
                title: "Light and dark",
                Text {
                    "A "
                    Code { source: "ThemeSet" }
                    " pairs a light theme with a dark one. The default pairs "
                    Code { source: "Theme::DEFAULT" }
                    " with "
                    Code { source: "Theme::DARK" }
                    " and follows the system setting. A dark theme is one whose "
                    Code { source: "surface" }
                    " is dark and "
                    Code { source: "ink" }
                    " light. The roles adapt on their own. "
                    Code { source: "ThemeSet::CATALOGUE" }
                    " has ready-made pairs such as Catppuccin, Dracula and Nord."
                }
                CodeBlock { source: THEME_SET, language: "rust" }
                Text {
                    Code { source: "ColorSchemeButton" }
                    " is a ready-made switch. For your own, use "
                    Code { source: "use_color_scheme()" }
                    "."
                }
                CodeBlock { source: SCHEME_TOGGLE, language: "rust" }
            }

            DocSection {
                title: "Reading the theme",
                Text {
                    Code { source: "use_theme()" }
                    " returns the active theme, for a value you need in Rust rather than in "
                    Code { source: "sx" }
                    "."
                }
                CodeBlock { source: READING_THE_THEME, language: "rust" }
            }

            DocSection {
                title: "Localization",
                Text {
                    "Words and date formats are not part of the theme. See "
                    Anchor { to: crate::Route::LocalizationPage {}, "Localization" }
                    "."
                }
            }
        }
    }
}
