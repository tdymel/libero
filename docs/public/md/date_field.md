# DateField

Crate: `libero`
Import: `use libero::{chrono::NaiveDate, components::DateField};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/date/date_field.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A text field holding a `Date`, typed leniently against a dayjs format, with a `DatePicker` in a dropdown.

Modelled on Mantine's `DateInput`. The dropdown is [DatePicker](date_picker.md).

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Date, DateField};

#[component]
fn Demo() -> Element {
    let mut day = use_signal(|| None::<Date>);

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

## Typing

Text stays exactly as typed until the field blurs or Enter is pressed. Then it
is read against `format`, leniently - only the order of day, month and year
has to match:

| Input | Read as, for `DD.MM.YYYY` |
|---|---|
| `1/2/2026`, `01-02-2026`, `1. 2. 2026` | February 1, 2026 |
| `01022026` | February 1, 2026 (digits alone, at full width) |
| `1 feb 2026`, `1 FEBR 2026` | February 1, 2026 (a name, or a prefix naming one month) |
| `1.2` | February 1 of the current year |
| `1.2.26` | rejected - two-digit years are not read |
| `31.2.2026` | rejected - no such day |

A weekday name in the text is skipped. Emptied text commits `None`. Text that
is not a day the field accepts - including one outside `min`/`max` or excluded
- stays, and the field shows `DateDefaults::invalid_date` as its error.

## Posting

The text input carries no `name`. A hidden input does, holding the day as
ISO 8601 (`2026-02-01`) whatever `format` shows, so a server never parses a
display format.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Option<Date>` | - | The day; strictly controlled. |
| `onchange` | `EventHandler<Option<Date>>` | - | On commit (blur, Enter) and on a picked day. |
| `format` | `String` | `DateDefaults::format` | dayjs tokens: `YYYY M MM MMM MMMM D DD dd ddd dddd`, `[literal]`. |
| `min` / `max` | `Date` | - | Limits for picking and typing. |
| `exclude_date` | `Callback<Date, bool>` | - | Days that cannot be picked or typed. |
| `today` | `Date` | clock | Marked day, and the year a yearless text takes. |
| `close_on_change` | `bool` | `true` | Picking a day closes the dropdown. |
| `name` | `FieldName<Option<Date>>` | - | Posts ISO 8601; a path binds to a `Form`. |
| `validate` | `Validators<Option<Date>>` | - | Rules over the day. |
| `placeholder` | `String` | - | Shown while empty. |
| `size`, `radius`, `label`, `description`, `helper`, `status`, `required`, `disabled` | | | The shared field props. |

## Theme

`Theme::date_field` (`DateFieldDefaults`): `size`, `radius`,
`close_on_change`. Names and formats come from `Theme::date`.
