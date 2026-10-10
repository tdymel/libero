use crate::components::{
    Control, Demo, DemoFile, DemoValues, DocPage, Wrap, a11y, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Code, Text, ToolbarPart},
    use_theme,
};

mod demo;
use demo::ToolbarPreview;

/// The live preview; its groups and the editor print from it.
const FILE: DemoFile = DemoFile(include_str!("toolbar/demo.rs"));

pub(super) fn focus_from(values: &DemoValues) -> bool {
    values.str("focus_from") == "true"
}

/// With `focus_from`, prints the handle and the editor it names.
fn wrap(values: &DemoValues, source: &str) -> String {
    if !focus_from(values) {
        return source.to_string();
    }
    let mut code = String::from("let editor = use_element();\n\nrsx! {\n");
    code.push_str(&indent(source));
    code.push_str(&indent(&FILE.section("editor")));
    code.push('}');
    code
}

#[component]
pub fn ToolbarPage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Toolbar",
            source: "libero/src/components/buttons/toolbar.rs",
            markdown: "/md/toolbar.md",
            properties: vec![
                props("Toolbar", vec![
                    prop("orientation", "Orientation")
                        .default("horizontal")
                        .doc("`\"vertical\"` stacks the controls; Up and Down move instead of Left and Right. Not themed."),
                    prop("loop_focus", "bool")
                        .default(theme.toolbar.loop_focus.to_string())
                        .doc("Whether the arrow keys wrap at the ends."),
                    prop("focus_from", "Option<ElementHandle>")
                        .doc("The element the bar serves, such as an editor: Alt+F10 inside it moves focus to the bar, Escape in the bar hands it back. Spread its `attributes()`."),
                    prop("parts", "Parts<ToolbarPart>")
                        .doc("Styles for the groups and separators, under `sx`."),
                    prop("children", "Element")
                        .default("required")
                        .doc("`Button`s, `ActionIcon`s, `Select`s, `ButtonGroup`s, `TextField`s, `NumberField`s, `Checkbox`es, `Switch`es, `SegmentedControl`s, `ToolbarGroup`s and `ToolbarSeparator`s."),
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
                .example("An editor bar, `Toolbar { \"aria-label\": \"Formatting\" }` with `ToolbarGroup`s named \"Style\" and \"History\": Tab enters on one button, the arrows move through Bold, Italic, Undo and Redo, and Tab leaves the bar.")
                .limits([
                    "Only `Button`, `ActionIcon`, `Select`, `Checkbox`, `Switch`, `SegmentedControl`, `TextField` and `NumberField` (and what is built on them) join the arrow order. Another focusable element inside stays its own tab stop.",
                    "A long horizontal bar wraps. A `ToolbarSeparator` stays where the flow puts it, so it can end a row; it is not dropped at a wrap.",
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
                children_code: FILE.section("children"),
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
