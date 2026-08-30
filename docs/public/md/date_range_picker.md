# DateRangePicker

Crate: `libero`
Import: `use libero::{chrono::NaiveDate, components::{DateRange, DateRangePicker}};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/date/pickers.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Months side by side to pick a start and an end from, with the hovered day previewing the range.

Part of the date and time components: values are `chrono`'s
`NaiveDate`, `NaiveTime` and `NaiveDateTime` - no time zone, re-exported as
`libero::chrono` - and libero's `DateRange<T>`. Names, formats and labels come from
the theme's `DateDefaults`. Typing, posting and the shared field props work as
[DateField](date_field.md) describes; pickers work as
[DatePicker](date_picker.md) describes.

The live docs page lists every prop.
