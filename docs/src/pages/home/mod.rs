//! The docs site's landing page at `/`: one module per section, top to bottom.

mod batteries;
mod booking;
mod closing;
mod example;
mod hero;
mod stats;

use dioxus::prelude::*;
use libero::{
    components::{Flex, OptionLabel, Title},
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
            example::Example {}
            batteries::Batteries {}
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

/// A tab's label with `icon` before its name; the name is still what names the tab.
pub(super) fn icon_label(name: String, icon: Element) -> OptionLabel {
    OptionLabel::rich(
        name.clone(),
        rsx! {
            span { display: "inline-flex", align_items: "center", gap: "6px",
                span { display: "inline-flex", width: "16px", height: "16px", {icon} }
                "{name}"
            }
        },
    )
}

/// A section's heading: an `h2` set small, blue and capitalised.
#[component]
pub(super) fn SectionTitle(id: &'static str, children: Element) -> Element {
    rsx! {
        Title {
            size: "sm",
            component: "h2",
            id,
            sx: sx()
                .color("primary.7")
                .font_weight("700")
                .text_transform("uppercase")
                .letter_spacing("0.08em"),
            {children}
        }
    }
}
