# DatePicker

Crate: `libero`
Import: `use libero::{chrono::{NaiveDate, NaiveTime}, components::{DateLevel, DatePicker, DateRange}};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/date/date_picker.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: One picker for every date and time value, from days, months and years to times, date-times and ranges of them.

One picker for every date and time value. The value's type picks what it draws,
and `level` turns a day picker into a month or a year picker.
[DateField](date_field.md) shows it in a dropdown.

For one value type each there is a picker with only the props that type uses,
and no turbofish.

- `DayPicker`, a `NaiveDate` day.
- `MonthPicker`, a month, as `level: DateLevel::Month`.
- `YearPicker`, a year, as `level: DateLevel::Year`.
- `TimePicker`, a `NaiveTime`, digital or analog.
- `DateRangePicker`, a `DateRange<NaiveDate>`.

## Value types

`DatePicker<V: DateValue>` holds an `Option<V>`. The type picks what is drawn.

| `V` | Draws | `min` / `max` |
|---|---|---|
| `NaiveDate` | a month of days, or with `level` the months of a year or a decade of years | `NaiveDate` |
| `NaiveTime` | an analog clock face, or digital columns | `NaiveTime` |
| `NaiveDateTime` | the day, then the time, with a switch between them | `NaiveDateTime` |
| `DateRange<NaiveDate>` | two months, and the second pick ends the range | `NaiveDate` |
| `DateRange<NaiveDateTime>` | start then end, each a day and a time | `NaiveDateTime` |

With `calendar: "mini"` a day or a date-time is picked from one row of
`days` days instead of a month. Each day shows its
month over its number. The row starts at the value, else today.

At `DateLevel::Month` the value is the month's first day, and at
`DateLevel::Year` the year's January 1. A range whose `end` is `None` waits for its second
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
stores into a typed signal does, and so does a turbofish such as
`DatePicker::<NaiveTime> { name: "at" }`.

The picker keeps the month it shows as its own state. It opens on `value`'s
month, else today's. Today comes from the platform clock after mount on the
web. A server render and native builds mark no day unless `today` is set.

## Accessibility

Days, months and years are one tab stop each, on the picked cell, else today,
else the first. Today's day, month and year carry `aria-current="date"`.

| Key | Days | Months and years |
|---|---|---|
| Arrow Left / Right | the day before / after | the cell before / after |
| Arrow Up / Down | a week earlier / later | a row up / down |
| Home / End | the first / last day of the week | the row's ends |
| Page Up / Page Down | a month, or a year with Shift | a year / a decade |
| Enter / Space | picks | picks |

Enter on the month heading climbs to the months and keeps focus on the year
heading. Enter on the year heading climbs to the years and moves focus into the
years. The decade heading is disabled, since there is no level above it.
Picking a month or a year below the lowest level climbs back down with focus on
it.

The mini calendar's days are one tab stop too. Arrow Left / Right move a day
and slide the row one day past its ends, Home / End go to the row's ends, and
Page Up / Page Down move `days` days. The two buttons page the row by `days`.

The clock keys:

| Key | Analog face (one tab stop) | Digital column (one tab stop each) |
|---|---|---|
| Arrow Up / Right | the hand forward by an hour or `step` minutes | Up: the option above |
| Arrow Down / Left | the hand back | Down: the option below |
| Home / End | - | the first / last option |
| Enter | from the hour to the minute | picks |
| Tab | to the next control | to the next column |

The arrows on the face change the value at once and skip what `min` and `max`
rule out. A date-time picks the day first, and picking it moves focus into the
clock.

## Props

### `DatePicker<V: DateValue>`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Option<V>` | - | The picked value. Its type picks what the picker draws. Pair it with `onchange`. |
| `onchange` | `EventHandler<Option<V>>` | - | Called with the value to hold next. |
| `level` | `DateLevel` | `Day` | Picks a `NaiveDate` as a day, a month (its first day) or a year (its January 1). Ignored for other values. |
| `min` | `V::Bound` | - | The earliest value that can be picked. For a range, the earliest end. |
| `max` | `V::Bound` | - | The latest value that can be picked. For a range, the latest end. |
| `exclude_date` | `Callback<NaiveDate, bool>` | - | Days that cannot be picked, on top of `min` and `max`. |
| `allow_deselect` | `bool` | `false` | Clicking the picked day again clears it. Only for a day. |
| `columns` | `usize` | `1, or 2 for a range` | Months side by side, for a day or a range of days. |
| `calendar` | `CalendarVariant` | `full` | A month of days, or `mini`, one row of days with buttons that page it. For a day or a date-time. |
| `days` | `usize` | `7` | Days in the mini calendar's row. |
| `variant` | `TimePickerVariant` | `analog` | Columns of numbers or a clock face, for values with a time. |
| `with_seconds` | `bool` | `false` | A seconds column. Digital only. |
| `step` | `u8` | `5` | Minutes between the offered minutes. |
| `twelve_hour` | `bool` | `formats` | A 12-hour clock with AM and PM. Defaults to whether `Formats::time` is one. |
| `today` | `NaiveDate` | - | The day marked as today. Unset, the platform clock answers after mount on the web. Elsewhere no day is marked. |
| `size` | `Size` | `md` | Cell, option and font size. |
| `name` | `String` | - | Posts the value as ISO 8601 in a hidden input of that name. |
| `focusable` | `bool` | `true` | `false` keeps the picker out of the tab order, for a picker inside a dropdown whose input keeps focus. |

Props that only some value types use are ignored by the rest, with a warning in
debug builds.

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes. The attributes land on the root.

## Theme defaults

`theme.date_picker` and `theme.time_picker` hold the size steps, the default
calendar and its `days`, and the default clock variant and minute `step`. The
month and weekday names and the button labels come from `Localization::date`, a
`DateLocale` (see [localization](localization.md)). The provider's `formats`,
`Formats::AMERICAN` by default or `Formats::GERMAN`, hold the first weekday and
the date and time patterns.
