use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{
    Button, Code, Flex, Input, Options, StepState, Stepper, StepperPart, Text,
};

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

/// Each step's own button moves on and removes the content it sits in, the
/// case the component hands focus back for.
// snippet: after STAGE_ENUM
// snippet: in Stepper { value: stage(), .. }
const CONTENT: &str = r#"panel: move |s: Stage| match s {
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

// snippet: after STAGE_ENUM
// snippet: in Stepper { value: stage(), panel: |_: Stage| rsx! {}, .. }
const DESCRIPTION: &str = r#"option_description: |s: Stage| match s {
    Stage::Account => "Who you are".to_string(),
    Stage::Shipping => "Where it goes".to_string(),
    Stage::Review => "Check and pay".to_string(),
}"#;

// snippet: after STAGE_ENUM
// snippet: in Stepper { value: stage(), panel: |_: Stage| rsx! {}, .. }
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
                    prop("value", "Option<T>").default("required").doc("The current step. `None` means every step is finished."),
                    prop("panel", "Callback<T, Element>").doc("A step's body. Horizontal shows it below the strip, vertical under its own step. A closed step's content is not mounted."),
                    prop("options", "Vec<T>").default("T::options()").doc("The steps to show, in order."),
                    prop("option_label", "Callback<T, OptionLabel>")
                        .default("T::label()")
                        .doc("Overrides a step's label. `OptionLabel::rich` draws it as rsx and keeps a text name."),
                    prop("option_description", "Callback<T, String>").doc("A second line under a step's label. An empty string prints none."),
                    prop("state", "Callback<T, Option<StepState>>")
                        .default("derived")
                        .doc("Overrides a step's state. `None` keeps the derived one. The only way to mark a step `Error`."),
                    prop("onstepclick", "EventHandler<T>").doc("Called with the picked step. Without it the steps are plain text with no tab stops."),
                    prop("allow_next_steps", "bool").default("false").doc("With `onstepclick`, lets steps not reached yet be picked too."),
                    prop("orientation", "Orientation").default("horizontal").doc("`vertical` puts each step's content under the step itself."),
                    prop("label_position", "StepLabelPosition").default("side").doc("`side` or `below` the marker. Ignored when vertical. Under 120px a step (360px for three), `side` draws as `below`."),
                    prop("size", "Size").default("md").doc("Marker, type and spacing."),
                    prop("color", "ThemeAwareValue").default("primary").doc("The current and completed markers, and the connectors behind them."),
                    prop("parts", "Parts<StepperPart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                ])
                .parts("StepperPart", vec![
                    (StepperPart::List, "The `<ol>` of steps."),
                    (StepperPart::Step, "One step's `<li>`. Its `data-state` names the step's state."),
                    (StepperPart::Header, "The step's button, or a plain box when steps cannot be picked."),
                    (StepperPart::Marker, "The number, check or cross."),
                    (StepperPart::Body, "Label and description."),
                    (StepperPart::Label, "The step's label."),
                    (StepperPart::Description, "The second line under the label."),
                    (StepperPart::Panel, "The `panel` content: below the strip, or under each step when vertical."),
                ]),
            ],
            accessibility: a11y()
                .handles([
                    "With `onstepclick`, each clickable step is a button and a tab stop. Enter and Space activate. There are no arrow keys.",
                    "`aria_label` and `aria_labelledby` land on the step list, not the root.",
                    "A horizontal strip never widens the page (WCAG 1.4.10): the connectors shrink first, then steps that still don't fit scroll inside the strip. Labels keep their words whole; only a word wider than the strip breaks.",
                ])
                .must([
                    "Name the steps with `aria_label` or `aria_labelledby`.",
                    "Give a rich label a name that contains its visible text.",
                ])
                .example("A checkout, `Stepper { aria_label: \"Checkout steps\", .. }` with Cart, Address and Payment: a screen reader reads the list as \"Checkout steps\", and with `onstepclick` each clickable step is a button and a tab stop."),
            lead: rsx! {
                Text {
                    "The stages of a process, one per variant of an enum, with the current "
                    "step's content. "
                    Code { source: "panel" }
                    " matches on the same enum, so a step without a body does not compile. You "
                    "own "
                    Code { source: "value" }
                    " and move it on, usually from a button inside the step."
                }
            },
            Demo {
                component: "Stepper",
                children_text: "",
                // Three `md` steps with side labels overflowed the side-by-side preview.
                wide_preview: true,
                wrap: Wrap(|_: &DemoValues, source: &str| format!("{STAGE_ENUM}{source}")),
                fixed: vec![
                    "value: stage()".to_string(),
                    "aria_label: \"Order steps\"".to_string(),
                    CONTENT.to_string(),
                ],
                controls: vec![
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"])
                        .default("horizontal"),
                    Control::toggle("label_position", ["side", "below"])
                        .labels(["Side", "Below"])
                        .default("side")
                        .hidden_when(|values| values.str("orientation") == "vertical"),
                    Control::sizes("size")
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
                    Control::switch("option_description").code(|_, values| {
                        if on(values, "option_description") {
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
                                value: stage(),
                                aria_label: "Order steps",
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
                                option_description: on(&values, "option_description")
                                    .then(|| Callback::new(description)),
                                state: on(&values, "error").then(|| Callback::new(errored)),
                                panel: content,
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
                title: "Step states",
                Text {
                    "Steps before "
                    Code { source: "value" }
                    " are completed, the rest pending. "
                    Code { source: "state" }
                    " overrides single steps and is the only way to mark one "
                    Code { source: "StepState::Error" }
                    ". An error changes the marker, not which step is current."
                }
            }
        }
    }
}
