use dioxus::prelude::*;
use libero::{
    components::{
        Anchor, Box, Button, Code, CodeBlock, Flex, Kbd, List, ListItem, Paper, Table, Text, Title,
        column,
    },
    sx::sx,
    theme::Size,
};

use super::{SectionHead, tint};
use crate::{Route, exports::COMPONENTS};

// snippet: ignore - needs the `dioxus-native` crate and the app's own `App`
const LAUNCH: &str = r#"fn main() {
    #[cfg(feature = "native")]
    dioxus_native::launch(App);
    #[cfg(not(feature = "native"))]
    dioxus::launch(App);
}"#;

const SX_BUTTON: &str = r#"Button {
    sx: sx()
        .padding_inline("xl")
        .hover(sx().background("primary.8")),
    "Save"
}"#;

/// The heavy components, by name, each linking to its page.
const HEAVY: [(&str, Route); 8] = [
    ("Combobox", Route::ComboboxPage {}),
    ("ChronoPicker", Route::ChronoPickerPage {}),
    ("Form", Route::FormPage {}),
    ("Spotlight", Route::SpotlightPage {}),
    ("Notifications", Route::NotificationsPage {}),
    ("Table", Route::TablePage {}),
    ("CodeBlock", Route::CodeBlockPage {}),
    ("QrCode", Route::QrCodePage {}),
];

#[derive(Clone, PartialEq)]
struct Layer {
    name: &'static str,
    role: &'static str,
}

const LAYERS: [Layer; 4] = [
    Layer {
        name: "Rust",
        role: "Types, and the compiler checking every prop",
    },
    Layer {
        name: "Dioxus",
        role: "Components, signals and the router",
    },
    Layer {
        name: "Blitz",
        role: "Native windows, no webview",
    },
    Layer {
        name: "libero",
        role: "Components, theme and the sx() builder",
    },
];

const GAP: &str = "24px";

/// One bento cell: `span` of the six columns from `Md` up.
#[component]
fn Cell(
    span: u8,
    title: &'static str,
    #[props(default)] accent: bool,
    children: Element,
) -> Element {
    rsx! {
        Paper {
            bordered: true,
            shadow: "xs",
            radius: "lg",
            sx: sx()
                .padding("lg")
                .min_width("0")
                .apply_if(accent.then_some(()), |base, ()| {
                    base.background(format!(
                        "linear-gradient(160deg, {} 0%, transparent 70%)",
                        tint(14)
                    ))
                })
                .flex("1 1 100%")
                // `span` sixths of the row less its share of the gaps, 1px short so
                // rounding never wraps a pair; `grow` takes the pixel back.
                .breakpoint(
                    Size::Md,
                    sx().flex(format!(
                        "1 1 calc({span} * (100% - 5 * {GAP}) / 6 + {} * {GAP} - 1px)",
                        span - 1
                    )),
                ),
            Flex { direction: "column", gap: "md",
                Title { size: "md", component: "h3", "{title}" }
                {children}
            }
        }
    }
}

/// What libero is built on and what it brings, as a bento grid.
#[component]
pub fn Features() -> Element {
    let components = COMPONENTS.len();

    rsx! {
        section { "aria-labelledby": "features-title",
            Flex { direction: "column", gap: "xl",
                SectionHead { id: "features-title", eyebrow: "Features", title: "Rust all the way down",
                    "Libero sits on Dioxus, so a component is a Rust function and a prop is a typed "
                    "field. No JavaScript toolchain, no CSS framework, one language from the "
                    "button to the build."
                }
                // A wrapping row, not a grid: Blitz did not wrap text in grid cells.
                Box {
                    sx: sx().display("flex").flex_wrap("wrap").align_items("stretch").gap(GAP),
                    Cell { span: 4, title: "Built on Dioxus and Rust", accent: true,
                        Text {
                            "The compiler catches a misspelt prop or a variant that does not "
                            "exist, where a convention would leave it to code review."
                        }
                        Table {
                            caption: "The stack, bottom to top",
                            data: LAYERS.to_vec(),
                            columns: vec![
                                column("Layer").value(|l: &Layer| l.name.to_string()),
                                column("What it brings").value(|l: &Layer| l.role.to_string()),
                            ],
                        }
                    }
                    Cell { span: 2, title: "Accessible by default",
                        Text {
                            "Press "
                            Kbd { "Tab" }
                            ": a skip link comes first, and a focus ring follows you. "
                            Kbd { "Ctrl K" }
                            " opens the search from anywhere."
                        }
                        List { size: "sm",
                            ListItem { "ARIA keyboard models for menus, tabs and trees" }
                            ListItem { "Never colour alone for a selected control" }
                            ListItem { "Keys listed on every component page" }
                        }
                        Anchor { to: Route::AccessibilityPage {}, "What holds, and what does not" }
                    }
                    Cell { span: 3, title: "One codebase, web and native",
                        Text {
                            "The same components run in the browser and in a native window "
                            "through Blitz. This site is one of them: its "
                            Code { source: "main" }
                            " picks the launcher, and nothing else changes."
                        }
                        CodeBlock { source: LAUNCH, language: "rust" }
                        Anchor { to: Route::PlatformPage {}, "What native still lacks" }
                    }
                    Cell { span: 3, title: "Styled with a typed builder",
                        Text {
                            Code { source: "sx()" }
                            " styles any component over current CSS. Your own unlayered CSS "
                            "still wins, because libero's rules sit in cascade layers."
                        }
                        CodeBlock { source: SX_BUTTON, language: "rust" }
                        Flex { direction: "row", gap: "md", align: "center", wrap: "wrap",
                            Button {
                                sx: sx()
                                    .padding_inline("xl")
                                    .hover(sx().background("primary.8")),
                                "Save"
                            }
                            Anchor { to: Route::StylingPage {}, "Read the styling guide" }
                        }
                    }
                    Cell { span: 4, title: "Batteries included",
                        Text { "Libero exports {components} components. Among the bigger ones:" }
                        Flex { direction: "row", gap: "sm", wrap: "wrap",
                            for (name, route) in HEAVY {
                                Button {
                                    key: "{name}",
                                    to: route,
                                    size: "sm",
                                    variant: "tonal",
                                    radius: "xl",
                                    "{name}"
                                }
                            }
                        }
                        Text {
                            "CodeBlock highlights 30 languages, and you compile only the grammars "
                            "you turn on."
                        }
                    }
                    Cell { span: 2, title: "Docs your assistant can read",
                        Text {
                            "Every page has a markdown copy, and the test suite compiles its "
                            "examples. Code you or your AI assistant copies out of it builds."
                        }
                    }
                }
            }
        }
    }
}
