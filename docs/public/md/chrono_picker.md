# ChronoPicker

Crate: `libero`
Import: `use libero::{chrono::{NaiveDate, NaiveTime}, components::{DateLevel, ChronoPicker, DateRange}};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/date/chrono_picker.rs>
Index: [index.md](index.md) lists every other page
Description: One picker for every date and time value, from days, months and years to times, date-times and ranges of them.

One picker for every date and time value. The value's type picks what it draws.
`NaiveDate` a month of days, `NaiveTime` a clock, `NaiveDateTime` the day and
then the time, a `DateRange` of either a start and an end, and a `TimeDelta` a
duration, one column per part. `level` turns a day picker into a month or a
year picker, and `calendar: "mini"` into one row of days.

It opens on the value's month, else today's. Names come from the localization's
`DateLocale`, and the first weekday and heading format from the provider's
`Formats`. As on [ChronoField](chrono_field.md), a typed handler or a turbofish
names the value type. For one value type there are `DatePicker`, `MonthPicker`,
`YearPicker`, `TimePicker` and `DateRangePicker`, with only the props that type
uses and no turbofish.

## Usage

```rust
use dioxus::prelude::*;
use libero::{chrono::NaiveDate, components::{DateLevel, ChronoPicker}};

#[component]
fn Demo() -> Element {
    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 9, 14));
    let mut month = use_signal(|| None::<NaiveDate>);

    rsx! {
        ChronoPicker {
            value: day(),
            onchange: move |next| day.set(next),
            min: NaiveDate::from_ymd_opt(2026, 9, 5),
            name: "arrival",
        }
        ChronoPicker {
            value: month(),
            onchange: move |next| month.set(next),
            level: DateLevel::Month,
        }
    }
}
```

As with `ChronoField`, a typed `value` alone does not name `V`. A handler that
stores into a typed signal does, and so does a turbofish such as
`ChronoPicker::<NaiveTime> { name: "at" }`.

The picker keeps the month it shows as its own state. It opens on `value`'s
month, else today's. Today comes from the platform clock after mount on the
web. A server render and native builds mark no day unless `today` is set.

## Props

### `ChronoPicker<V: DateValue>`

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
| `variant` | `TimePickerVariant` | `analog` | A digital clock, `HH:MM` with a column to turn per part, or a clock face, for values with a time. |
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

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work. Every picker takes `ChronoPickerPart` and draws only
the parts its value uses. A date field's dropdown is portaled, so the field's
`parts` do not reach the picker in it.

| Part | `data-slot` | Description |
|---|---|---|
| `ChronoPickerPart::Header` | `header` | Calendar: the row over a month, a year or a decade, with paging buttons and title. |
| `ChronoPickerPart::Nav` | `nav` | Calendar: a paging button, in the header or beside the mini calendar's row. |
| `ChronoPickerPart::Title` | `title` | Calendar: the month, year or decade heading, a button that climbs a level. |
| `ChronoPickerPart::Months` | `months` | Days: the months side by side, or the mini calendar's row. |
| `ChronoPickerPart::Weekday` | `weekday` | Days: a weekday name over a month's columns. |
| `ChronoPickerPart::Day` | `day` | Days: a day button. |
| `ChronoPickerPart::Month` | `month` | Mini calendar: the month over a day's number. |
| `ChronoPickerPart::Blank` | `blank` | Days, `columns` over 1: an empty cell instead of a neighbour's day. |
| `ChronoPickerPart::Cells` | `cells` | Months and years: their grid. |
| `ChronoPickerPart::Cell` | `cell` | Months and years: a month or a year button. |
| `ChronoPickerPart::Strip` | `strip` | Mini calendar: the row of days between its paging buttons. |
| `ChronoPickerPart::Columns` | `columns` | Digital clock, duration: the columns. |
| `ChronoPickerPart::Spin` | `spin` | Digital clock, duration: one column, a spinbutton. |
| `ChronoPickerPart::Value` | `value` | Digital clock, duration: a column's value. |
| `ChronoPickerPart::Neighbour` | `neighbour` | Digital clock, duration: the faded values above and below a column's value. |
| `ChronoPickerPart::Separator` | `separator` | Digital clock: the `:` between columns. |
| `ChronoPickerPart::Unit` | `unit` | Duration: the unit after each column. |
| `ChronoPickerPart::Readout` | `readout` | Analog clock: the digits over the face, buttons that pick the hand. |
| `ChronoPickerPart::Face` | `face` | Analog clock: the face, a slider. |
| `ChronoPickerPart::Mark` | `mark` | Analog clock: a number on the face. |
| `ChronoPickerPart::Ticks` | `ticks` | Analog clock: the ring of ticks for steps finer than the marks. |
| `ChronoPickerPart::Hand` | `hand` | Analog clock: the hand. |
| `ChronoPickerPart::Pivot` | `pivot` | Analog clock: the dot the hand turns on. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Left` or `Right` | Days: the day before or after. Months and years: the cell before or after. Mini calendar: a day, sliding the row one day past its ends. |
| `Up` or `Down` | Days: a week earlier or later. Months and years: a row up or down. |
| `Home` or `End` | Days: the first or last day of the week. Months, years and the mini calendar: the row's ends. |
| `PageUp` or `PageDown` | Days: a month. Months: a year. Years: a decade. Mini calendar: `days` days. |
| `Shift+PageUp` or `Shift+PageDown` | Days: a year. |
| `Enter` or `Space` | Picks the cell. |
| `Enter` | On the month heading: climbs to the months, focus on the year heading. On the year heading: climbs to the years, focus into them. |
| `Up` or `Right` | Analog clock: the hand forward by an hour or `step` minutes. Digital column (`Up`): a step forward. |
| `Down` or `Left` | Analog clock: the hand back. Digital column (`Down`): a step back. |
| `PageUp` or `PageDown` | Digital column: a bigger step. |
| `Home` or `End` | Digital column: the first or last value. |
| `Digit` | Digital column: picks. A filled column moves on to the next. |
| `Enter` | Analog clock: from the hour to the minute. Digital column: to the next column. |
| `Tab` | Analog clock: to the next control. Digital column: to the next column. |

### Libero handles

- Days, months and years are one tab stop each, on the picked cell, else today,
  else the first.
- Today's day, month and year carry `aria-current="date"`.
- The decade heading is disabled, since there is no level above it.
- Picking a month or a year below the lowest level climbs back down with focus
  on it.
- The mini calendar's days are one tab stop. Its two buttons page the row by
  `days`.
- The analog clock face is one tab stop. Each digital column is a spinbutton
  and one tab stop.
- The clock keys change the value at once and skip what `min` and `max` rule
  out.
- The wheel and a drag turn a digital column too, and a press on the value
  above or below picks it.
- A date-time's day and time are tabs above the picker, and picking the day
  moves focus into the clock.
- A date-time range has three tabs: the days, the start time and the end time.
  Each shows its value once picked, such as `12–14 Oct` or `09:00`, and moves on
  to the next when complete. The start and the end may be on different days.

## Theme defaults

`theme.chrono_picker` and `theme.time_picker` hold the size steps, the default
calendar and its `days`, and the default clock variant and minute `step`. The
month and weekday names and the button labels come from `Localization::date`, a
`DateLocale` (see [localization](localization.md)). The provider's `formats`,
`Formats::AMERICAN` by default or `Formats::GERMAN`, hold the first weekday and
the date and time patterns.
