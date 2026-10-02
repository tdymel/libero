//! `Stepper`, each arm moved on by a button inside the current step's content.

use dioxus::prelude::*;
use libero::components::{Button, Flex, OptionLabel, Options, Stepper};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/stepper", || rsx! { StepperPage { vertical: false } }),
    (
        "/stepper-vertical",
        || rsx! { StepperPage { vertical: true } },
    ),
    ("/stepper-long", || rsx! { LongLabelPage {} }),
    ("/stepper-named", || rsx! { NamedPage {} }),
    ("/stepper-many", || rsx! { ManyStepsPage {} }),
];

#[derive(Clone, Copy, PartialEq, Options)]
enum Phase {
    Cart,
    Account,
    Shipping,
    Billing,
    Payment,
    Review,
    Confirm,
    Receipt,
    Survey,
}

/// Todo 1573: five and nine steps in both horizontal arms, for reflow at 390 and 320px.
#[component]
fn ManyStepsPage() -> Element {
    let all = Phase::options();
    rsx! {
        Flex { direction: "column", gap: "md",
            for (id, count, position) in [
                ("side-5", 5, "side"),
                ("below-5", 5, "below"),
                ("side-9", 9, "side"),
                ("below-9", 9, "below"),
            ] {
                Stepper {
                    key: "{id}",
                    id,
                    value: Some(Phase::Account),
                    options: all[..count].to_vec(),
                    label_position: position,
                    onstepclick: |_| {},
                }
            }
        }
    }
}

/// Todo 541: one step list named by `aria_label`, one by a heading.
#[component]
fn NamedPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md",
            Stepper { id: "labelled", aria_label: "Checkout", value: Some(Stage::Shipping) }
            h2 { id: "heading", "Onboarding" }
            Stepper { id: "labelledby", aria_labelledby: "heading", value: Some(Stage::Shipping) }
        }
    }
}

/// One label with no break opportunity, in every arm, for reflow at a narrow width.
#[component]
fn LongLabelPage() -> Element {
    let label = |stage: Stage| match stage {
        Stage::Shipping => OptionLabel::from("Versandkostenberechnungsgrundlagenverordnung"),
        other => OptionLabel::from(other.label()),
    };
    rsx! {
        Flex { direction: "column", gap: "md",
            for (id, orientation, position) in [
                ("side", "horizontal", "side"),
                ("below", "horizontal", "below"),
                ("vertical", "vertical", "side"),
            ] {
                Stepper {
                    key: "{id}",
                    id,
                    value: Some(Stage::Shipping),
                    orientation,
                    label_position: position,
                    option_label: label,
                    panel: |_: Stage| rsx! { "Body." },
                }
            }
            // Ordinary labels with a description. `side` needs ~360px for
            // three steps, so it breaks words at 320px; `below` must not.
            Stepper {
                id: "plain",
                value: Some(Stage::Shipping),
                label_position: "below",
                option_description: |stage: Stage| format!("About the {}", stage.label()),
                panel: |_: Stage| rsx! { "Body." },
            }
            // The same in `side`, in a 320px box: the container query stacks it.
            div { width: "320px",
                Stepper {
                    id: "boxed",
                    value: Some(Stage::Shipping),
                    option_description: |stage: Stage| format!("About the {}", stage.label()),
                    panel: |_: Stage| rsx! { "Body." },
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Options)]
enum Stage {
    Account,
    Shipping,
    Review,
}

/// The last step's button finishes, so `value` becomes `None`.
#[component]
fn StepperPage(vertical: bool) -> Element {
    let mut stage = use_signal(|| Some(Stage::Account));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "560px",
            Stepper {
                id: "stepper",
                value: stage(),
                orientation: if vertical { "vertical" } else { "horizontal" },
                label_position: "below",
                onstepclick: move |s| stage.set(Some(s)),
                panel: move |s: Stage| match s {
                    Stage::Account => rsx! {
                        Button { id: "next-0", onclick: move |_| stage.set(Some(Stage::Shipping)), "Continue to shipping" }
                    },
                    Stage::Shipping => rsx! {
                        Button { id: "next-1", onclick: move |_| stage.set(Some(Stage::Review)), "Continue to review" }
                    },
                    Stage::Review => rsx! {
                        Button { id: "finish", onclick: move |_| stage.set(None), "Finish" }
                    },
                },
            }
        }
    }
}
