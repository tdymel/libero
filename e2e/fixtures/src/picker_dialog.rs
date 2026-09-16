//! Fields whose dropdown is a `role="dialog"`: `ColorField` and `DateField`.
//! One field per page, so `[role=dialog]` is unambiguous.

use dioxus::prelude::*;
use libero::{
    chrono::NaiveDate,
    components::{ColorCode, ColorField, DateField, DateRange, Flex, SliderChangeEvent},
    localization::{DateLocale, Localization},
    use_localization_handle,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/color-field", || rsx! { ColorFieldPage {} }),
    ("/date-field", || rsx! { DateFieldPage {} }),
    ("/date-range-field", || rsx! { DateRangeFieldPage {} }),
];

/// A range separator none of the built-in fallbacks match.
static WAVE_DASH: Localization = Localization {
    date: DateLocale {
        range_separator: " ～ ",
        ..DateLocale::ENGLISH
    },
    ..Localization::ENGLISH
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
fn DateFieldPage() -> Element {
    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 9, 25));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            DateField {
                id: "date-field",
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
    let localization = use_localization_handle();
    use_effect(move || localization.set(&WAVE_DASH));
    let mut stay = use_signal(|| None::<DateRange<NaiveDate>>);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            DateField {
                id: "date-range-field",
                label: "Stay",
                name: "stay",
                value: stay(),
                onchange: move |next| stay.set(next),
            }
        }
    }
}
