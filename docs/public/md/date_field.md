# DateField

Crate: `libero`
Import: `use libero::{chrono::{NaiveDate, NaiveDateTime, NaiveTime}, components::{DateField, DateRange}};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/date/date_field.rs>
Index: [index.md](index.md) lists every other page
Description: A text field for every date and time value, typed leniently, with the matching `DatePicker` in a dropdown.

A text field for every date and time value, with a [DatePicker](date_picker.md)
of the same value type in a dropdown. `level` makes a `NaiveDate` field a month
or a year field. The types are `chrono`'s, re-exported as `libero::chrono`.

Typed text is read on blur or Enter, and leniently. Only the order of day, month
and year follows `format`. Text the field cannot accept stays, and the error
says why. The form always gets ISO 8601.

For one value type each there is a field with only the props that type uses,
and no turbofish.

- `DayField`, a `NaiveDate`, with `DayPicker` in the dropdown.
- `TimeField`, a `NaiveTime`, with `TimePicker`.
- `DateTimeField`, a `NaiveDateTime`, the day and then the time.
- `DateRangeField`, a `DateRange<NaiveDate>`, with `DateRangePicker`.
- `DateTimeRangeField`, a `DateRange<NaiveDateTime>`, the start and then the end.

## Usage

```rust
use dioxus::prelude::*;
use libero::{chrono::NaiveDate, components::DateField};

#[component]
fn Demo() -> Element {
    let mut day = use_signal(|| None::<NaiveDate>);

    rsx! {
        DateField {
            label: "Arrival",
            value: day(),
            onchange: move |next| day.set(next),
            format: "DD.MM.YYYY",
            name: "arrival",
        }
    }
}
```

A typed `value` alone does not name `V`, because dioxus converts every prop. A
handler that stores into a typed signal names it, as above. Otherwise use a
turbofish, `DateField::<NaiveTime> { name: "alarm" }`. Without either, the
compiler reports E0283 "type annotations needed".

## Value types

`DateField<V: DateValue>` holds an `Option<V>`. The type picks the dropdown:

| `V` | Dropdown | `min` / `max` |
|---|---|---|
| `NaiveDate` | a month of days | `NaiveDate` |
| `NaiveTime` | a clock | `NaiveTime` |
| `NaiveDateTime` | the day, then the time | `NaiveDateTime` |
| `DateRange<NaiveDate>` | two months, start then end | `NaiveDate` |
| `DateRange<NaiveDateTime>` | start then end, each a day and a time | `NaiveDateTime` |

`DateValue` is sealed. The `chrono` types are re-exported as `libero::chrono`.

## Months and years

`level: DateLevel::Month` makes a `NaiveDate` field a month field. The text
reads `MMMM YYYY` unless `format` says otherwise, the value is the month's first
day, and the dropdown opens on the month grid. `DateLevel::Year` reads `YYYY`
and holds January 1. `min` and `max` accept any month or year they reach. A
month without a year takes the current year, and a year field reads only a
four-digit year.

```rust
use dioxus::prelude::*;
use libero::{chrono::NaiveDate, components::{DateField, DateLevel}};

