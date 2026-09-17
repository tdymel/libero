use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, or_unset, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Input, Kbd, List, ListItem, Splitter, Text},
    sx::sx,
};

// snippet: in Splitter { initial_size: 50.0, aria_label: "Resize panes", panel_b: rsx! {}, .. }
const PANEL_A: &str = r#"panel_a: rsx! { Box { sx: sx().height("100%").padding("md").background("primary.1"), "A" } }"#;
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

/// Both panes hang off the `composed` switch, so they are printed by it
/// rather than held fixed.
fn panels_code(_control: &Control, values: &DemoValues) -> Vec<String> {
    let nested = values.str("composed") == "true";
    vec![
        PANEL_A.to_string(),
        match nested {
            true => NESTED_PANEL_B.to_string(),
            false => PANEL_B.to_string(),
        },
    ]
}

/// A splitter fills its container, so the preview has to give it one. The code
/// block prints that wrapper.
fn wrap_splitter(_values: &DemoValues, code: &str) -> String {
    let indented = indent(code);
    format!(
        "Box {{\n    sx: sx().height(\"160px\").width(\"100%\").max_width(\"320px\").border(\"1px solid var(--lsx-muted-3)\"),\n{indented}}}"
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
                prop("initial_size", "f64")
                    .doc("Pane A's starting size in percent, kept within `min_size`. After that the divider owns the size, and `onresize` reports it."),
                prop("min_size", "f64").default("10").doc("The smallest size of either pane in percent, at most 50."),
                prop("divider_size", "Size").default("sm").doc("Thickness of the divider line."),
                prop("divider_color", "ThemeAwareValue").doc("The divider's color."),
                prop("onresize", "EventHandler<SplitterResizeEvent>")
                    .doc("Fires as the divider moves, with both panes' sizes in percent. A key press or double-click sends `Change` then `End`."),
                prop("aria_label", "String")
                    .doc("Names the divider after the pane it resizes. A debug build warns without it."),
                prop("panel_a", "Element").doc("The start or top pane."),
                prop("panel_b", "Element").doc("The end or bottom pane."),
            ])],
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
                    "a size."
                }
            },
            Demo {
                component: "Splitter",
                children_text: "",
                // Required, and not something a control varies.
                fixed: vec![
                    "initial_size: 50.0".to_string(),
                    r#"aria_label: "Resize panes""#.to_string(),
                ],
                controls: vec![
                    Control::toggle("orientation", ["vertical", "horizontal"]),
                    Control::slider("min_size", ["10", "20", "30", "40"]).code(percent_code),
                    Control::slider("divider_size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    // A bare `grey` is what an unset `divider_color`
                    // falls back to, so that swatch prints nothing.
                    Control::color("divider_color").default("muted"),
                    // Last, so the panes print below the props they
                    // configure.
                    Control::switch("composed").code(panels_code),
                ],
                render: move |values: DemoValues| rsx! {
                    Box {
                        sx: sx()
                            .height("160px")
                            .width("100%")
                            .max_width("320px")
                            .border("1px solid var(--lsx-muted-3)"),
                        Splitter {
                            orientation: values.str("orientation"),
                            initial_size: 50.0,
                            aria_label: "Resize panes",
                            min_size: percent(values.str("min_size")),
                            divider_size: or_unset(values.str("divider_size")),
                            divider_color: match values.str("divider_color").as_str() {
                                "muted" => Input::None,
                                color => Input::from(color),
                            },
                            panel_a: rsx! {
                                Box {
                                    sx: sx()
                                        .height("100%")
                                        .padding("md")
                                        .background("primary.1"),
                                    "A"
                                }
                            },
                            panel_b: panel_b(&values),
                        }
                    }
                },
                wrap: Wrap(wrap_splitter),
            }

            DocSection {
                title: "Accessibility",
                List {
                    ListItem {
                        "Set "
                        Code { source: "aria_label" }
                        " to name the divider after the pane it resizes."
                    }
                    ListItem {
                        "The divider is a tab stop. "
                        Kbd { "←" } " " Kbd { "→" } ", or " Kbd { "↑" } " " Kbd { "↓" }
                        " when horizontal, move it, with " Kbd { "Shift" } " in bigger steps. "
                        Kbd { "Home" } " and " Kbd { "End" } " jump to either limit."
                    }
                    ListItem {
                        "Double-click the divider to collapse pane A to "
                        Code { source: "min_size" }
                        ", and again to restore it."
                    }
                    ListItem {
                        "The divider takes presses about 12px into each pane. Keep a pane's "
                        "scrollbar or edge buttons out of that gutter, for example with "
                        Code { source: "padding: 12px" }
                        " on the side next to the divider."
                    }
                }
            }
        }
    }
}
