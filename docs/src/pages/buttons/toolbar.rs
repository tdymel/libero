use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::components::{
    ActionIcon, Code, Pictogram, Text, Textarea, Toolbar, ToolbarGroup, ToolbarPart,
    ToolbarSeparator,
};
use libero::hooks::use_element;
use pictogram_icons_lucide as lucide;

/// The controls inside - a subtree, so the code block prints it verbatim.
// snippet: mirrors ToolbarPreview
const CHILDREN: &str = r#"ToolbarGroup { "aria-label": "Style",
    ActionIcon { aria_label: "Bold", Pictogram { icon: lucide::bold::outlined } }
    ActionIcon { aria_label: "Italic", Pictogram { icon: lucide::italic::outlined } }
    ActionIcon { aria_label: "Underline", Pictogram { icon: lucide::underline::outlined } }
}
ToolbarSeparator {}
ToolbarGroup { "aria-label": "History",
    ActionIcon { aria_label: "Undo", Pictogram { icon: lucide::undo_2::outlined } }
    ActionIcon { aria_label: "Redo", disabled: true, Pictogram { icon: lucide::redo_2::outlined } }
}"#;

/// The element `focus_from` names: Alt+F10 in it reaches the bar, Escape there comes back.
// snippet: let editor = use_element();
// snippet: mirrors ToolbarPreview
const EDITOR: &str = r#"div { onmounted: editor.mount(), ..editor.attributes(),
    Textarea { label: "Text" }
}"#;

fn focus_from(values: &DemoValues) -> bool {
    values.str("focus_from") == "true"
}

/// With `focus_from`, prints the handle and the editor it names.
fn wrap(values: &DemoValues, source: &str) -> String {
    if !focus_from(values) {
        return source.to_string();
    }
    let mut code = String::from("let editor = use_element();\n\nrsx! {\n");
    code.push_str(&indent(source));
    code.push_str(&indent(EDITOR));
    code.push('}');
    code
}

/// Owns the editor's handle, which `render` cannot hold.
#[component]
fn ToolbarPreview(values: DemoValues) -> Element {
    let editor = use_element();
    let from = focus_from(&values);
    rsx! {
        Toolbar {
            "aria-label": "Formatting",
            orientation: values.str("orientation"),
            loop_focus: values.str("loop_focus") == "true",
            focus_from: from.then_some(editor),
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
        }
        if from {
            div { onmounted: editor.mount(), ..editor.attributes(),
                Textarea { label: "Text" }
            }
        }
    }
}

#[component]
pub fn ToolbarPage() -> Element {
    rsx! {
        DocPage {
            title: "Toolbar",
            source: "libero/src/components/buttons/toolbar.rs",
            markdown: "/md/toolbar.md",
            properties: vec![
                props("Toolbar", vec![
                    prop("orientation", "Orientation")
                        .default("horizontal")
                        .doc("`\"vertical\"` stacks the controls; Up and Down move instead of Left and Right."),
                    prop("loop_focus", "bool")
                        .default("true")
                        .doc("Whether the arrow keys wrap at the ends."),
                    prop("focus_from", "Option<ElementHandle>")
                        .doc("The element the bar serves, such as an editor: Alt+F10 inside it moves focus to the bar, Escape in the bar hands it back. Spread its `attributes()`."),
                    prop("parts", "Parts<ToolbarPart>")
                        .doc("Styles for the groups and separators, under `sx`."),
                    prop("children", "Element")
                        .default("required")
                        .doc("`Button`s, `ActionIcon`s, `Select`s, `ButtonGroup`s, fields, `ToolbarGroup`s and `ToolbarSeparator`s."),
                ])
                .parts("ToolbarPart", vec![
                    (ToolbarPart::Group, "A `ToolbarGroup`, laid out along the bar."),
                    (ToolbarPart::Separator, "A `ToolbarSeparator`, a 1px line across the bar."),
                ]),
                props("ToolbarGroup", vec![
                    prop("children", "Element")
                        .default("required")
                        .doc("The section's controls. They stay in the bar's arrow order."),
                ]),
                props("ToolbarSeparator", vec![]),
            ],
            accessibility: a11y()
                .key(["Tab"], "Enters the bar on the control focused last, the first one at the start, and leaves it.")
                .key(["Left", "Right"], "Moves to the previous or next control, disabled ones included. Swapped under right-to-left text. Up and Down in a vertical bar.")
                .key(["Home", "End"], "Goes to the first or last control.")
                .key(["Alt", "F10"], "With `focus_from`, moves from that element to the bar's tab stop.")
                .key(["Escape"], "After Alt+F10, hands focus back to where it was, until focus leaves the bar; a menu or dialog the bar opened does not count.")
                .handles([
                    "The root is a `role=\"toolbar\"`, with `aria-orientation=\"vertical\"` when vertical.",
                    "The bar is one tab stop: the controls inside get a roving `tabindex`.",
                    "A disabled `Button`, `ActionIcon` or `Select` inside stays focusable with `aria-disabled`, so a keyboard user finds it; `focusable_when_disabled: false` opts out. A disabled field stays focusable too, read-only with `aria-disabled`.",
                    "A key a control uses itself stays its own: `Select` opens on Home, End and Up, a slider keeps its arrows.",
                    "`Checkbox`, `Switch`, `SegmentedControl`, `TextField` and `NumberField` join the arrow order. A `SegmentedControl` moves through its segments and passes the arrow on past its last one.",
                    "A text field keeps Left and Right until the caret sits at its start or end with nothing selected; then the arrow moves on. Home and End stay the field's.",
                    "`NumberField`'s Up and Down stay its stepper, in a vertical bar too; its stepper buttons stay out of the arrow order.",
                    "`ToolbarGroup` is a `role=\"group\"`; `ToolbarSeparator` a `role=\"separator\"` across the bar.",
                ])
                .must([
                    "Name the bar with `aria-label`, or `aria-labelledby` on a visible heading. Without either it warns in debug builds.",
                    "Name each `ToolbarGroup` with `aria-label`. Without it the group warns in debug builds too.",
                ])
                .limits([
                    "Only `Button`, `ActionIcon`, `Select`, `Checkbox`, `Switch`, `SegmentedControl`, `TextField` and `NumberField` (and what is built on them) join the arrow order. Another focusable element inside stays its own tab stop.",
                ]),
            lead: rsx! {
                Text {
                    "A row of controls that is one tab stop: the arrow keys move between them, "
                    "Home and End jump to the ends. An editor's formatting bar is the typical use."
                }
                Text {
                    "Put "
                    Code { source: "Button" }
                    "s, "
                    Code { source: "ActionIcon" }
                    "s, "
                    Code { source: "ButtonGroup" }
                    "s and "
                    Code { source: "Select" }
                    "s inside, sectioned by "
                    Code { source: "ToolbarGroup" }
                    " and "
                    Code { source: "ToolbarSeparator" }
                    ". For buttons that stay separate tab stops, use "
                    Code { source: "ButtonGroup" }
                    " alone."
                }
            },
            Demo {
                component: "Toolbar",
                children_text: "",
                children_code: CHILDREN,
                fixed: vec!["\"aria-label\": \"Formatting\"".to_string()],
                controls: vec![
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"])
                        .default("horizontal"),
                    Control::switch("loop_focus").default("true"),
                    Control::switch("focus_from").code(|_, values| match focus_from(values) {
                        true => vec!["focus_from: editor".to_string()],
                        false => vec![],
                    }),
                ],
                wrap: Wrap(wrap),
                render: move |values: DemoValues| rsx! {
                    ToolbarPreview { values }
                },
            }
        }
    }
}
