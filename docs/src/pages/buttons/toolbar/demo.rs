use super::focus_from;
use crate::components::DemoValues;
use dioxus::prelude::*;
use libero::components::{
    ActionIcon, Pictogram, Textarea, Toolbar, ToolbarGroup, ToolbarSeparator,
};
use libero::hooks::use_element;
use pictogram_icons_lucide as lucide;

/// Owns the editor's handle, which `render` cannot hold.
#[component]
pub fn ToolbarPreview(values: DemoValues) -> Element {
    let editor = use_element();
    let from = focus_from(&values);
    rsx! {
        Toolbar {
            "aria-label": "Formatting",
            orientation: values.str("orientation"),
            loop_focus: values.str("loop_focus") == "true",
            focus_from: from.then_some(editor),
            // demo-code: children start
            ToolbarGroup { "aria-label": "Style",
                ActionIcon { aria_label: "Bold", Pictogram { icon: lucide::bold::outlined } }
                ActionIcon { aria_label: "Italic", Pictogram { icon: lucide::italic::outlined } }
                ActionIcon { aria_label: "Underline", Pictogram { icon: lucide::underline::outlined } }
            }
            ToolbarSeparator {}
            ToolbarGroup { "aria-label": "History",
                ActionIcon { aria_label: "Undo", Pictogram { icon: lucide::undo_2::outlined } }
                ActionIcon { aria_label: "Redo", disabled: true, Pictogram { icon: lucide::redo_2::outlined } }
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
    }
}
