//! The docs site's landing page at `/`: one module per section, top to bottom.

mod booking;
mod closing;
mod features;
mod hero;
mod philosophy;
mod showcase;
mod stats;
mod status;
mod theming;
mod try_it;

use dioxus::prelude::*;
use libero::{
    components::{Flex, Text, Title},
    sx::sx,
    theme::{ColorCss, ColorShade},
};

#[component]
pub fn Home() -> Element {
    rsx! {
        document::Title { "Libero - accessible, themeable components for Dioxus" }
        Flex { direction: "column", gap: "xxl", sx: sx().padding_bottom("lg"),
            hero::Hero {}
            stats::Stats {}
            features::Features {}
            showcase::Showcase {}
            try_it::TryIt {}
            theming::Theming {}
            philosophy::Philosophy {}
            status::Status {}
            closing::Closing {}
        }
    }
}

/// The palette's primary colour at `percent` over transparent: a tint that
/// follows the palette and the scheme.
pub(super) fn tint(percent: u8) -> String {
    format!(
        "color-mix(in srgb, {} {percent}%, transparent)",
        ColorCss::PRIMARY.value(ColorShade::S6)
    )
}

/// A section's eyebrow, `h2` and lead, the same on every section.
#[component]
pub(super) fn SectionHead(
    id: &'static str,
    eyebrow: &'static str,
    title: &'static str,
    children: Element,
) -> Element {
    rsx! {
        Flex { direction: "column", gap: "sm", sx: sx().max_width("720px"),
            Text {
                size: "sm",
                sx: sx()
                    .color("primary.7")
                    .font_weight("700")
                    .text_transform("uppercase")
                    .letter_spacing("0.08em"),
                "{eyebrow}"
            }
            Title { size: "xl", component: "h2", id, "{title}" }
            Text { size: "lg", {children} }
        }
    }
}
