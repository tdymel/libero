use crate::components::{
    Child, Control, Demo, DemoValues, DocPage, UNSET, Wrap, a11y, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Box, Divider, DividerPart, Input, Text},
    sx::sx,
};

/// The label is a child, not a prop, so a control has to decide it here.
fn label_child(values: &DemoValues) -> String {
    match values.str("with_label").as_str() {
        "true" => "OR".to_string(),
        _ => String::new(),
    }
}

/// A printed wrapper gives the rule an axis to span and neighbours for `spacing`'s margin.
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
                    .doc("The direction of the rule."),
                prop("size", "Size").default("xs").doc("Line thickness."),
                prop("label_position", "LabelPosition")
                    .default("center")
                    .doc("Where the label sits along the rule."),
                prop("spacing", "ThemeAwareValue")
                    .default("none")
                    .doc("Margin on both sides of the rule, a spacing step or a CSS length."),
                prop("color", "ThemeAwareValue")
                    .default("muted.4")
                    .doc("Line color. A bare theme color like `primary` resolves to its shade 3."),
                prop("children", "Element")
                    .doc("An optional label in the line."),
                prop("parts", "Parts<DividerPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`."),
            ])
            .parts("DividerPart", vec![
                (DividerPart::Label, "The label between the two line halves, with `children` only."),
            ])],
            accessibility: a11y()
                .handles(["The rule is a `separator`, named by its label. Your own `aria-label` or `aria-labelledby` wins."])
                .must(["Pass `role: \"none\"` for a purely visual rule."]),
            lead: rsx! {
                Text { "A horizontal or vertical rule, with an optional label sitting in the line." }
            },
            Demo {
                component: "Divider",
                children_text: "OR",
                controls: vec![
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"]),
                    Control::sizes("size")
                        .default("xs"),
                    Control::switch("with_label").default("true").code(|_, _| vec![]),
                    // Meaningless without a label.
                    Control::toggle("label_position", ["start", "center", "end"])
                        .labels(["Start", "Center", "End"])
                        .default("center")
                        .hidden_when(|values| values.str("with_label") != "true"),
                    // Unset draws muted.4, which bare `muted` (shade 3) is not: the unset swatch is painted muted.4.
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
