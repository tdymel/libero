# MonthPicker

Crate: `libero`
Import: `use libero::{chrono::NaiveDate, components::MonthPicker};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/date/pickers.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: The months of a year to pick one from - a DatePicker's month view. The value is the month's first day.

Part of the date and time components: values are `chrono`'s
`NaiveDate`, `NaiveTime` and `NaiveDateTime` - no time zone, re-exported as
`libero::chrono` - and libero's `DateRange<T>`. Names, formats and labels come from
the theme's `DateDefaults`. Typing, posting and the shared field props work as
[DateField](date_field.md) describes; pickers work as
[DatePicker](date_picker.md) describes.

The live docs page lists every prop.
