//! `Toolbar`, for the `RovingTabindex` archetype: every kind of item, a vertical bar, RTL.

use dioxus::prelude::*;
use libero::components::{
    ActionIcon, Button, ButtonGroup, Flex, Options, Select, Toolbar, ToolbarGroup, ToolbarSeparator,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/toolbar", || rsx! { ToolbarPage {} }),
    ("/toolbar-vertical", || rsx! { ToolbarVerticalPage {} }),
    ("/toolbar-rtl", || rsx! { ToolbarRtlPage {} }),
];

#[derive(Clone, PartialEq, Options)]
enum Font {
    Serif,
    Sans,
    Mono,
}

/// Icons in a group, a separator, a `ButtonGroup`, a disabled button and a `Select`.
#[component]
fn ToolbarPage() -> Element {
    let mut font = use_signal(|| Some(Font::Serif));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "640px",
            Toolbar { "aria-label": "Formatting",
                ToolbarGroup { "aria-label": "Style",
                    ActionIcon { id: "bold", aria_label: "Bold", "B" }
                    ActionIcon { id: "italic", aria_label: "Italic", "I" }
                }
                ToolbarSeparator {}
                ButtonGroup { "aria-label": "Align", variant: "outlined",
                    Button { id: "left", "Left" }
                    Button { id: "right", "Right" }
                }
                Button { id: "undo", disabled: true, "Undo" }
                Select { value: font(), onchange: move |next| font.set(next), "aria-label": "Font" }
            }
            button { id: "after", "after" }
        }
    }
}

#[component]
fn ToolbarVerticalPage() -> Element {
    rsx! {
        Toolbar { "aria-label": "Tools", orientation: "vertical", loop_focus: false,
            ActionIcon { id: "pen", aria_label: "Pen", "P" }
            ToolbarSeparator {}
            ActionIcon { id: "eraser", aria_label: "Eraser", "E" }
            ActionIcon { id: "fill", aria_label: "Fill", "F" }
        }
    }
}

#[component]
fn ToolbarRtlPage() -> Element {
    rsx! {
        div { dir: "rtl",
            Toolbar { "aria-label": "Formatting",
                Button { id: "one", "One" }
                Button { id: "two", "Two" }
                Button { id: "three", "Three" }
            }
        }
    }
}
