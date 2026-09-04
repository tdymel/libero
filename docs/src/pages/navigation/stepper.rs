use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, prop, props};
use dioxus::prelude::*;
use libero::components::{Button, Code, Flex, Input, Options, StepState, Stepper, Text};

/// The steps are the enum, so the snippet is a lie without it.
const STAGE_ENUM: &str = r#"#[derive(Clone, PartialEq, Options)]
enum Stage {
    Account,
    #[option(label = "Shipping address")]
    Shipping,
    Review,
}

let mut stage = use_signal(|| Some(Stage::Account));

"#;

/// Each step's own button moves on, which removes the content it sits in -
/// the case the component hands focus back for.
const CONTENT: &str = r#"content: move |s: Stage| match s {
    Stage::Account => rsx! {
        Flex { gap: "sm", align: "flex-start",
            Text { "Who is ordering?" }
            Button { onclick: move |_| stage.set(Some(Stage::Shipping)), "Continue" }
        }
    },
    Stage::Shipping => rsx! {
        Flex { gap: "sm", align: "flex-start",
            Text { "Where should the parcel go?" }
            Button { onclick: move |_| stage.set(Some(Stage::Review)), "Continue" }
        }
    },
    Stage::Review => rsx! {
        Flex { gap: "sm", align: "flex-start",
            Text { "Check the order, then place it." }
            Button { onclick: move |_| stage.set(None), "Place order" }
        }
    },
}"#;

const DESCRIPTION: &str = r#"description: |s: Stage| match s {
    Stage::Account => "Who you are".to_string(),
    Stage::Shipping => "Where it goes".to_string(),
    Stage::Review => "Check and pay".to_string(),
}"#;

const ERROR: &str = r#"state: |s: Stage| (s == Stage::Shipping).then_some(StepState::Error)"#;

#[derive(Clone, PartialEq, Options)]
enum Stage {
    Account,
    #[option(label = "Shipping address")]
    Shipping,
    Review,
}

fn description(stage: Stage) -> String {
    match stage {
        Stage::Account => "Who you are".into(),
        Stage::Shipping => "Where it goes".into(),
        Stage::Review => "Check and pay".into(),
    }
}

fn errored(stage: Stage) -> Option<StepState> {
    (stage == Stage::Shipping).then_some(StepState::Error)
}

fn on(values: &DemoValues, name: &str) -> bool {
    values.str(name) == "true"
}

