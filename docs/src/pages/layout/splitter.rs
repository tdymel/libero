use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, or_unset};
use dioxus::prelude::*;
use libero::{
    components::{Box, Input, Splitter, Text},
    sx::sx,
};

const PANEL_A: &str = r#"panel_a: rsx! { Box { sx: sx().height("100%").padding("md").background("primary.1"), "A" } }"#;
const PANEL_B: &str = r#"panel_b: rsx! { Box { sx: sx().height("100%").padding("md").background("secondary.1"), "B" } }"#;

/// `panel_b` holding a nested `Splitter` - the composed case, printed with the
/// continuation indent the generated prop list expects.
const NESTED_PANEL_B: &str = r#"panel_b: rsx! {
        Splitter {
            orientation: "horizontal",
            initial_size: 65.0,
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
        "Box {{\n    sx: sx().height(\"160px\").width(\"320px\").border(\"1px solid var(--lsx-grey-3)\"),\n{indented}}}"
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
            lead: rsx! {
                Text { "Splits two panes with a draggable/keyboard-resizable divider. Panes go in `panel_a` and `panel_b`; nest another `Splitter` in a pane for more than two." }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Splitter",
                    children_text: "",
                    // Required, and not something a control varies.
                    fixed: vec!["initial_size: 50.0".to_string()],
                    controls: vec![
                        Control::toggle("orientation", ["vertical", "horizontal"]),
                        Control::slider("min_size", ["10", "20", "30", "40"]).code(percent_code),
                        Control::slider("divider_size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("sm"),
                        // A bare `grey` is what an unset `divider_color`
                        // falls back to, so that swatch prints nothing.
                        Control::color(
                            "divider_color",
                            ["grey", "primary", "secondary", "success", "error", "warning", "info"],
                        ),
                        // Last, so the panes print below the props they
                        // configure.
                        Control::switch("composed").code(panels_code),
                    ],
                    render: move |values: DemoValues| rsx! {
                        Box {
                            sx: sx()
                                .height("160px")
                                .width("320px")
                                .border("1px solid var(--lsx-grey-3)"),
                            Splitter {
                                orientation: values.str("orientation"),
                                initial_size: 50.0,
                                min_size: percent(values.str("min_size")),
                                divider_size: or_unset(values.str("divider_size")),
                                divider_color: match values.str("divider_color").as_str() {
                                    "grey" => Input::None,
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
            }
        }
    }
}
