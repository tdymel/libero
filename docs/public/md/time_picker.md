# TimePicker

Crate: `libero`
Import: `use libero::{chrono::NaiveTime, components::TimePicker};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/date/time_picker.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A time to pick - scrolling columns of hours, minutes and seconds, or an analog clock face that takes the hour, then the minute.

Part of the date and time components: values are `chrono`'s
`NaiveDate`, `NaiveTime` and `NaiveDateTime` - no time zone, re-exported as
`libero::chrono` - and libero's `DateRange<T>`. Names, formats and labels come from
the theme's `DateDefaults`. Typing, posting and the shared field props work as
[DateField](date_field.md) describes; pickers work as
[DatePicker](date_picker.md) describes.

The live docs page lists every prop.
