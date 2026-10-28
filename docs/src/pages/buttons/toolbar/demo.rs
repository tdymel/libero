use super::focus_from;
use crate::components::DemoValues;
use dioxus::prelude::*;
use libero::components::{ActionIcon, Flex, Textarea, Toolbar, ToolbarGroup, ToolbarSeparator};
use libero::hooks::use_element;
use pictogram_icons_lucide as lucide;

/// Owns the editor's handle, which `render` cannot hold.
#[component]
pub fn ToolbarPreview(values: DemoValues) -> Element {
    let editor = use_element();
    let from = focus_from(&values);
    // The editor under the bar, so at 375px the bar keeps its row (todo 4708).
    rsx! { Flex { gap: "sm",
        Toolbar {
            "aria-label": "Formatting",
            orientation: values.str("orientation"),
            loop_focus: values.str("loop_focus") == "true",
            focus_from: from.then_some(editor),
            // demo-code: children start
            ToolbarGroup { "aria-label": "Style",
                ActionIcon { aria_label: "Bold", icon: lucide::bold::outlined }
                ActionIcon { aria_label: "Italic", icon: lucide::italic::outlined }
                ActionIcon { aria_label: "Underline", icon: lucide::underline::outlined }
            }
            ToolbarSeparator {}
            ToolbarGroup { "aria-label": "History",
                ActionIcon { aria_label: "Undo", icon: lucide::undo_2::outlined }
                ActionIcon { aria_label: "Redo", disabled: true, icon: lucide::redo_2::outlined }
            }
            // demo-code: children end
        }
        if from {
            // demo-code: editor start
            div { onmounted: editor.mount(), ..editor.attributes(),
                Textarea { label: "Text" }
            }
            // demo-code: editor end
        }
    } }
}
