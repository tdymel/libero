//! The picker fields below `sm`: the phone sheet (todo 3900), the text-input switch (3905),
//! the clock face (3926), the color sheet (3923), the short range text (3904), the scrolling
//! range (3903), the swiped day (3924), the swiped-away sheet (3957) and a range's explicit
//! `columns` cut to the months that fit (3958), the outside tap that only closes (4826) and, on
//! its own page, the mini row and presets over week numbers (4825).

use dioxus::prelude::*;
use libero::{
    chrono::{NaiveDate, NaiveTime},
    components::{
        ChronoField, ColorCode, ColorField, DatePreset, DateRange, Flex, Input, SliderChangeEvent,
        Text,
    },
    theme::{CalendarVariant, TimePickerVariant},
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/picker-phone", || rsx! { PickerPhonePage {} }),
    ("/picker-phone/grid", || rsx! { PickerPhoneGridPage {} }),
];

const SWATCHES: [&str; 3] = ["#fa5252", "#40c057", "#228be6"];

/// A range of days, a day, a clock, a digital clock and a color, values echoed.
#[component]
fn PickerPhonePage() -> Element {
    let day = |day| NaiveDate::from_ymd_opt(2026, 9, day);
    let mut stay = use_signal(move || day(14).map(|start| DateRange::new(start, day(16))));
    let mut pair = use_signal(move || day(14).map(|start| DateRange::new(start, day(16))));
    let mut due = use_signal(move || day(14));
    let mut at = use_signal(|| NaiveTime::from_hms_opt(9, 30, 0));
    let mut digital = use_signal(|| NaiveTime::from_hms_opt(9, 30, 0));
    let mut color = use_signal(|| "#228be6".parse::<ColorCode>().unwrap());
    let mut taps = use_signal(|| 0u32);

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            // Counts the clicks an outside tap lets through (4826).
            button { id: "tally", onclick: move |_| taps += 1, "Tapped {taps}" }
            div { id: "stay",
                ChronoField::<DateRange<NaiveDate>> {
                    label: "Stay",
                    today: day(9),
                    value: stay(),
                    onchange: move |next| stay.set(next),
                }
            }
            div { id: "pair",
                ChronoField::<DateRange<NaiveDate>> {
                    label: "Pair",
                    today: day(9),
                    columns: 2usize,
                    value: pair(),
                    onchange: move |next| pair.set(next),
                }
            }
            div { id: "due",
                ChronoField::<NaiveDate> {
                    label: "Due",
                    today: day(9),
                    value: due(),
                    onchange: move |next| due.set(next),
                }
            }
            div { id: "at",
                ChronoField::<NaiveTime> { label: "At", value: at(), onchange: move |next| at.set(next) }
            }
            div { id: "digital",
                ChronoField::<NaiveTime> {
                    label: "Digital",
                    variant: Input::Value(TimePickerVariant::Digital),
                    value: digital(),
                    onchange: move |next| digital.set(next),
                }
            }
            div { id: "color",
                ColorField {
                    label: "Accent",
                    swatches: SWATCHES,
                    value: color(),
                    oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
                }
            }
            Text { id: "echo", "{stay:?} {due:?} {at:?} {color:?}" }
        }
    }
}

/// A mini row and a month with week numbers under wrapping presets; the first page would scroll.
#[component]
fn PickerPhoneGridPage() -> Element {
    let day = |day| NaiveDate::from_ymd_opt(2026, 9, day);
    let mut mini = use_signal(move || day(14));
    let mut weeks = use_signal(move || day(14));

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            div { id: "mini",
                ChronoField::<NaiveDate> {
                    label: "Mini",
                    today: day(9),
                    calendar: Input::Value(CalendarVariant::Mini),
                    value: mini(),
                    onchange: move |next| mini.set(next),
                }
            }
            div { id: "weeks",
                ChronoField::<NaiveDate> {
                    label: "Weeks",
                    today: day(9),
                    week_numbers: true,
                    presets: vec![
                        DatePreset::new("Start of month", day(1).unwrap()),
                        DatePreset::new("End of month", day(30).unwrap()),
                        // Wraps the row, so the presets span the sheet.
                        DatePreset::new("Next month", NaiveDate::from_ymd_opt(2026, 10, 1).unwrap()),
                    ],
                    value: weeks(),
                    onchange: move |next| weeks.set(next),
                }
            }
        }
    }
}
