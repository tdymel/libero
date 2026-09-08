use crate::components::{
    Child, Control, Demo, DemoValues, DocPage, UNSET, Wrap, indent, prop, props,
};
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
            source: "libero/src/components/layout/divider.rs",
            markdown: "/md/divider.md",
            properties: vec![props("Divider", vec![
                prop("orientation", "Orientation")
                    .default("horizontal")
                    .doc("Horizontal or vertical rule."),
                prop("size", "Size").default("xs").doc("Line thickness."),
                prop("label_position", "LabelPosition")
                    .default("center")
                    .doc("Where the label sits along the rule."),
                prop("spacing", "ThemeAwareValue")
                    .default("none")
                    .doc("Margin on either side of the rule, from the spacing scale."),
                prop("color", "ThemeAwareValue")
                    .default("muted.4")
                    .doc("Line color. A bare theme color is tinted to shade 3."),
                prop("children", "Element")
                    .doc("The optional centered/positioned label."),
            ])],
            lead: rsx! {
                Text { "A horizontal or vertical rule, with an optional centered/positioned label." }
            },
            Demo {
                component: "Divider",
                children_text: "OR",
                controls: vec![
                    Control::toggle("orientation", ["horizontal", "vertical"]),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("xs"),
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
                    // The unset line is grey-4, which a bare `grey` would
                    // *not* resolve to - it is tinted to shade 3 like every
                    // other bare color. So the first swatch is unset, painted
                    // the grey-4 the rule actually draws.
                    Control::color("color").with_unset()
                    .unset_swatch("muted.4"),
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
                            size: values.str("size"),
                            label_position: values.str("label_position"),
                            color: match values.str("color").as_str() {
                                UNSET => Input::None,
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
