//! `RadioGroup`, for the `RadioSet` archetype.

use dioxus::prelude::*;
use libero::components::{Flex, Options, RadioGroup};

use crate::{Routes, common::Between};

pub const ROUTES: Routes = &[("/radio-group", || rsx! { RadioGroupPage {} })];

#[derive(Clone, Copy, PartialEq, Options)]
enum Plan {
    Free,
    Pro,
    Team,
}

/// Starts on the **second** option, so "Tab enters at the checked radio" can
/// fail: with the first one checked it is indistinguishable from "Tab enters at
/// the first radio".
#[component]
fn RadioGroupPage() -> Element {
    let mut plan = use_signal(|| Some(Plan::Pro));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Between {
                RadioGroup {
                    label: "Plan",
                    value: plan(),
                    onchange: move |next| plan.set(Some(next)),
                }
            }
        }
    }
}
