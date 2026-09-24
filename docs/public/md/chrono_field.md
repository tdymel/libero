# ChronoField

Crate: `libero`
Import: `use libero::{chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeDelta}, components::{ChronoField, DateRange}};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/date/chrono_field.rs>
Index: [index.md](index.md) lists every other page
Description: A text field for every date and time value, typed leniently, with the matching `ChronoPicker` in a dropdown.

A text field for every date and time value, with a [ChronoPicker](chrono_picker.md)
in a dropdown. The value's type picks what the dropdown shows: `NaiveDate` a
calendar, `NaiveTime` a clock, `NaiveDateTime` both, a `DateRange` of either
picks two, and `TimeDelta` is a duration. `level` makes a `NaiveDate` field a
month or a year field, typed as `September 2026` or `2026`. The types are
`chrono`'s, re-exported as `libero::chrono`.

Typed text is read on blur or Enter, and leniently. Any separator works, as do
one-digit days, month names as a unique prefix, and a missing year. Two-digit
years are not read. Only the order of day, month and year follows `format`.
Text the field cannot accept stays, and the error says why. The form always
gets ISO 8601.

The language of names, labels and errors comes from the provider's
`Localization`. The patterns, first weekday and 12- or 24-hour clock come from
its `Formats`, `Formats::AMERICAN` by default or `Formats::GERMAN`.

A typed value alone does not name the type. A handler that stores into a typed
signal does, or a turbofish such as `ChronoField::<NaiveTime> { .. }`. For one
value type there are `DateField`, `TimeField`, `DateTimeField`, `DateRangeField`
and `DateTimeRangeField`, with only the props that type uses and no turbofish.

## Usage

```rust
use dioxus::prelude::*;
use libero::{chrono::NaiveDate, components::ChronoField};

#[component]
fn Demo() -> Element {
    let mut day = use_signal(|| None::<NaiveDate>);

    rsx! {
        ChronoField {
            label: "Arrival",
            value: day(),
            onchange: move |next| day.set(next),
            format: "DD.MM.YYYY",
            name: "arrival",
        }
    }
}
```

A month field:

```rust
use dioxus::prelude::*;
use libero::{chrono::NaiveDate, components::{ChronoField, DateLevel}};

#[component]
fn Demo() -> Element {
    let mut month = use_signal(|| None::<NaiveDate>);

    rsx! {
        ChronoField {
            label: "Billing month",
            level: DateLevel::Month,
            value: month(),
            onchange: move |next| month.set(next),
        }
    }
}
```

## Duration

A `TimeDelta` field holds a span of time rather than a moment. It shows
`1 h 30 min`, with the units from `DateLocale`, and posts ISO 8601, `PT1H30M`.
Typing reads `1 h 30 min`, `1h30`, `1:30` or a bare number of minutes.

The dropdown has a column each for the hours, the minutes at `step` and, with
`with_seconds`, the seconds. The minutes and seconds wrap round without
carrying into the next column. The value runs from `min`, 0 by default, to
`max`, 99 h 59 min 59 s by default. The hours column ends at `max`.

A duration has its own errors: `Must be at least 15 min` names the bound it
missed, and `Not a valid duration` is text it cannot read. A screen reader
hears each column's value with its unit, `2 hours`.

```rust
use dioxus::prelude::*;
use libero::{chrono::TimeDelta, components::ChronoField};

