use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::components::{Chip, Code, Flex, Text};

#[component]
pub fn ChipPage() -> Element {
    let mut selected = use_signal(|| vec!["rust".to_string()]);
    let mut clicks = use_signal(|| 0);

    rsx! {
        DocPage {
            title: "Chip",
            source: "libero/src/components/inputs/chip",
            markdown: "/md/chip.md",
            properties: vec![props("Chip", vec![
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("Accent color; a theme color name or a literal CSS color."),
                prop("variant", "ButtonVariant")
                    .default("filled")
                    .doc("The unselected look; a checked chip is a tonal container whatever its variant."),
                prop("size", "Size").default("md").doc("Controls height, padding, and font size."),
                prop("radius", "Size")
                    .default("xl")
                    .doc("Corner radius, independent of size."),
                prop("checked", "bool")
                    .doc("Strictly controlled selection state - pair it with `onchange`."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables interaction and dims the chip."),
                prop("onchange", "EventHandler<bool>")
                    .doc("Called with the value `checked` should take next. Its presence makes the chip a real checkbox."),
                prop("onclick", "EventHandler<MouseEvent>")
                    .doc("A plain action; its presence makes the chip a `button`."),
                prop("to", "NavigationTarget")
                    .doc("Renders a router-aware link instead. Takes precedence over `onclick`."),
                prop("target", "String")
                    .doc("Link target, e.g. `_blank`. Only with `to`."),
                prop("children", "Element")
                    .doc("Text and `Icon` only - a `<label>` hijacks clicks on nested controls."),
            ])],
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
            Demo {
                component: "Chip",
                children_text: "rust",
                controls: vec![
                    Control::color("color"),
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "text"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Text"]),
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
            DocSection {
                title: "Selectable",
                Text {
                    "Strictly controlled: "
                    Code { source: "checked" }
                    " drives the look, "
                    Code { source: "onchange" }
                    " reports the value it should take next. A checked chip is a tinted "
                    "container - Material 3's selected filter chip - whatever its "
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
            DocSection {
                title: "Actions and links",
                Text {
                    Code { source: "onclick" }
                    " makes the chip a "
                    Code { source: "button" }
                    ", "
                    Code { source: "to" }
                    " a router-aware link. Neither combines with "
                    Code { source: "onchange" }
                    "."
                }
                Flex {
                    direction: "row",
                    gap: "md",
                    Chip {
                        onclick: move |_| clicks += 1,
                        "Clicked {clicks}x"
                    }
                    Chip {
                        to: "https://dioxuslabs.com",
                        target: "_blank",
                        "Dioxus"
                    }
                }
            }
        }
    }
}
