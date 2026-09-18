use dioxus::prelude::*;
use libero::{
    components::{
        Badge, Button, CodeBlock, Flex, Options, Paper, ProgressBar, SegmentedControl, Slider,
        SliderChangeEvent, Switch, Text, Title,
    },
    hooks::use_localization,
    sx::sx,
    theme::{ColorCss, ColorShade, Size},
};

use super::tint;
use crate::{GITHUB, Route};

#[derive(Clone, Copy, PartialEq, Options)]
enum Quality {
    Draft,
    Good,
    Best,
}

/// The pitch, the way in, and a few live controls to touch on the first screen.
#[component]
pub fn Hero() -> Element {
    let localization = use_localization();

    rsx! {
        section {
            "aria-labelledby": "hero-title",
            Flex {
                direction: "column",
                gap: "xl",
                sx: sx()
                    .padding("lg")
                    .border_radius("xl")
                    .background(format!(
                        "linear-gradient(135deg, {} 0%, {} 55%, transparent 100%)",
                        tint(18),
                        tint(6),
                    ))
                    .breakpoint(
                        Size::Md,
                        sx().flex_direction("row").align_items("center").padding("48px"),
                    ),
                Flex { direction: "column", gap: "lg", sx: sx().min_width("0").breakpoint(Size::Md, sx().flex("1 1 0")),
                    Flex { direction: "row", gap: "sm", wrap: "wrap",
                        Badge { variant: "outlined", color: "muted", "Pre-release" }
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
                            .font_size("2.25rem")
                            .line_height("1.1")
                            .breakpoint(Size::Md, sx().font_size("3.25rem")),
                        span { color: ColorCss::PRIMARY.role_value("text-", ColorShade::S6), "Accessible, themeable" }
                        " components for Dioxus"
                    }
                    Text { size: "lg",
                        "Libero is a component library written in Rust. You style it with a typed "
                        "builder, theme it with a struct, and run the same code in the browser and "
                        "natively through Blitz."
                    }
                    Flex { direction: "row", gap: "md", wrap: "wrap",
                        Button { to: Route::GettingStarted {}, size: "lg", "Get started" }
                        Button { to: Route::BoxPage {}, size: "lg", variant: "outlined", "Browse components" }
                        Button {
                            to: GITHUB,
                            target: "_blank",
                            size: "lg",
                            variant: "standard",
                            aria_label: format!("GitHub {}", localization.anchor.new_tab),
                            "GitHub"
                        }
                    }
                    CodeBlock {
                        source: "cargo add libero",
                        language: "shell",
                        header: false,
                        line_numbers: false,
                        sx: sx().max_width("360px"),
                    }
                }
                Controls {}
            }
        }
    }
}

/// A handful of live components on one card, wired to each other.
#[component]
fn Controls() -> Element {
    let mut quality = use_signal(|| Quality::Good);
    let mut level = use_signal(|| 64.0);
    let mut spatial = use_signal(|| true);

    rsx! {
        Paper {
            shadow: "lg",
            radius: "lg",
            "aria-label": "Sample controls",
            "role": "group",
            sx: sx()
                .padding("lg")
                .width("100%")
                .max_width("420px")
                .align_self("center")
                .breakpoint(Size::Md, sx().flex("0 1 400px")),
            Flex { direction: "column", gap: "lg",
                Flex { direction: "row", gap: "sm", align: "center", justify: "space-between",
                    Text { sx: sx().font_weight("700"), "Export" }
                    Badge { color: if spatial() { "success" } else { "muted" }, size: "sm",
                        if spatial() { "Spatial on" } else { "Stereo" }
                    }
                }
                SegmentedControl {
                    "aria-label": "Quality",
                    full_width: true,
                    value: quality(),
                    onchange: move |next| quality.set(next),
                }
                Flex { direction: "column", gap: "xs",
                    Text { size: "sm", sx: sx().font_weight("600"), "Level" }
                    Slider {
                        aria_label: "Level",
                        value: level(),
                        oninput: move |event: SliderChangeEvent| level.set(event.value()),
                    }
                }
                Switch {
                    label: "Spatial audio",
                    checked: spatial(),
                    onchange: move |on: bool| spatial.set(on),
                }
                ProgressBar { aria_label: "Export progress", value: level(), size: "lg" }
                Text { size: "sm",
                    "Every control on this card is a libero component. Try it with the keyboard."
                }
            }
        }
    }
}
