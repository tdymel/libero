use dioxus::prelude::*;
use libero::{
    components::{Anchor, Code, CodeBlock, ColorSchemeButton, Flex, Text, Title},
    hooks::use_theme_set,
    sx::sx,
    theme::{Size, ThemeSet},
};

use crate::Route;

// snippet: item #[derive(Clone, PartialEq, Routable)] enum Route { #[route("/")] Home {} }
// snippet: item #[component] fn Home() -> Element { rsx! {} }
const THEME: &str = r#"static THEME: Theme = Theme {
    primary: HexColor::new(0x0F766E),
    button: ButtonDefaults {
        radius: Size::Xl,
        ..ButtonDefaults::DEFAULT
    },
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

/// The palettes, driving the same switcher as the header's.
#[component]
pub fn Theming() -> Element {
    let themes = use_theme_set();
    let palettes = ThemeSet::CATALOGUE.len();

    rsx! {
        section { "aria-labelledby": "theming-title",
            Flex {
                direction: "column",
                gap: "lg",
                sx: sx().breakpoint(Size::Md, sx().flex_direction("row").align_items("flex-start")),
                Flex { direction: "column", gap: "md", sx: sx().min_width("0").breakpoint(Size::Md, sx().flex("1 1 0")),
                    Title { size: "xl", component: "h2", id: "theming-title", "Theming" }
                    Text {
                        "A theme is one Rust struct: colours, spacing, radii and every "
                        "component's defaults. Light and dark ship together, and switching "
                        "between them is one attribute on the page root."
                    }
                    Text {
                        "Libero comes with {palettes} palettes. Pick one from the menu beside "
                        "the scheme toggle, and the whole site follows."
                    }
                    Flex { direction: "row", gap: "md", align: "center", wrap: "wrap",
                        ColorSchemeButton { size: "lg", themes: ThemeSet::CATALOGUE }
                        Text { "Showing "
                            Code { source: themes.name() }
                        }
                    }
                    Anchor { to: Route::ThemingPage {}, "Read the theming guide" }
                }
                CodeBlock {
                    source: THEME,
                    language: "rust",
                    sx: sx().min_width("0").breakpoint(Size::Md, sx().flex("1 1 0")),
                }
            }
        }
    }
}
