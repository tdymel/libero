use crate::components::{Control, Demo, DemoValues, DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Chip, Code, Flex, Text};

#[component]
pub fn ChipPage() -> Element {
    let mut selected = use_signal(|| vec!["rust".to_string()]);

    rsx! {
        DocPage {
            title: "Chip",
            lead: rsx! {
                Text {
                    "A compact token. With "
                    Code { source: "onchange" }
                    " it is a real checkbox - a visually hidden "
                    Code { source: "input" }
                    " plus a "
                    Code { source: "label" }
                    ", so selection is announced and Space toggles it. Without it, a plain tag."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Chip",
                    children_text: "rust",
                    controls: vec![
                        Control::color(
                            "color",
                            ["primary", "secondary", "success", "error", "warning", "info"],
                        ),
                        Control::toggle("variant", ["outlined", "filled", "text"])
                            .labels(["Outlined", "Filled", "Text"]),
                        Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("md"),
                        Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("xl"),
                        // Controlled state is `checked` + `onchange`; the
                        // library warns about one without the other.
                        Control::switch("checked").code(|_, values| {
                            match values.str("checked").as_str() {
                                "true" => vec![
                                    "checked: true".to_string(),
                                    "onchange: move |_| {}".to_string(),
                                ],
                                _ => vec![],
                            }
                        }),
                        Control::switch("disabled"),
                    ],
                    render: move |values: DemoValues| rsx! {
                        Chip {
                            color: values.str("color"),
                            variant: values.str("variant"),
                            size: values.str("size"),
                            radius: values.str("radius"),
                            // Both or neither: `checked` alone can never
                            // change, `onchange` alone can never look selected.
                            checked: match values.str("checked").as_str() {
                                "true" => Some(true),
                                _ => None,
                            },
                            onchange: (values.str("checked") == "true")
                                .then(|| EventHandler::new(move |_: bool| {})),
                            disabled: values.str("disabled") == "true",
                            "rust"
                        }
                    },
                }
            }
            DocSection {
                title: "Selectable",
                Text {
                    "Strictly controlled: "
                    Code { source: "checked" }
                    " drives the look, "
                    Code { source: "onchange" }
                    " reports the value it should take next. A checked chip is filled "
                    "whatever its "
                    Code { source: "variant" }
                    ", so "
                    Code { source: "variant" }
                    " describes the unselected state."
                }
                Flex {
                    direction: "row",
                    gap: "md",
                    for language in ["rust", "css", "html"] {
                        Chip {
                            key: "{language}",
                            checked: selected().iter().any(|s| s == language),
                            onchange: move |next: bool| {
                                selected
                                    .with_mut(|selected| {
                                        if next {
                                            selected.push(language.to_string());
                                        } else {
                                            selected.retain(|s| s != language);
                                        }
                                    });
                            },
                            "{language}"
                        }
                    }
                }
                Text { "Selected: {selected():?}" }
            }
        }
    }
}
