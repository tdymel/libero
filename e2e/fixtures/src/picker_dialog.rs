//! Fields whose dropdown is a `role="dialog"`: `ColorField` and `DateField`.
//! One field per page, so `[role=dialog]` is unambiguous.

use dioxus::prelude::*;
use libero::{
    chrono::NaiveDate,
    components::{ColorCode, ColorField, DateField, DateRange, Flex, SliderChangeEvent},
    hooks::use_theme_set,
    theme::{DateDefaults, Theme, ThemeSet},
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/color-field", || rsx! { ColorFieldPage {} }),
    ("/date-field", || rsx! { DateFieldPage {} }),
    ("/date-range-field", || rsx! { DateRangeFieldPage {} }),
];

/// A range separator none of the built-in fallbacks match.
static WAVE_DASH: Theme = Theme {
    date: DateDefaults {
        range_separator: " ～ ",
        ..DateDefaults::ENGLISH
    },
    ..Theme::DEFAULT
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

#[component]
fn DateFieldPage() -> Element {
    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 9, 25));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            DateField {
                id: "date-field",
                label: "Arrival",
                value: day(),
                onchange: move |next| day.set(next),
            }
        }
    }
}

#[component]
fn DateRangeFieldPage() -> Element {
    let themes = use_theme_set();
    use_effect(move || themes.set(ThemeSet::from(&WAVE_DASH)));
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
