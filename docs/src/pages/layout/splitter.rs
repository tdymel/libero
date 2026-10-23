use crate::components::{
    Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, or_unset, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Input, Splitter, SplitterResizeEvent, Text},
    sx::sx,
};

// snippet: in Splitter { initial_size: 50.0, aria_label: "Resize panes", panel_b: rsx! {}, .. }
const PANEL_A: &str = r#"panel_a: rsx! { Box { sx: sx().height("100%").padding("md").background("primary.1"), "A" } }"#;
// snippet: ignore - `size` exists only in the printed body
const SIZED_PANEL_A: &str = r#"panel_a: rsx! { Box { sx: sx().height("100%").padding("md").background("primary.1"), "A {size:.0}%" } }"#;
// snippet: in Splitter { initial_size: 50.0, aria_label: "Resize panes", panel_a: rsx! {}, .. }
const PANEL_B: &str = r#"panel_b: rsx! { Box { sx: sx().height("100%").padding("md").background("secondary.1"), "B" } }"#;

/// `panel_b` holding a nested `Splitter` - the composed case, printed with the
/// continuation indent the generated prop list expects.
// snippet: in Splitter { initial_size: 50.0, aria_label: "Resize panes", panel_a: rsx! {}, .. }
const NESTED_PANEL_B: &str = r#"panel_b: rsx! {
        Splitter {
            orientation: "horizontal",
            initial_size: 65.0,
            aria_label: "Resize pane B",
            panel_a: rsx! { Box { sx: sx().height("100%").padding("md").background("secondary.1"), "B" } },
            panel_b: rsx! { Box { sx: sx().height("100%").padding("md").background("info.1"), "C" } },
        }
    }"#;

fn is_controlled(values: &DemoValues) -> bool {
    values.str("controlled") == "true"
}

/// Both panes hang off the `composed` switch, so they are printed by it
/// rather than held fixed. Controlled, pane A shows the size it is held at.
fn panels_code(_control: &Control, values: &DemoValues) -> Vec<String> {
    let nested = values.str("composed") == "true";
    vec![
        match is_controlled(values) {
            true => SIZED_PANEL_A.to_string(),
            false => PANEL_A.to_string(),
        },
        match nested {
            true => NESTED_PANEL_B.to_string(),
            false => PANEL_B.to_string(),
        },
    ]
}

/// A splitter fills its container, so the preview has to give it one. The code
/// block prints that wrapper, and the signal a controlled splitter reads.
fn wrap_splitter(values: &DemoValues, code: &str) -> String {
    let indented = indent(code);
    let size = match is_controlled(values) {
        true => "let mut size = use_signal(|| 30.0);\n\n",
        false => "",
    };
    format!(
        "{size}Box {{\n    sx: sx().height(\"160px\").width(\"100%\").max_width(\"320px\").border(\"1px solid var(--lsx-muted-3)\"),\n{indented}}}"
    )
}

/// The percentage controls print unquoted floats, not strings.
fn percent_code(control: &Control, values: &DemoValues) -> Vec<String> {
    let value = values.str(control.name);
    match value == control.default {
        true => vec![],
        false => vec![format!("{}: {value}.0", control.name)],
    }
}

/// `B` alone, or `B` plus `C` split the other way - the composed case, where
/// a pane holds another `Splitter`.
fn panel_b(values: &DemoValues) -> Element {
    let pane = |color: &'static str, label: &'static str| {
        rsx! {
            Box { sx: sx().height("100%").padding("md").background(color), {label} }
        }
    };

    match values.str("composed") == "true" {
        false => pane("secondary.1", "B"),
        true => rsx! {
            Splitter {
                orientation: "horizontal",
                initial_size: 65.0,
                aria_label: "Resize pane B",
                panel_a: pane("secondary.1", "B"),
                panel_b: pane("info.1", "C"),
            }
        },
    }
}

/// `initial_size`, or the controlled pair: `onresize` stores every move, or snaps it to 10%,
/// which the panes then follow. Printed whole: `size` exists only in the printed body.
fn size_code(_control: &Control, values: &DemoValues) -> Vec<String> {
    if !is_controlled(values) {
        return vec!["initial_size: 50.0".to_string()];
    }
    let stored = match values.str("snap") == "true" {
        true => "(a / 10.0).round() * 10.0",
        false => "a",
    };
    vec![
        "size: size()".to_string(),
        format!(
            "onresize: move |event: SplitterResizeEvent| if let SplitterResizeEvent::Change(a, _) = event {{ size.set({stored}) }}"
        ),
    ]
}

/// Pane A's size lives here when `controlled`; with `snap` it is stored rounded to 10%.
#[component]
fn SplitterDemo(values: DemoValues) -> Element {
    let mut size = use_signal(|| 30.0);
    let controlled = is_controlled(&values);
    let snap = values.str("snap") == "true";
    let label = match controlled {
        true => format!("A {:.0}%", size()),
        false => "A".to_string(),
    };

    rsx! {
        Box {
            sx: sx()
                .height("160px")
                .width("100%")
                .max_width("320px")
                .border("1px solid var(--lsx-muted-3)"),
            Splitter {
                orientation: values.str("orientation"),
                initial_size: 50.0,
                size: controlled.then_some(size()),
                aria_label: "Resize panes",
                min_size: percent(values.str("min_size")),
                divider_size: or_unset(values.str("divider_size")),
                divider_color: match values.str("divider_color").as_str() {
                    "muted" => Input::None,
                    color => Input::from(color),
                },
                onresize: move |event: SplitterResizeEvent| {
                    if let (true, SplitterResizeEvent::Change(a, _)) = (controlled, event) {
                        size.set(if snap { (a / 10.0).round() * 10.0 } else { a });
                    }
                },
                panel_a: rsx! {
                    Box {
                        sx: sx().height("100%").padding("md").background("primary.1"),
                        "{label}"
                    }
                },
                panel_b: panel_b(&values),
            }
        }
    }
}