#[component]
fn Demo() -> Element {
    let mut month = use_signal(|| None::<NaiveDate>);

    rsx! {
        DateField {
            label: "Billing month",
            level: DateLevel::Month,
            value: month(),
            onchange: move |next| month.set(next),
        }
    }
}
```

## Typing

Text stays as typed until the field blurs or Enter is pressed. Then it is read
leniently. For a day, only the order of day, month and year has to match
`format`.

| Input | Read as, for `DD.MM.YYYY` |
|---|---|
| `1/2/2026`, `01-02-2026`, `1. 2. 2026` | February 1, 2026 |
| `01022026` | February 1, 2026 (digits alone, at full width) |
| `1 feb 2026`, `1 FEBR 2026` | February 1, 2026 (a name, or a prefix naming one month) |
| `1.2` | February 1 of the current year |
| `1.2.26` | rejected, two-digit years are not read |
| `31.2.2026` | rejected, no such day |

Literal text from the format is skipped, so `2026年3月4日` reads for
`YYYY年M月D日` and `14 h 05` for `HH[ h ]mm`. Accents are not folded, so
`fevrier` is not `février`.

Times read `13:05`, `1305`, `1:05 pm` and `9`. A date-time is a day, then a
time. A range splits on `–`, `—`, ` - ` or ` to `.

Emptied text commits `None`. Text that is not a value the field accepts
stays, and the field says why, in words from the `DateLocale`.

| Text | Error (English) |
|---|---|
| unreadable | `invalid_date`: Not a valid date |
| before `min`, no `max` | `on_or_after`: Must be on or after March 5, 2026 |
| after `max`, no `min` | `on_or_before`: Must be on or before March 9, 2026 |
| outside `min` and `max` | `between`: Must be between March 5, 2026 and March 9, 2026 |
| a day `exclude_date` refuses | `unavailable`: That date is not available |

The bounds are shown in the field's own format, so a month field says
"March 2026".

## Accessibility

The input is a `combobox` whose dropdown is a non-modal `dialog`, named by
`DateLocale::date_label` (`time_label` for a time). Focus opens the dropdown
and stays in the text input, so you can type at once.

| Key | In the text input | In the dropdown |
|---|---|---|
| Arrow Down | moves focus into the picker, onto the picked day, else today, or the clock | the picker's own keys ([DatePicker](date_picker.md#accessibility)) |
| Enter | commits the typed text | picks. A pick that closes returns focus to the text |
| Escape | closes the dropdown | closes it and returns focus to the text |

Focus leaving both the text and the dropdown closes it. A mouse click in the
dropdown leaves focus in the text.

## Posting

The text input carries no `name`. A hidden input does, holding ISO 8601
whatever the text shows, such as `2026-02-01`, `13:05:00`, `2026-02-01T13:05:00`, and
`start/end` for a range.

## Props

### `DateField<V: DateValue>`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Control height, font size and the dropdown's picker. |
| `radius` | `Size` | `sm` | Corner radius of the frame. |
| `value` | `Option<V>` | - | The value. `None` is the empty field. Its type picks the dropdown. Pair it with `onchange`. |
| `onchange` | `EventHandler<Option<V>>` | - | Called on every pick, and when typed text is committed on blur or Enter. Emptied text commits `None`. |
| `level` | `DateLevel` | `Day` | Types and picks a `NaiveDate` as a day, a month (its first day) or a year (its January 1). Ignored for other values. |
| `format` | `String` | `(Formats::date)(level)` | How the text shows the value, in dayjs tokens. The default is `MMMM D, YYYY` in American formats, `D. MMMM YYYY` in German, and `MMMM YYYY` or `YYYY` in both for a month or a year. Typing only has to match the order of day, month and year. |
| `time_format` | `String` | `Formats::time` | How the text shows a time. `h:mm A` in American formats, `HH:mm` in German. |
| `min` | `V::Bound` | - | The earliest value accepted. For a range, the earliest end. |
| `max` | `V::Bound` | - | The latest value accepted. For a range, the latest end. |
| `exclude_date` | `Callback<NaiveDate, bool>` | - | Days that are not accepted, on top of `min` and `max`. Ignored for a time, a month and a year. |
| `today` | `NaiveDate` | - | The day marked as today, and the year used when typed text has none. Unset, the platform clock answers after mount. |
| `variant` | `TimePickerVariant` | `analog` | A digital clock, `HH:MM` with a column to turn per part, or a clock face, for values with a time. |
| `with_seconds` | `bool` | `false` | Seconds in the text and on the clock. |
| `step` | `u8` | `5` | Minutes between the offered minutes. |
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
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. Text the field cannot accept shows its own error instead, such as `DateLocale::invalid_date` or the bound it missed. |
| `required` | `bool` | `false` | Sets `required` on the input and marks the label. |
| `disabled` | `bool` | `false` | Disables typing and the dropdown, and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the field from the tab order and the post instead. |

`format` takes the dayjs tokens `YYYY M MM MMM MMMM D DD dd ddd dddd` and
`[literal]` text. Props that only some value types use are ignored by the rest,
with a warning in debug builds.

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes. The attributes land on the input.

## Theme defaults

`theme.date_field` is a `DateFieldDefaults` with `size`, `radius` and
`close_on_change`. Names, labels and errors come from `Localization::date`, a
`DateLocale` (see [localization](localization.md)). Its weekday arrays are
Sunday first. The date and time patterns, the first weekday and the range
separator come from the provider's `formats`, `Formats::AMERICAN` by default
(Sunday first, a 12-hour clock) or `Formats::GERMAN` (Monday first, a 24-hour
clock, `14. September 2026`). Any language goes with any formats. This site is
English in German formats.
