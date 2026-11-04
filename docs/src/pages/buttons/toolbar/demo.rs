use super::{focus_from, responsive_on};
use crate::components::DemoValues;
use dioxus::prelude::*;
use libero::components::{
    ActionIcon, Flex, Orientation, Textarea, Toolbar, ToolbarGroup, ToolbarSeparator,
};
use libero::hooks::use_element;
use libero::theme::responsive;
use pictogram_icons_lucide as lucide;

/// Owns the editor's handle, which `render` cannot hold.
#[component]
pub fn ToolbarPreview(values: DemoValues) -> Element {
    let editor = use_element();
    let from = focus_from(&values);
    // A formatting bar's B, I and U are toggles (APG toolbar, todo 5170).
    // demo-code: signals start
    let mut bold = use_signal(|| false);
    let mut italic = use_signal(|| false);
    let mut underline = use_signal(|| false);
    // demo-code: signals end
    // The editor under the bar, so at 375px the bar keeps its row (todo 4708).
    rsx! { Flex { gap: "sm",
        Toolbar {
            "aria-label": "Formatting",
            orientation: match (responsive_on(&values), Orientation::from(values.str("orientation").as_str())) {
                (true, chosen) => responsive(Orientation::Vertical).md(chosen),
                (false, chosen) => chosen.into(),
            },
            loop_focus: values.str("loop_focus") == "true",
            focus_from: from.then_some(editor),
            // demo-code: children start
            ToolbarGroup { "aria-label": "Style",
                ActionIcon {
                    aria_label: "Bold",
                    selected: bold(),
                    onclick: move |_| bold.toggle(),
                    icon: lucide::bold::outlined,
                }
                ActionIcon {
                    aria_label: "Italic",
                    selected: italic(),
                    onclick: move |_| italic.toggle(),
                    icon: lucide::italic::outlined,
                }
                ActionIcon {
                    aria_label: "Underline",
                    selected: underline(),
                    onclick: move |_| underline.toggle(),
                    icon: lucide::underline::outlined,
                }
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
