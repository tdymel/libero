use dioxus::prelude::*;
use libero::{
    components::{Anchor, Code, CodeBlock, Flex, Text, Title},
    sx::sx,
    theme::Size,
};

use crate::Route;

// snippet: ignore - needs the `dioxus-native` crate and the app's own `App`
const LAUNCH: &str = r#"fn main() {
    #[cfg(feature = "native")]
    dioxus_native::launch(App);
    #[cfg(not(feature = "native"))]
    dioxus::launch(App);
}"#;

/// One `App`, launched on the web or natively.
#[component]
pub fn Platforms() -> Element {
    rsx! {
        section { "aria-labelledby": "platforms-title",
            Flex {
                direction: "column",
                gap: "lg",
                sx: sx().breakpoint(Size::Md, sx().flex_direction("row").align_items("flex-start")),
                Flex { direction: "column", gap: "md", sx: sx().min_width("0").breakpoint(Size::Md, sx().flex("1 1 0")),
                    Title { size: "xl", component: "h2", id: "platforms-title", "Web and native" }
                    Text {
                        "The same components run in the browser and in a native window through "
                        "Blitz, the renderer behind "
                        Code { source: "dioxus-native" }
                        ". This site is one of them: its "
                        Code { source: "main" }
                        " picks the launcher, and nothing else changes."
                    }
                    Text {
                        "Native is still catching up. Where Blitz lacks something, the platform "
                        "API returns "
                        Code { source: "None" }
                        " instead of breaking, and the "
                        Anchor { to: Route::PlatformPage {}, "Platform guide" }
                        " lists what is missing."
                    }
                }
                CodeBlock {
                    source: LAUNCH,
                    language: "rust",
                    sx: sx().min_width("0").breakpoint(Size::Md, sx().flex("1 1 0")),
                }
            }
        }
    }
}
