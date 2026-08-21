use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, or_unset};
use dioxus::prelude::*;
use libero::{
    components::{Box, Input, Splitter, Text},
    sx::sx,
};

const PANEL_A: &str = r#"panel_a: rsx! { Box { sx: sx().height("100%").padding("md").background("primary.1"), "A" } }"#;
const PANEL_B: &str = r#"panel_b: rsx! { Box { sx: sx().height("100%").padding("md").background("secondary.1"), "B" } }"#;

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
                    // Required, and both panes are the fixture the props act
                    // on rather than anything a control varies.
                    fixed: vec![
                        "initial_size: 50.0".to_string(),
                        PANEL_A.to_string(),
                        PANEL_B.to_string(),
                    ],
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
                                panel_b: rsx! {
                                    Box {
                                        sx: sx()
                                            .height("100%")
                                            .padding("md")
                                            .background("secondary.1"),
                                        "B"
                                    }
                                },
                            }
                        }
                    },
                    wrap: Wrap(wrap_splitter),
                }
            }
            DocSection {
                title: "Composed",
                Text { "Nesting `Splitter`s composes more than 2 panes - here a horizontal split on the right of a vertical one." }
                Box {
                    sx: sx().height("260px").border("1px solid var(--lsx-grey-3)"),
                    Splitter {
                        initial_size: 35.0,
                        panel_a: rsx! { Box { sx: sx().height("100%").padding("md").background("primary.1"), "Sidebar" } },
                        panel_b: rsx! {
                            Splitter {
                                orientation: "horizontal",
                                initial_size: 65.0,
                                panel_a: rsx! { Box { sx: sx().height("100%").padding("md").background("secondary.1"), "Main" } },
                                panel_b: rsx! { Box { sx: sx().height("100%").padding("md").background("info.1"), "Panel" } },
                            }
                        },
                    }
                }
            }
        }
    }
}
