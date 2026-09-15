//! `SegmentedControl`, for the `RadioSet` archetype.

use dioxus::prelude::*;
use libero::components::{Fields, Flex, Form, OptionList, Options, SegmentedControl};

use crate::{Routes, common::Between};

pub const ROUTES: Routes = &[
    ("/segmented-control", || rsx! { SegmentedControlPage {} }),
    ("/segmented-control/disabled-pick", || {
        rsx! { DisabledSegmentPage { start: Alignment::Center } }
    }),
    ("/segmented-control/disabled-middle", || {
        rsx! { DisabledSegmentPage { start: Alignment::Left } }
    }),
    (
        "/segmented-control/readonly",
        || rsx! { SegmentedControlPage { readonly: true } },
    ),
    ("/segmented-control/field", || rsx! { FieldPage {} }),
    ("/segmented-control/long", || rsx! { LongPage {} }),
    ("/segmented-control/form", || rsx! { FormPage {} }),
];

#[derive(Clone, PartialEq, Default, Fields)]
struct Layout {
    alignment: Alignment,
}

/// Todo 508: a bound strip in a `Form` with a submit button, counting submits.
#[component]
fn FormPage() -> Element {
    let value = use_store(Layout::default);
    let mut submits = use_signal(|| 0u32);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            span { id: "submits", "data-submits": "{submits}", "Submits {submits}" }
            Form { value, onsubmit: move |_| submits += 1,
                SegmentedControl { label: "Alignment", name: Layout::FIELDS.alignment() }
                button { r#type: "submit", "Save" }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Options)]
enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// Seven segments, far wider than a phone.
#[component]
fn LongPage() -> Element {
    let mut day = use_signal(|| Weekday::Wednesday);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            SegmentedControl {
                label: "Day",
                value: day(),
                onchange: move |next| day.set(next),
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default, Options)]
enum Alignment {
    Left,
    #[default]
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

/// A vertical, full-width strip with every caption and an error.
#[component]
fn FieldPage() -> Element {
    let mut alignment = use_signal(|| Alignment::Center);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Between {
                SegmentedControl {
                    label: "Alignment",
                    description: "Where each line starts.",
                    helper: "Applies to the whole document.",
                    status: "Pick an alignment.",
                    required: true,
                    orientation: "vertical",
                    full_width: true,
                    value: alignment(),
                    onchange: move |next| alignment.set(next),
                }
            }
        }
    }
}

/// `Center` disabled; `start: Center` is a pick that was disabled later.
#[component]
fn DisabledSegmentPage(start: Alignment) -> Element {
    let mut alignment = use_signal(move || start);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Between {
                SegmentedControl {
                    label: "Alignment",
                    options: OptionList::from_options().disabling(|align| *align == Alignment::Center),
                    value: alignment(),
                    onchange: move |next| alignment.set(next),
                }
            }
        }
    }
}