#[component]
pub fn StepperPage() -> Element {
    let mut stage = use_signal(|| Some(Stage::Account));

    rsx! {
        DocPage {
            title: "Stepper",
            source: "libero/src/components/navigation/stepper",
            markdown: "/md/stepper.md",
            properties: vec![
                props("Stepper", vec![
                    prop("active", "Option<T>").doc("The current step. `None` means every step is finished. Required and strictly controlled."),
                    prop("content", "Callback<T, Element>").doc("A step's body. Horizontal shows the current one below the strip; vertical shows it under its own step and collapses the rest. A closed step's content is not mounted."),
                    prop("steps", "Vec<T>").default("T::options()").doc("The steps to show, in order."),
                    prop("label", "Callback<T, OptionLabel>")
                        .default("T::label()")
                        .doc("Overrides what the derive named a step. `OptionLabel::rich` draws it as rsx and still names it."),
                    prop("description", "Callback<T, String>").doc("A second line under a step's label. An empty string prints none."),
                    prop("state", "Callback<T, Option<StepState>>")
                        .default("derived")
                        .doc("Overrides a step's derived state; `None` keeps it. The only way to mark a step `Error`."),
                    prop("onstepclick", "EventHandler<T>").doc("Called with a picked step. Without it the steps are not interactive: no buttons, no tab stops."),
                    prop("allow_next_steps", "bool").default("false").doc("With `onstepclick`, whether steps not reached yet can be picked too."),
                    prop("orientation", "Orientation").default("horizontal").doc("`vertical` puts each step's content under the step itself."),
                    prop("label_position", "StepLabelPosition").default("side").doc("`side` or `below` the marker. Ignored when vertical."),
                    prop("size", "Size").default("md").doc("Marker, type and spacing."),
                    prop("color", "ThemeAwareValue").default("primary").doc("The current and completed markers, and the connectors behind them."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "The stages of a process over an enum, with the current one's content. The "
                    "steps are the enum's variants and "
                    Code { source: "content" }
                    " is a match over the same type, so a step without a body is a compile error. "
                    Code { source: "active" }
                    " is strictly controlled - moving on is the caller's, usually from a button "
                    "inside the step - and "
                    Code { source: "None" }
                    " means every step is finished."
                }
                Text {
                    "A step's state comes from its position: before "
                    Code { source: "active" }
                    " is completed, "
                    Code { source: "active" }
                    " itself is current, the rest are pending. "
                    Code { source: "state" }
                    " only overrides, and returning "
                    Code { source: "None" }
                    " keeps what was derived - so a caller names just the step that differs. It "
                    "is the only way to say "
                    Code { source: "StepState::Error" }
                    ", which changes the marker and never which step is current."
                }
            },
            Demo {
                component: "Stepper",
                children_text: "",
                wrap: Wrap(|_: &DemoValues, source: &str| format!("{STAGE_ENUM}{source}")),
                fixed: vec!["active: stage()".to_string(), CONTENT.to_string()],
                controls: vec![
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .default("horizontal"),
                    Control::toggle("label_position", ["side", "below"])
                        .default("side")
                        .hidden_when(|values| values.str("orientation") == "vertical"),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::color("color"),
                    Control::switch("clickable").code(|_, values| {
                        if on(values, "clickable") {
                            vec!["onstepclick: move |s| stage.set(Some(s))".to_string()]
                        } else {
                            vec![]
                        }
                    }),
                    Control::switch("allow_next_steps")
                        .code(|_, values| {
                            if on(values, "allow_next_steps") {
                                vec!["allow_next_steps: true".to_string()]
                            } else {
                                vec![]
                            }
                        })
                        .hidden_when(|values| !on(values, "clickable")),
                    Control::switch("description").code(|_, values| {
                        if on(values, "description") {
                            vec![DESCRIPTION.to_string()]
                        } else {
                            vec![]
                        }
                    }),
                    Control::switch("error").code(|_, values| {
                        if on(values, "error") {
                            vec![ERROR.to_string()]
                        } else {
                            vec![]
                        }
                    }),
                ],
                render: move |values: DemoValues| {
                    let content = move |s: Stage| match s {
                        Stage::Account => rsx! {
                            Flex { gap: "sm", align: "flex-start",
                                Text { "Who is ordering?" }
                                Button {
                                    onclick: move |_| stage.set(Some(Stage::Shipping)),
                                    "Continue"
                                }
                            }
                        },
                        Stage::Shipping => rsx! {
                            Flex { gap: "sm", align: "flex-start",
                                Text { "Where should the parcel go?" }
                                Button {
                                    onclick: move |_| stage.set(Some(Stage::Review)),
                                    "Continue"
                                }
                            }
                        },
                        Stage::Review => rsx! {
                            Flex { gap: "sm", align: "flex-start",
                                Text { "Check the order, then place it." }
                                Button { onclick: move |_| stage.set(None), "Place order" }
                            }
                        },
                    };
                    rsx! {
                        div { style: "display: flex; flex-direction: column; align-items: flex-start; gap: 16px; width: 100%;",
                            Stepper {
                                active: stage(),
                                orientation: values.str("orientation"),
                                label_position: values.str("label_position"),
                                size: values.str("size"),
                                color: match values.str("color").as_str() {
                                    "primary" => Input::None,
                                    color => Input::from(color),
                                },
                                onstepclick: on(&values, "clickable")
                                    .then(|| EventHandler::new(move |s| stage.set(Some(s)))),
                                allow_next_steps: on(&values, "allow_next_steps"),
                                description: on(&values, "description")
                                    .then(|| Callback::new(description)),
                                state: on(&values, "error").then(|| Callback::new(errored)),
                                content,
                            }
                            // Not part of the snippet: once every step is
                            // done there is no content left to move back from.
                            if stage().is_none() {
                                Button {
                                    variant: "outlined",
                                    onclick: move |_| stage.set(Some(Stage::Account)),
                                    "Start over"
                                }
                            }
                        }
                    }
                },
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "With "
                    Code { source: "onstepclick" }
                    " each clickable step is a button and a tab stop, in document order; "
                    "Enter and Space activate. There are no arrow keys."
                }
            }
        }
    }
}
