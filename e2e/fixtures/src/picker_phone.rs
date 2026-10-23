//! The picker fields below `sm`: the phone sheet (todo 3900), the text-input switch (3905),
//! the clock face (3926), the color sheet (3923), the short range text (3904), the scrolling
//! range (3903), the swiped day (3924), the swiped-away sheet (3957) and a range's explicit
//! `columns` cut to the months that fit (3958).

use dioxus::prelude::*;
use libero::{
    chrono::{NaiveDate, NaiveTime},
    components::{
        ChronoField, ColorCode, ColorField, DateRange, Flex, Input, SliderChangeEvent, Text,
    },
    theme::TimePickerVariant,
};

use crate::Routes;

pub const ROUTES: Routes = &[("/picker-phone", || rsx! { PickerPhonePage {} })];

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

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
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
