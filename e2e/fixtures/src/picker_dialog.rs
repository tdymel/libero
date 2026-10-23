//! Fields whose dropdown is a `role="dialog"`: `ColorField` and `ChronoField`.
//! One field per page, so `[role=dialog]` is unambiguous.

use dioxus::prelude::*;
use libero::{
    chrono::NaiveDate,
    components::{ChronoField, ColorCode, ColorField, DateRange, Flex, SliderChangeEvent},
    localization::Formats,
    use_formats_handle,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/color-field", || rsx! { ColorFieldPage {} }),
    ("/chrono-field", || rsx! { ChronoFieldPage {} }),
    ("/date-range-field", || rsx! { DateRangeFieldPage {} }),
    ("/color-field/tab-out", || rsx! { ColorTabOutPage {} }),
    ("/chrono-field/tab-out", || rsx! { ChronoTabOutPage {} }),
];

/// A control after the field, for Tab past the dropdown's last stop (todo 4042).
#[component]
fn ColorTabOutPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            ColorField { id: "color-field", label: "Accent", swatches: vec!["#fa5252", "#40c057"] }
            button { id: "after", "After" }
        }
    }
}

#[component]
fn ChronoTabOutPage() -> Element {
    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 9, 25));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            ChronoField {
                id: "chrono-field",
                label: "Arrival",
                value: day(),
                onchange: move |next| day.set(next),
            }
            button { id: "after", "After" }
        }
    }
}

/// A range separator none of the built-in fallbacks match.
static WAVE_DASH: Formats = Formats {
    range_separator: " ～ ",
    ..Formats::AMERICAN
};

#[component]
fn ColorFieldPage() -> Element {
    let mut color = use_signal(|| "#1c7ed6".parse::<ColorCode>().unwrap());

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            ColorField {
                id: "color-field",
                label: "Accent",
                value: color(),
                swatches: vec!["#fa5252", "#40c057", "#228be6"],
                oninput: move |event: SliderChangeEvent<ColorCode>| {
                    if let SliderChangeEvent::Change(next) = event {
                        color.set(next);
                    }
                },
            }
        }
    }
}

/// `today` pinned outside the held month, so the baseline snapshot never
/// depends on the clock (todo 658).
#[component]
fn ChronoFieldPage() -> Element {
    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 9, 25));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            ChronoField {
                id: "chrono-field",
                label: "Arrival",
                today: NaiveDate::from_ymd_opt(2026, 3, 18),
                value: day(),
                onchange: move |next| day.set(next),
            }
        }
    }
}

#[component]
fn DateRangeFieldPage() -> Element {
    let formats = use_formats_handle();
    use_effect(move || formats.set(&WAVE_DASH));
    let mut stay = use_signal(|| None::<DateRange<NaiveDate>>);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            ChronoField {
                id: "date-range-field",
                label: "Stay",
                name: "stay",
                value: stay(),
                onchange: move |next| stay.set(next),
            }
        }
    }
}
