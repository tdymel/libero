# Localization

Crate: `libero`
Import: `use libero::{LiberoProvider, localization::{Formats, Localization}};`
Index: [index.md](index.md) - every other component's markdown page
Description: The words components say on their own and how dates and numbers are written: `Localization`, `Formats` and the hooks that switch them.

Every string a component says on its own, such as an accessible name, an
announcement or a month name, comes from `LiberoProvider`'s `localization`. Two
languages ship: `Localization::ENGLISH`, the default, and `Localization::GERMAN`.

How a date or a number is written depends on the region, not the language, so
it is a prop of its own, `formats`. Any language goes with any formats: this
site is English in German formats.

## Language and formats

| Const | First weekday | Day format | Clock | Decimal separator |
|---|---|---|---|---|
| `Formats::AMERICAN` (default) | Sunday | `September 14, 2026` | `3:30 PM` | `.` |
| `Formats::GERMAN` | Monday | `14. September 2026` | `15:30` | `,` |

The docs page's two switches change the site's.

```rust
use dioxus::prelude::*;
use libero::{LiberoProvider, localization::{Formats, Localization}};

fn App() -> Element {
    rsx! {
        LiberoProvider {
            localization: &Localization::GERMAN,
            formats: &Formats::GERMAN,
            Router::<Route> {}
        }
    }
}
#
# #[derive(Clone, PartialEq, Routable)]
# enum Route {
#     #[route("/")]
#     Home {},
# }
# #[component] fn Home() -> Element { rsx! {} }
```

## Formats

`Formats` holds the first weekday, a date pattern per level (day, month, year),
the month heading, the time pattern, and the range and decimal separators. The
patterns use dayjs tokens such as `YYYY`, `MMMM`, `D` and `HH`. An `h` or an `A`
in the time pattern makes the pickers 12-hour.

```rust
use libero::localization::Formats;

static SWISS: Formats = Formats {
    decimal_separator: ".",
    ..Formats::GERMAN
};
```

| Field | Type | Description |
|---|---|---|
| `first_weekday` | `chrono::Weekday` | The first column of a calendar. |
| `date` | `fn(DateLevel) -> &'static str` | How a date field shows a day, a month or a year. Text in `[brackets]` is literal. |
| `month_heading` | `&'static str` | A calendar's month heading. |
| `time` | `&'static str` | How a time is shown. |
| `range_separator` | `&'static str` | Between a range's two ends in a field's text. |
| `decimal_separator` | `&'static str` | Between a number's whole and its fraction: `5.4 MB`, `5,4 MB`. |

## Changing the words

`Localization` is shaped like a theme: one struct with a group per component,
plus `common` for the words many components share. Change it with struct update
syntax in a `static`.

```rust
use libero::localization::{CommonLabels, Localization, PaginationLabels};

static WORDS: Localization = Localization {
    common: CommonLabels {
        close: "Zumachen",
        ..CommonLabels::GERMAN
    },
    pagination: PaginationLabels {
        page: "Gehe zu Seite {n}",
        ..PaginationLabels::GERMAN
    },
    ..Localization::GERMAN
};
```

A string with a value in it is a template with named holes, such as
`"Go to page {n}"`, so a language can put the value where its grammar wants it.
`fill(template, &[("n", &3)])` fills the holes. A prop that names what only the
call site knows, such as a dialog's close label, still wins over the
localization.

The groups live in `libero::localization`, each with an `ENGLISH` and a `GERMAN`
const: `CommonLabels`, `DateLocale`, `PaginationLabels`, `AvatarLabels`,
`BurgerLabels`, `AnchorLabels`, `PinFieldLabels`, `ColorSchemeButtonLabels`,
`SpotlightLabels`, `CarouselLabels`, `NavLinkLabels`, `LightboxLabels`,
`FloatingWindowLabels`, `NotificationsLabels`, `ScrollerLabels`,
`StepperLabels`, `MarqueeLabels`, `ChipsLabels`, `ComboboxLabels`,
`TagsFieldLabels`, `ImageLabels`, `CodeBlockLabels`, `ColorLabels`,
`PhoneFieldLabels`, `PasswordFieldLabels`, `NumberFieldLabels`,
`FileFieldLabels`, `TextareaLabels`, `SliderLabels` and `MenuLabels`.
`DateLocale` holds the month and weekday names and every date and time
component's labels.

## Switching at runtime

`use_localization()` and `use_formats()` read the active ones, and every
component that reads them re-renders on a switch. `use_localization_handle()`
and `use_formats_handle()` add `get()` and `set()`, which a language or region
picker is built on. The provider reads its props once, at mount, so switch
through the handles.

```rust
use dioxus::prelude::*;
use libero::{
    components::Button,
    hooks::{use_formats_handle, use_localization_handle},
    localization::{Formats, Localization},
};

#[component]
fn LanguagePicker() -> Element {
    let localization = use_localization_handle();
    let formats = use_formats_handle();

    rsx! {
        Button {
            onclick: move |_| {
                localization.set(&Localization::GERMAN);
                formats.set(&Formats::GERMAN);
            },
            "Deutsch"
        }
    }
}
```

The handles take a `&'static` reference, so a catalogue loaded at runtime is
leaked once per language with `Box::leak`.

## API

| Hook | Returns | Description |
|---|---|---|
| `use_localization()` | `&'static Localization` | The active words. Reactive. |
| `use_localization_handle()` | `LocalizationHandle` | `get()` and `set(&'static Localization)`. |
| `use_formats()` | `&'static Formats` | The active formats. Reactive. |
| `use_formats_handle()` | `FormatsHandle` | `get()` and `set(&'static Formats)`. |

Each hook panics outside a `LiberoProvider`. `set` with the active value does
nothing, so readers do not re-render.
