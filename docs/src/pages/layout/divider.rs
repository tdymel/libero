use crate::components::{Child, Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent};
use dioxus::prelude::*;
use libero::{
    components::{Box, Divider, Input, Text},
    sx::sx,
};

/// The label is a child, not a prop, so a control has to decide it here.
fn label_child(values: &DemoValues) -> String {
    match values.str("with_label").as_str() {
        "true" => "OR".to_string(),
        _ => String::new(),
    }
}

/// A rule has no size of its own, and `spacing` is a margin - so the preview
/// gives it both an axis to span and something to be spaced from. The code
/// block prints that wrapper.
fn wrap_rule(values: &DemoValues, code: &str) -> String {
    let indented = indent(code);
    match values.str("orientation").as_str() {
        "vertical" => format!(
            "Box {{\n    sx: sx().display(\"flex\").align_items(\"center\").justify_content(\"center\").height(\"64px\"),\n    Text {{ \"Left\" }}\n{indented}    Text {{ \"Right\" }}\n}}"
        ),
        _ => format!(
            "Box {{\n    sx: sx().width(\"240px\").text_align(\"center\"),\n    Text {{ \"Above\" }}\n{indented}    Text {{ \"Below\" }}\n}}"
        ),
    }
}

#[component]
pub fn DividerPage() -> Element {
    rsx! {
        DocPage {
            title: "Divider",
            lead: rsx! {
                Text { "A horizontal or vertical rule, with an optional centered/positioned label." }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Divider",
                    children_text: "OR",
                    controls: vec![
                        Control::toggle("orientation", ["horizontal", "vertical"]),
                        Control::switch("with_label").default("true").code(|_, _| vec![]),
                        // Meaningless without a label, so it doesn't print then.
                        Control::toggle("label_position", ["center", "start", "end"]).code(
                            |control, values| {
                                let value = values.str("label_position");
                                match values.str("with_label") == "true" && value != control.default
                                {
                                    true => vec![format!("label_position: {value:?}")],
                                    false => vec![],
                                }
                            },
                        ),
                        Control::color(
                            "color",
                            ["grey", "primary", "secondary", "success", "error", "warning", "info"],
                        ),
                        Control::slider("spacing", ["auto", "xs", "sm", "md", "lg", "xl"])
                            .default("md")
                            .code(
                                |_, values| match values.str("spacing").as_str() {
                                    // The theme's own default, not a value.
                                    "auto" => vec![],
                                    spacing => vec![format!("spacing: {spacing:?}")],
                                },
                            ),
                    ],
                    render: move |values: DemoValues| {
                        let divider = rsx! {
                            Divider {
                                orientation: values.str("orientation"),
                                label_position: values.str("label_position"),
                                // A bare `grey` is grey-3; *unset* is grey-4,
                                // and unset is what the code block prints.
                                color: match values.str("color").as_str() {
                                    "grey" => Input::None,
                                    color => Input::from(color),
                                },
                                // An empty string would be a literal value,
                                // not "leave it to the theme".
                                spacing: match values.str("spacing").as_str() {
                                    "auto" => Input::None,
                                    spacing => Input::from(spacing),
                                },
                                children: (values.str("with_label") == "true")
                                    .then(|| rsx! { "OR" }),
                            }
                        };
                        match values.str("orientation").as_str() {
                            "vertical" => rsx! {
                                Box {
                                    sx: sx()
                                        .display("flex")
                                        .align_items("center")
                                        .justify_content("center")
                                        .height("64px"),
                                    Text { "Left" }
                                    {divider}
                                    Text { "Right" }
                                }
                            },
                            _ => rsx! {
                                Box {
                                    sx: sx().width("240px").text_align("center"),
                                    Text { "Above" }
                                    {divider}
                                    Text { "Below" }
                                }
                            },
                        }
                    },
                    wrap: Wrap(wrap_rule),
                    child: Child(label_child),
                }
            }
        }
    }
}
