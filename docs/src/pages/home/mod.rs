//! The docs site's landing page at `/`: one module per section, top to bottom.

mod booking;
mod closing;
mod hero;
mod platforms;
mod principles;
mod status;
mod theming;

use dioxus::prelude::*;
use libero::components::Flex;

#[component]
pub fn Home() -> Element {
    rsx! {
        document::Title { "Libero - accessible, themeable components for Dioxus" }
        Flex { direction: "column", gap: "xxl",
            hero::Hero {}
            principles::Principles {}
            theming::Theming {}
            platforms::Platforms {}
            status::Status {}
            closing::Closing {}
        }
    }
}
