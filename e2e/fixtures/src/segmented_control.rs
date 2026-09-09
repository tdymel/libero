//! `SegmentedControl`, for the `RadioSet` archetype.

use dioxus::prelude::*;
use libero::components::{Flex, Options, SegmentedControl};

use crate::{Routes, common::Between};

pub const ROUTES: Routes = &[("/segmented-control", || rsx! { SegmentedControlPage {} })];

#[derive(Clone, Copy, PartialEq, Options)]
enum Alignment {
    Left,
    Center,
    Right,
}

/// Starts on the middle segment, for the same reason as `RadioGroupPage`.
///
/// `readonly` exists for the planted defect only: a read-only strip refuses the
/// arrows, which is exactly "the arrows do nothing" as a keyboard user meets
/// it.
#[component]
pub fn SegmentedControlPage(#[props(default)] readonly: bool) -> Element {
    let mut alignment = use_signal(|| Alignment::Center);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Between {
                SegmentedControl {
                    label: "Alignment",
                    readonly,
                    value: alignment(),
                    onchange: move |next| alignment.set(next),
                }
            }
        }
    }
}
