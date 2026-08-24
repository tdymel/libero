use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Flex, Switch, Text};

#[component]
pub fn SwitchPage() -> Element {
    let mut notifications = use_signal(|| true);

    rsx! {
        DocPage {
            title: "Switch",
            source: "libero/src/components/inputs/switch",
            markdown: "/md/switch.md",
            properties: vec![props("Switch", vec![
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("Track color when checked; a theme color name or a literal CSS color."),
                prop("size", "Size").default("md").doc("Controls track, thumb, and label size."),
                prop("radius", "Size")
                    .default("xl")
                    .doc("Track corner radius; the thumb is always a circle."),
                prop("checked", "bool")
                    .doc("Strictly controlled - pair it with `onchange`."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables interaction and dims the switch."),
                prop("onchange", "EventHandler<bool>")
                    .doc("Called with the value `checked` should take next."),
                prop("aria_label", "String")
                    .doc("Names the switch when it has no `children`; `attributes` cannot, since they land on the root, not the input."),
                prop("children", "Element")
                    .doc("Text and `Icon` only - a `<label>` hijacks clicks on nested controls."),
            ])],
            lead: rsx! {
                Text {
                    "A checkbox styled as a track and thumb. A visually hidden "
                    Code { source: "input" }
                    " does the real work, so it is announced as a switch and Space toggles it."
                }
            },
            Demo {
                component: "Switch",
                children_text: "Notifications",
                controls: vec![
                    Control::color(
                        "color",
                        ["primary", "secondary", "success", "error", "warning", "info"],
                    ),
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
                    Switch {
                        color: values.str("color"),
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
                        "Notifications"
                    }
                },
            }
            DocSection {
                title: "Controlled",
                Text {
                    "Strictly controlled: "
                    Code { source: "checked" }
                    " drives the look, "
                    Code { source: "onchange" }
                    " reports the value it should take next. The click is cancelled, so the "
                    "switch only moves when its state does."
                }
                Flex {
                    direction: "row",
                    gap: "md",
                    align: "center",
                    Switch {
                        checked: notifications(),
                        onchange: move |next| notifications.set(next),
                        "Notifications"
                    }
                }
                Text { "Notifications: {notifications()}" }
            }
            DocSection {
                title: "Without a label",
                Text {
                    "Give it an "
                    Code { source: "aria_label" }
                    " - "
                    Code { source: "attributes" }
                    " land on the root, not on the input that carries the role."
                }
                Switch { aria_label: "Airplane mode", checked: true, onchange: move |_| {} }
            }
        }
    }
}
