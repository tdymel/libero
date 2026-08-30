# DatePicker

Crate: `libero`
Import: `use libero::{chrono::{NaiveDate, NaiveTime}, components::{DateLevel, DatePicker, DateRange}};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/date/date_picker.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: One picker for every date and time value - days, months, years, times, date-times and ranges of them.

Modelled on Mantine's `DatePicker`, `MonthPicker`, `YearPicker` and
`TimePicker`, in one component. [DateField](date_field.md) shows it in a
dropdown.

## Value types

`DatePicker<V: DateValue>` holds an `Option<V>`. The type picks what is drawn:

| `V` | Draws | `min` / `max` |
|---|---|---|
| `NaiveDate` | a month of days; with `level`, the months of a year or a decade of years | `NaiveDate` |
| `NaiveTime` | an analog clock face, or digital columns | `NaiveTime` |
| `NaiveDateTime` | the day, then the time, with a switch between them | `NaiveDateTime` |
| `DateRange<NaiveDate>` | two months; the second pick ends the range | `NaiveDate` |
| `DateRange<NaiveDateTime>` | start then end, each a day and a time | `NaiveDateTime` |

At `DateLevel::Month` the value is the month's first day; at `DateLevel::Year`
it is the year's January 1. A range whose `end` is `None` waits for its second
pick, and the days up to the hovered one preview it.

## Usage

```rust
use dioxus::prelude::*;
use libero::{chrono::NaiveDate, components::{DateLevel, DatePicker}};

#[component]
fn Demo() -> Element {
    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 9, 14));
    let mut month = use_signal(|| None::<NaiveDate>);

    rsx! {
        DatePicker {
            value: day(),
            onchange: move |next| day.set(next),
            min: NaiveDate::from_ymd_opt(2026, 9, 5),
            name: "arrival",
        }
        DatePicker {
            value: month(),
            onchange: move |next| month.set(next),
            level: DateLevel::Month,
        }
    }
}
```

As with `DateField`, a typed `value` alone does not name `V`. A handler that
stores into a typed signal does, and so does a turbofish:
`DatePicker::<NaiveTime> { name: "at" }`.

## The month shown

The picker keeps the month it shows as its own state. It opens on `value`'s
month, else today's. Today comes from the platform clock after mount - on the
web; a server render and native builds have no clock, so no day is marked
unless `today` is set.

## Keyboard

Days, months and years are one tab stop each: the picked cell, else today,
else the first.

| Key | Days | Months and years |
|---|---|---|
| Arrow Left / Right | the day before / after | the cell before / after |
| Arrow Up / Down | a week earlier / later | a row up / down |
| Home / End | the first / last day of the week | the row's ends |
| Page Up / Page Down | a month; with Shift a year | a year / a decade |
| Enter / Space | picks | picks |

Enter on a heading climbs a level and keeps focus on the new heading; picking
a month or a year below the lowest level climbs back down with focus on it.

The clock:

| Key | Analog face (one tab stop) | Digital column (one tab stop each) |
|---|---|---|
| Arrow Up / Right | the hand forward: an hour, or `step` minutes | Up: the option above |
| Arrow Down / Left | the hand back | Down: the option below |
| Home / End | - | the first / last option |
| Enter | from the hour to the minute | picks |
| Tab | to the next control | to the next column |

The arrows on the face change the value at once and skip what `min` and `max`
rule out. A date-time picks the day first; picking it moves focus into the
clock.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Option<V>` | - | Strictly controlled. |
| `onchange` | `EventHandler<Option<V>>` | - | Called with the value the caller should hold next. |
| `level` | `DateLevel` | `Day` | A `NaiveDate` as a day, a month or a year. |
| `min` / `max` | `V::Bound` | - | Limits for picking. |
| `exclude_date` | `Callback<NaiveDate, bool>` | - | Days that cannot be picked. |
| `allow_deselect` | `bool` | `false` | Clicking the picked day again clears it. |
| `columns` | `usize` | `1`, `2` for a range | Months side by side. |
| `variant` | `TimePickerVariant` | `analog` | Columns or a clock face, for values with a time. |
| `with_seconds` | `bool` | `false` | A seconds column. Digital only. |
| `step` | `u8` | `1` | Minutes between offered minutes. |
| `twelve_hour` | `bool` | theme | A 12-hour clock. |
| `today` | `NaiveDate` | clock | The day marked as today. |
| `size` | `Size` | `md` | Cell, option and font size. |
| `name` | `String` | - | Emits a hidden input posting ISO 8601. |
| `focusable` | `bool` | `true` | `false` keeps the picker out of the tab order. |

Props that only some value types use are ignored by the rest.

## Alternatives

The same picker for one value type each, with only the props that type uses
and no turbofish.

- `DayPicker` - a `NaiveDate` day.
- `MonthPicker` - a month, as `level: DateLevel::Month`.
- `YearPicker` - a year, as `level: DateLevel::Year`.
- `TimePicker` - a `NaiveTime`, digital or analog.
- `DateRangePicker` - a `DateRange<NaiveDate>`.

## Theme

`Theme::date` (`DateDefaults`) holds the month and weekday names, the first
weekday, the formats and the button labels - one place to translate.
`Theme::date_picker` and `Theme::time_picker` hold the size steps and the
default clock variant.
