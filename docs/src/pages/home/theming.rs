use dioxus::prelude::*;
use libero::{
    components::{Anchor, Box, Code, CodeBlock, ColorSchemeButton, Flex, Paper, Text},
    hooks::use_theme_set,
    sx::sx,
    theme::{Size, ThemeSet},
};

use super::SectionHead;
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

/// The palette's colours, swatched from the live theme.
const SWATCHES: [&str; 6] = [
    "primary",
    "secondary",
    "success",
    "info",
    "warning",
    "error",
];

/// The palettes, driving the same switcher as the header's.
#[component]
pub fn Theming() -> Element {
    let themes = use_theme_set();
    let palettes = ThemeSet::CATALOGUE.len();

    rsx! {
        section { "aria-labelledby": "theming-title",
            Flex {
                direction: "column",
                gap: "xl",
                sx: sx().breakpoint(Size::Md, sx().flex_direction("row").align_items("flex-start")),
                Flex { direction: "column", gap: "lg", sx: sx().min_width("0").breakpoint(Size::Md, sx().flex("1 1 0")),
                    SectionHead { id: "theming-title", eyebrow: "Theming", title: "One struct, every colour",
                        "A theme is one Rust struct: colours, spacing, radii and every component's "
                        "defaults. Light and dark ship together, and switching is one attribute on "
                        "the page root."
                    }
                    Paper { bordered: true, shadow: "xs", radius: "lg", sx: sx().padding("lg"),
                        Flex { direction: "column", gap: "md",
                            Text {
                                "Libero comes with {palettes} palettes. Pick one here or in the "
                                "header, and the whole site follows."
                            }
                            Flex { direction: "row", gap: "md", align: "center", wrap: "wrap",
                                ColorSchemeButton { size: "lg", themes: ThemeSet::CATALOGUE }
                                Text { "Showing "
                                    Code { source: themes.name() }
                                }
                            }
                            // The names are beside each swatch, so the colours need no text of their own.
                            Flex { direction: "row", gap: "md", wrap: "wrap",
                                for color in SWATCHES {
                                    Flex { key: "{color}", direction: "row", gap: "xs", align: "center",
                                        Box {
                                            sx: sx()
                                                .width("20px")
                                                .height("20px")
                                                .border_radius("50%")
                                                .background(color),
                                        }
                                        Text { size: "sm", component: "span", "{color}" }
                                    }
                                }
                            }
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