fn percent(value: String) -> f64 {
    value.parse().unwrap_or(0.0)
}

#[component]
pub fn SplitterPage() -> Element {
    rsx! {
        DocPage {
            title: "Splitter",
            source: "libero/src/components/layout/splitter",
            markdown: "/md/splitter.md",
            properties: vec![props("Splitter", vec![
                prop("orientation", "Orientation")
                    .default("vertical")
                    .doc("The divider's axis. `vertical` puts the panes side by side, `horizontal` stacks them."),
                prop("initial_size", "f64").default("50")
                    .doc("Pane A's starting size in percent, kept within `min_size`. After that the divider owns the size, and `onresize` reports it. Ignored when `size` is set."),
                prop("size", "Option<f64>")
                    .default("None")
                    .doc("Pane A's size in percent, kept within `min_size`. Set, the splitter is controlled: a drag, key or double-click only reports through `onresize`, and the panes follow `size`."),
                prop("min_size", "f64").default("10").doc("The smallest size of either pane in percent, at most 50."),
                prop("divider_size", "Size").default("sm").doc("Thickness of the divider line."),
                prop("divider_color", "ThemeAwareValue")
                    .doc("The divider's color. Unset it is `muted.6`, 3:1 on the page and `Paper`."),
                prop("onresize", "EventHandler<SplitterResizeEvent>")
                    .doc("Fires as the divider moves, with both panes' sizes in percent, A first. A drag sends `Start`, `Change`, then `End`; a key press or double-click sends `Change` then `End`, so saving the layout on `End` catches every resize."),
                prop("aria_label", "String")
                    .doc("Names the divider after the pane it resizes. A debug build warns without it."),
                prop("panel_a", "Element").default("required").doc("The start or top pane."),
                prop("panel_b", "Element").default("required").doc("The end or bottom pane. For more than two panes, put a `Splitter` in a pane, split the other way."),
            ])],
            accessibility: a11y()
                .key(["Tab"], "Focuses the divider, a tab stop.")
                .key(["Left", "Right"], "Moves a vertical divider by 1%.")
                .key(["Up", "Down"], "Moves a horizontal divider by 1%.")
                .key(["Shift+Left", "Shift+Right", "Shift+Up", "Shift+Down"], "Moves the divider by 10%.")
                .key(["Home", "End"], "Jumps to either limit.")
                .handles([
                    "The divider is a focusable separator.",
                    "Double-clicking the divider resizes without dragging (WCAG 2.5.7): pane A collapses to `min_size`, and the next double-click restores the size it had before the move that reached `min_size`, by a double-click, a drag or `Home`. Arrow keys walking down to `min_size` restore the size one step above it. A single click only focuses the divider.",
                    "In forced colours the divider line takes the system text colour, so it stays visible.",
                    "The divider's hit area is 24px thick (WCAG 2.5.8), so it takes presses up to 24px minus the divider's thickness into pane B. Pane A's own scrollbar stays free to grab.",
                    "Each pane scrolls its own overflow, so a pane at `min_size` never paints over the divider or hides its controls under the other pane.",
                    "A debug build warns without `aria_label`.",
                ])
                .must([
                    "Set `aria_label` to name the divider after the pane it resizes, such as `\"Resize sidebar\"`. It has no name of its own.",
                    "Keep buttons at pane B's start edge out of the 24px next to the divider, which gets no press there. For example, use `padding-inline-start: 24px` on pane B of a side-by-side split, `padding-block-start: 24px` of a stacked one.",
                ])
                .example("A file sidebar beside an editor, `Splitter { aria_label: \"Resize sidebar\", .. }`: Tab reaches the divider, Left and Right move it by 1%, and a double-click collapses the sidebar without a drag."),
            lead: rsx! {
                Text {
                    "Two panes, "
                    Code { source: "panel_a" }
                    " and "
                    Code { source: "panel_b" }
                    ", split by a divider you can drag or move with the keyboard. Nest "
                    "another "
                    Code { source: "Splitter" }
                    " in a pane for more than two. It fills its parent, so give the parent "
                    "a size. Set "
                    Code { source: "size" }
                    " to hold pane A's size yourself, stored from "
                    Code { source: "onresize" }
                    "."
                }
            },
            Demo {
                component: "Splitter",
                children_text: "",
                // Required, and not something a control varies.
                fixed: vec![r#"aria_label: "Resize panes""#.to_string()],
                controls: vec![
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"])
                        .default("vertical"),
                    Control::slider("min_size", ["10", "20", "30", "40"]).code(percent_code),
                    Control::sizes("divider_size")
                        .default("sm"),
                    // A bare `grey` is what an unset `divider_color`
                    // falls back to, so that swatch prints nothing.
                    Control::color("divider_color").default("muted"),
                    // Holds pane A's size in a signal instead of `initial_size`.
                    Control::switch("controlled").code(size_code),
                    // Not a prop: how the caller stores the size, printed by `controlled`.
                    Control::switch("snap")
                        .code(|_, _| vec![])
                        .hidden_when(|values| !is_controlled(values)),
                    // Last, so the panes print below the props they
                    // configure.
                    Control::switch("composed").code(panels_code),
                ],
                render: move |values: DemoValues| rsx! { SplitterDemo { values } },
                wrap: Wrap(wrap_splitter),
            }
        }
    }
}