#[component]
fn Demo() -> Element {
    let mut length = use_signal(|| TimeDelta::try_minutes(90));

    rsx! {
        ChronoField {
            label: "Length",
            value: length(),
            onchange: move |next| length.set(next),
            max: TimeDelta::try_hours(12),
            step: 15,
            name: "length",
        }
    }
}
```

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `FieldPart::Label` | `label` | The label above the control. |
| `FieldPart::Required` | `required` | The required asterisk, in the label. |
| `FieldPart::Description` | `description` | The caption between the label and the control. |
| `FieldPart::Frame` | `frame` | The bordered box around the control. |
| `FieldPart::Control` | `control` | The element the label names. |
| `FieldPart::Trailing` | `trailing` | The slot after the control: a chevron, a toggle. |
| `FieldPart::Helper` | `helper` | The caption under the control. |
| `FieldPart::Status` | `status` | The validation message. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Down` | Moves focus into the picker, onto the picked day or the clock, where the [ChronoPicker](chrono_picker.md#accessibility) keys apply. |
| `Escape` | Moves focus back to the text. |

### Libero handles

- Focus opens the dropdown and stays in the text, so you can type at once.
- A pick that closes the dropdown moves focus back to the text.
- Focus leaving both the text and the dropdown closes it.
- A mouse click in the dropdown leaves focus in the text.

## Props

### `ChronoField<V: DateValue>`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Control height, font size and the dropdown's picker. |
| `radius` | `Size` | `sm` | Corner radius of the frame. |
| `value` | `Option<V>` | - | The value. `None` is the empty field. Its type picks the dropdown. Pair it with `onchange`. |
| `onchange` | `EventHandler<Option<V>>` | - | Called on every pick, and when typed text is committed on blur or Enter. Emptied text commits `None`. |
| `level` | `DateLevel` | `Day` | Types and picks a `NaiveDate` as a day, a month (its first day) or a year (its January 1). Ignored for other values. |
| `format` | `String` | `(Formats::date)(level)` | How the text shows the value, in dayjs tokens. The default is `MMMM D, YYYY` in American formats, `D. MMMM YYYY` in German, and `MMMM YYYY` or `YYYY` in both for a month or a year. Typing only has to match the order of day, month and year. |
| `time_format` | `String` | `Formats::time` | How the text shows a time. `h:mm A` in American formats, `HH:mm` in German. |
| `min` | `V::Bound` | - | The earliest value accepted. For a range, the earliest end. A duration's is 0 when unset. |
| `max` | `V::Bound` | - | The latest value accepted. For a range, the latest end. A duration's is 99 h 59 min 59 s when unset. |
| `exclude_date` | `Callback<NaiveDate, bool>` | - | Days that are not accepted, on top of `min` and `max`. Ignored for a time, a month and a year. |
| `today` | `NaiveDate` | - | The day marked as today, and the year used when typed text has none. Unset, the platform clock answers after mount. |
| `variant` | `TimePickerVariant` | `analog` | A digital clock, `HH:MM` with a column to turn per part, or a clock face, for values with a time. |
| `with_seconds` | `bool` | `false` | Seconds in the text and on the clock, or a duration's seconds column. |
| `step` | `u8` | `5` | Minutes between the offered minutes, on a clock or a duration's minutes column. |
| `twelve_hour` | `bool` | - | A 12-hour clock with AM and PM. Defaults to whether the time format is one. |
| `calendar` | `CalendarVariant` | `full` | A month of days, or `mini`, one row of days with buttons that page it. For a day or a date-time. |
| `days` | `usize` | `7` | Days in the mini calendar's row. |
| `columns` | `usize` | `1, or 2 for a range` | Months side by side. |
| `close_on_change` | `bool` | `true` | Picking a day, or a range's second end, closes the dropdown. |
| `name` | `FieldName<Option<V>>` | - | What the field posts as, the value in ISO 8601 whatever the text shows. A path also binds it to the surrounding `Form`'s value when it has no `onchange`. |
| `validate` | `Validators<Option<V>>` | - | Rules over the value, shown once the field loses focus or its form is submitted. |
| `placeholder` | `String` | - | Shown while the text is empty. |
| `label` | `Caption` | - | The caption above the control, and the field's name. |
| `description` | `Caption` | - | Between the label and the control. What to enter. |
| `helper` | `Caption` | - | Under the control. Formatting rules, or what the entry changes. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. Text the field cannot accept shows its own error instead, such as `DateLocale::invalid_date`, `invalid_duration` or the bound it missed. |
| `required` | `bool` | `false` | Sets `required` on the input and marks the label. |
| `disabled` | `bool` | `false` | Disables typing and the dropdown, and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the field from the tab order and the post instead. |

`format` takes the dayjs tokens `YYYY M MM MMM MMMM D DD dd ddd dddd` and
`[literal]` text. Props that only some value types use are ignored by the rest,
with a warning in debug builds.

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes. The attributes land on the input.

## Theme defaults

`theme.chrono_field` is a `ChronoFieldDefaults` with `size`, `radius` and
`close_on_change`. Names, labels and errors come from `Localization::date`, a
`DateLocale` (see [localization](localization.md)). Its weekday arrays are
Sunday first. The date and time patterns, the first weekday and the range
separator come from the provider's `formats`, `Formats::AMERICAN` by default
(Sunday first, a 12-hour clock) or `Formats::GERMAN` (Monday first, a 24-hour
clock, `14. September 2026`). Any language goes with any formats. This site is
English in German formats.
