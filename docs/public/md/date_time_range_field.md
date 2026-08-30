# DateTimeRangeField

Crate: `libero`
Import: `use libero::{chrono::NaiveDateTime, components::{DateRange, DateTimeRangeField}};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/date/fields.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A text field holding a DateRange of DateTimes; the dropdown picks the start's day and time before the end's.

Part of the date and time components: values are `chrono`'s
`NaiveDate`, `NaiveTime` and `NaiveDateTime` - no time zone, re-exported as
`libero::chrono` - and libero's `DateRange<T>`. Names, formats and labels come from
the theme's `DateDefaults`. Typing, posting and the shared field props work as
[DateField](date_field.md) describes; pickers work as
[DatePicker](date_picker.md) describes.

The live docs page lists every prop.
