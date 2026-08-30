# DateFieldPrototype

Crate: `libero`
Import: `use libero::{chrono::{NaiveDate, NaiveDateTime, NaiveTime}, components::{DateFieldPrototype, DateRange}};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/date/prototype.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A prototype of one date field for every date and time value; the value's type picks the dropdown.

`DateFieldPrototype<V: DateValue>` holds an `Option<V>` where `V` is
`NaiveDate` (calendar), `NaiveTime` (clock), `NaiveDateTime` (calendar, then
clock), `DateRange<NaiveDate>` (two months) or `DateRange<NaiveDateTime>`
(start and end, each a day and a time). `min` and `max` take `V::Bound`: the
value's own type, or a range's end type. `DateValue` is sealed.

It stands in for [DateField](date_field.md), [TimeField](time_field.md),
[DateTimeField](date_time_field.md), [DateRangeField](date_range_field.md) and
[DateTimeRangeField](date_time_range_field.md), over the same engine: typing,
posting and the shared field props work as those pages describe.

The live docs page lists every prop.
