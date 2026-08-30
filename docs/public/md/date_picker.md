# DatePicker

Crate: `libero`
Import: `use libero::{chrono::NaiveDate, components::DatePicker};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/date/date_picker.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A month of days to pick one `Date` from - a keyboard grid with min, max and excluded days.

Modelled on Mantine's `DatePicker`. The value is `chrono::NaiveDate`
(re-exported as `libero::chrono`): a calendar day with no time zone.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Date, DatePicker};

#[component]
fn Demo() -> Element {
    let mut day = use_signal(|| Date::new(2026, 9, 14));

    rsx! {
        DatePicker {
            value: day(),
            onchange: move |next| day.set(next),
            min: Date::new(2026, 9, 5),
            exclude_date: |day: Date| day.weekday().index() >= 5,
            name: "arrival",
        }
    }
}
```

## The month shown

The picker keeps the month it shows as its own state. It opens on `value`'s
month, else today's. Today comes from the platform clock after mount - on the
web; a server render and native builds have no clock, so no day is marked
unless `today` is set.

## Keyboard

The grid is one tab stop: the picked day, else today, else the 1st.

| Key | Moves to |
|---|---|
| Arrow Left / Right | the day before / after |
| Arrow Up / Down | the same weekday a week earlier / later |
| Home / End | the first / last day of the week |
| Page Up / Page Down | the same day a month earlier / later |
| Shift + Page Up / Page Down | the same day a year earlier / later |
| Enter / Space | picks the day |

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Option<Date>` | - | The picked day; strictly controlled. |
| `onchange` | `EventHandler<Option<Date>>` | - | Called with the day the caller should hold next. |
| `min` | `Date` | - | The earliest day that can be picked. |
| `max` | `Date` | - | The latest day that can be picked. |
| `exclude_date` | `Callback<Date, bool>` | - | Days that cannot be picked. Always compares equal, so changing only the closure does not redraw. |
| `allow_deselect` | `bool` | `false` | Clicking the picked day again clears it. |
| `today` | `Date` | clock | The day marked as today. |
| `size` | `Size` | `md` | Day cell and font size. |
| `name` | `String` | - | Emits a hidden input posting the day as ISO 8601. |
| `focusable` | `bool` | `true` | `false` keeps days and buttons out of the tab order. |

## Theme

`Theme::date` (`DateDefaults`) holds the month and weekday names, the first
weekday, the display `format`, the heading `month_format` and the button labels
- one place to translate. `Theme::date_picker` holds the size steps.

CSS variables: `--lsx-date-picker-day-size-{size}`,
`--lsx-date-picker-font-size-{size}`.

Data attributes on each day button: `data-date` (ISO), `data-outside`,
`data-today`, `data-selected`.
