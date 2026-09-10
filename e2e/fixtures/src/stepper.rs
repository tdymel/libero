//! `Stepper`, each arm moved on by a button inside the current step's content.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Options, Stepper};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/stepper", || rsx! { StepperPage { vertical: false } }),
    (
        "/stepper-vertical",
        || rsx! { StepperPage { vertical: true } },
    ),
];

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
