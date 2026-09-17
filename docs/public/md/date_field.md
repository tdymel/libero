# DateField

Crate: `libero`
Import: `use libero::{chrono::{NaiveDate, NaiveDateTime, NaiveTime}, components::{DateField, DateRange}};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/date/date_field.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: One text field for every date and time value, typed leniently, with the matching `DatePicker` in a dropdown.

Modelled on Mantine's `DateInput`, `TimeInput`, `DateTimePicker` and
`DatePickerInput`, in one component. The dropdown is a
[DatePicker](date_picker.md) of the same value type.

There is also the same field for one value type each, with only the props that type uses.
They need no turbofish, and a value of the wrong type is a plain type mismatch.

- `DayField` - a `NaiveDate`, with `DayPicker` in the dropdown.
- `TimeField` - a `NaiveTime`, with `TimePicker`.
- `DateTimeField` - a `NaiveDateTime`: the day, then the time.
- `DateRangeField` - a `DateRange<NaiveDate>`, with `DateRangePicker`.
- `DateTimeRangeField` - a `DateRange<NaiveDateTime>`: the start, then the end.

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

`level: DateLevel::Month` makes a `NaiveDate` field a month field: the text
reads `(Formats::date)(DateLevel::Month)` (`MMMM YYYY`), the value is the month's
first day, and the dropdown opens on the month grid. `DateLevel::Year` reads
`YYYY` and holds January 1. The `format` prop still overrides
the text format. `min` and `max` accept a month or year they reach, as the grid
does. A month without a year takes the current year; a year field reads only a
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

A typed `value` alone does not name `V`: dioxus converts every prop, so
`Some(day)` could become more than one type. A handler that stores into a typed
signal names it, as above. Otherwise use a turbofish:
`DateField::<NaiveTime> { name: "alarm" }`. Without either, the compiler
reports E0283 "type annotations needed".

## Typing

Text stays exactly as typed until the field blurs or Enter is pressed. Then it
is read leniently - for a day, only the order of day, month and year has to
match `format`:

| Input | Read as, for `DD.MM.YYYY` |
|---|---|
| `1/2/2026`, `01-02-2026`, `1. 2. 2026` | February 1, 2026 |
| `01022026` | February 1, 2026 (digits alone, at full width) |
| `1 feb 2026`, `1 FEBR 2026` | February 1, 2026 (a name, or a prefix naming one month) |
| `1.2` | February 1 of the current year |
| `1.2.26` | rejected - two-digit years are not read |
| `31.2.2026` | rejected - no such day |

A word the format writes as literal text is skipped: `2026年3月4日` reads for
`YYYY年M月D日`, `14 h 05` for `HH[ h ]mm`. Accents are not folded: `fevrier`
is not `février`.

Times read `13:05`, `1305`, `1:05 pm` and `9`. A date-time is a day, then a
time. A range splits on `–`, `—`, ` - ` or ` to `.

Emptied text commits `None`. Text that is not a value the field accepts
stays, and the field says why, from the `DateLocale`:

| Text | Error (English) |
|---|---|
| unreadable | `invalid_date`: Not a valid date |
| before `min`, no `max` | `on_or_after`: Must be on or after March 5, 2026 |
| after `max`, no `min` | `on_or_before`: Must be on or before March 9, 2026 |
| outside `min` and `max` | `between`: Must be between March 5, 2026 and March 9, 2026 |
| a day `exclude_date` refuses | `unavailable`: That date is not available |

The bounds are shown in the field's own format: a month field says
"March 2026".

## Accessibility

The input is a `combobox` whose dropdown is a non-modal `dialog`, named by
`DateLocale::date_label` (`time_label` for a time). Focus opens the dropdown
and stays in the text input, so typing works at once.

| Key | In the text input | In the dropdown |
|---|---|---|
| Arrow Down | moves focus into the picker: the picked day, else today, or the clock | the picker's own keys ([DatePicker](date_picker.md#accessibility)) |
| Enter | commits the typed text | picks; a pick that closes returns focus to the text |
| Escape | closes the dropdown | closes it and returns focus to the text |

Focus leaving both the text and the dropdown closes it. A mouse click in the
dropdown leaves focus in the text.

## Posting

The text input carries no `name`. A hidden input does, holding ISO 8601
whatever the text shows: `2026-02-01`, `13:05:00`, `2026-02-01T13:05:00`, and
`start/end` for a range.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Option<V>` | - | Strictly controlled. |
| `onchange` | `EventHandler<Option<V>>` | - | On commit (blur, Enter) and on every pick. |
| `level` | `DateLevel` | `Day` | A `NaiveDate` as a day, a month (its first day) or a year (its January 1). |
| `format` | `String` | `(Formats::date)(level)` | dayjs tokens: `YYYY M MM MMM MMMM D DD dd ddd dddd`, `[literal]`. American: `MMMM D, YYYY`, German: `D. MMMM YYYY`; `MMMM YYYY`, `YYYY` in both. |
| `time_format` | `String` | `Formats::time` | How a time shows: `h:mm A` American, `HH:mm` German. |
| `min` / `max` | `V::Bound` | - | Limits for picking and typing. |
| `exclude_date` | `Callback<NaiveDate, bool>` | - | Days that cannot be picked or typed. Ignored for a time. |
| `today` | `NaiveDate` | clock | Marked day, and the year a yearless text takes. |
| `variant` | `TimePickerVariant` | `analog` | The clock, for values with a time. |
| `with_seconds` | `bool` | `false` | Seconds, for values with a time. |
| `step` | `u8` | `5` | Minutes between offered minutes. Theme: `TimePickerDefaults::step`. |
| `twelve_hour` | `bool` | from the time format | A 12-hour clock. |
| `calendar` | `CalendarVariant` | `full` | `mini`: one row of days in the dropdown. For a day or a date-time. |
| `days` | `usize` | `7` | Days in the mini calendar's row. |
| `columns` | `usize` | `2` | Months side by side, for a range of days. |
| `close_on_change` | `bool` | `true` | Picking a day, or a range's end, closes the dropdown. |
| `name` | `FieldName<Option<V>>` | - | Posts ISO 8601; a path binds to a `Form`. |
| `validate` | `Validators<Option<V>>` | - | Rules over the value. |
| `placeholder` | `String` | - | Shown while empty. |
| `size`, `radius`, `label`, `description`, `helper`, `status`, `required`, `disabled`, `readonly` | | | The shared field props. `readonly` keeps the field focusable and posted, refuses the typing and does not open the picker. |

Props that only some value types use are ignored by the rest.

## Theme

`Theme::date_field` (`DateFieldDefaults`): `size`, `radius`,
`close_on_change`. Names, labels and errors come from `Localization::date`, a
`DateLocale` (see [localization](theming.md#localization)). Its weekday arrays
are Sunday first, as dayjs' locale files have them. The date and time patterns,
the first weekday and the range separator come from the provider's `formats`:
`Formats::AMERICAN`, the default (Sunday first, a 12-hour clock), or
`Formats::GERMAN` (Monday first, a 24-hour clock, `14. September 2026`). Any
language goes with any formats; this site is English in German formats. The
docs' `language` and `formats` controls switch each.
