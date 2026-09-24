//! The `parts` prop on the date and time pickers: a calendar, a digital clock,
//! a duration and a date-time flow, whose calendar sits in a tab panel.

use dioxus::prelude::*;
use libero::chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeDelta};
use libero::components::{
    ChronoPicker, ChronoPickerPart, DatePicker, Flex, Parts, StaticParts, TimePicker,
};
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[("/picker-parts", || rsx! { PickerPartsPage {} })];

static SPACED: StaticParts<ChronoPickerPart> = StaticParts::new(|| {
    Parts::new()
        .part(ChronoPickerPart::Day, sx().letter_spacing("3px"))
        .part(ChronoPickerPart::Title, sx().font_style("italic"))
        .part(ChronoPickerPart::Value, sx().letter_spacing("4px"))
        .part(ChronoPickerPart::Unit, sx().font_style("italic"))
});

#[component]
fn PickerPartsPage() -> Element {
    let day = NaiveDate::from_ymd_opt(2026, 9, 14);
    rsx! {
        Flex { direction: "column", gap: "md", align: "flex-start",
            DatePicker { id: "date", value: day, parts: &SPACED }
            TimePicker { id: "time", value: NaiveTime::from_hms_opt(9, 30, 0), variant: "digital", parts: &SPACED }
            ChronoPicker::<TimeDelta> { id: "duration", value: TimeDelta::minutes(90), parts: &SPACED }
            ChronoPicker::<NaiveDateTime> { id: "flow", value: day.and_then(|day| day.and_hms_opt(9, 0, 0)), parts: &SPACED }
            DatePicker { id: "plain", value: day }
        }
    }
}
