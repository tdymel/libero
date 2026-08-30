# YearPicker

Crate: `libero`
Import: `use libero::{chrono::NaiveDate, components::YearPicker};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/date/pickers.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: The years of a decade to pick one from - a DatePicker's year view. The value is the year's January 1.

Part of the date and time components: values are `chrono`'s
`NaiveDate`, `NaiveTime` and `NaiveDateTime` - no time zone, re-exported as
`libero::chrono` - and libero's `DateRange<T>`. Names, formats and labels come from
the theme's `DateDefaults`. Typing, posting and the shared field props work as
[DateField](date_field.md) describes; pickers work as
[DatePicker](date_picker.md) describes.

The live docs page lists every prop.
