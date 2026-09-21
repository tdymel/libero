use dioxus::prelude::*;
use libero::{
    components::{Badge, Button, CodeBlock, Flex, Text, Title},
    hooks::use_localization,
    sx::sx,
    theme::{ColorCss, ColorShade, Size},
};

use super::tint;
use crate::{GITHUB, Route};

const PLATFORMS: [&str; 4] = ["Web", "Desktop", "Android", "iOS"];

/// The pitch and the way in, centred on the first screen.
#[component]
pub fn Hero() -> Element {
    let localization = use_localization();

    rsx! {
        section {
            "aria-labelledby": "hero-title",
            Flex {
                direction: "column",
                align: "center",
                gap: "lg",
                sx: sx()
                    .padding("lg")
                    .border_radius("xl")
                    .text_align("center")
                    .background(format!(
                        "linear-gradient(180deg, {} 0%, {} 60%, transparent 100%)",
                        tint(18),
                        tint(6),
                    ))
                    .breakpoint(Size::Md, sx().padding("64px 48px")),
                Flex { direction: "row", gap: "sm", wrap: "wrap", justify: "center",
                    // The tint under it takes the grey and the text role 6 under 4.5:1 (899).
                    Badge { variant: "outlined", color: "primary.8", "Pre-release" }
                    Badge { variant: "tonal", "Rust" }
                    Badge { variant: "tonal", "Dioxus" }
                }
                Title {
                    size: "xxl",
                    component: "h1",
                    id: "hero-title",
                    // Focused after a navigation (`AppShell`), without a ring round the heading.
                    tabindex: "-1",
                    sx: sx()
                        .selector("&:focus", sx().outline("none"))
                        .max_width("900px")
                        .font_size("2.25rem")
                        .line_height("1.1")
                        .breakpoint(Size::Md, sx().font_size("3.5rem")),
                    span { color: ColorCss::PRIMARY.role_value("text-", ColorShade::S6), "Focus on your game," }
                    " while the "
                    span { color: ColorCss::PRIMARY.role_value("text-", ColorShade::S6), "Libero" }
                    " has your back!"
                }
                Text { size: "lg", sx: sx().max_width("720px"),
                    "Libero is a component library for Rust and Dioxus. Like the libero on a "
                    "volleyball court, it covers the defence: accessibility, keyboard support "
                    "and theming are handled, so you can focus on your app."
                }
                Flex { direction: "column", align: "center", gap: "sm",
                    Text {
                        component: "p",
                        sx: sx()
                            .font_size("1.75rem")
                            .font_weight("800")
                            .line_height("1.2")
                            .color("primary.7"),
                        "Build once, run everywhere"
                    }
                    Text { "One Dioxus codebase for the web, the desktop, Android and iOS." }
                    Flex { direction: "row", gap: "sm", wrap: "wrap", justify: "center",
                        for platform in PLATFORMS {
                            Badge { key: "{platform}", variant: "outlined", color: "primary.8", "{platform}" }
                        }
                    }
                }
                Flex { direction: "row", gap: "md", wrap: "wrap", justify: "center",
                    Button { to: Route::GettingStarted {}, size: "lg", "Get started" }
                    Button { to: Route::BoxPage {}, size: "lg", variant: "outlined", color: "primary.8", "Browse components" }
                    Button {
                        to: GITHUB,
                        target: "_blank",
                        size: "lg",
                        variant: "standard",
                        color: "primary.8",
                        aria_label: format!("GitHub {}", localization.anchor.new_tab),
                        "GitHub"
                    }
                }
                CodeBlock {
                    source: "cargo add libero",
                    language: "shell",
                    header: false,
                    line_numbers: false,
                    sx: sx().width("100%").max_width("360px").text_align("left"),
                }
            }
        }
    }
}
