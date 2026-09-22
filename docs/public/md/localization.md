# Localization

Crate: `libero`
Import: `use libero::{LiberoProvider, localization::{Formats, Localization}};`
Index: [index.md](index.md) lists every other page
Description: The words components say on their own, how dates and numbers are written, and the reading direction: `Localization`, `Formats` and the hooks that switch them.

Every word a component says on its own, such as an accessible name or a month,
comes from `LiberoProvider`'s `localization`. How dates and numbers are written
depends on the region, so it is a prop of its own, `formats`. Any language goes
with any formats: this site is English in German formats.

Direction comes from the document. Components follow a `dir="rtl"` around them,
and their directional props say start and end, not left and right.

## Usage

The docs page's controls switch the site's language, formats and direction.

```rust
use dioxus::prelude::*;
use libero::{LiberoProvider, localization::{Formats, Localization}};

fn App() -> Element {
    rsx! {
        // Round the provider, so its portals (menus, popovers) flip too.
        div { dir: "rtl",
            LiberoProvider {
                localization: &Localization::GERMAN,
                formats: &Formats::GERMAN,
                Router::<Route> {}
            }
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

## What ships

`Localization::ENGLISH` is the default language, and `Localization::GERMAN` the
other. Two formats ship too:

| Formats | First weekday | Day | Time |
|---|---|---|---|
| `Formats::AMERICAN` (default) | Sunday | `September 14, 2026` | `3:30 PM` |
| `Formats::GERMAN` | Monday | `14. September 2026` | `15:30` |

## Your own words and formats

Both are plain structs. Change what you need with struct update syntax in a
`static`. The format patterns use dayjs tokens such as `D. MMMM YYYY`; an `h` or
an `A` in the time pattern makes the pickers 12-hour.

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

`Localization` has a group per component, plus `common` for the words many
share. A value inside a string is a named hole such as `{n}`, so each language
puts it where its grammar wants it. `fill(template, &[("n", &3)])` fills the
holes. A prop that names what only the call site knows, such as a dialog's
close label, still wins over the localization.

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

The groups live in `libero::localization`, each with an `ENGLISH` and a `GERMAN`
const: `CommonLabels`, `DateLocale`, `PaginationLabels`, `AvatarLabels`,
`BurgerLabels`, `AnchorLabels`, `PinFieldLabels`, `ThemeToggleLabels`,
`RepoButtonLabels`, `TldrLabels`, `DirectionToggleLabels`, `SpotlightLabels`, `CarouselLabels`, `NavLinkLabels`, `LightboxLabels`,
`FloatingWindowLabels`, `NotificationsLabels`, `ScrollerLabels`,
`StepperLabels`, `MarqueeLabels`, `ChipsLabels`, `ComboboxLabels`,
`TagsFieldLabels`, `ImageLabels`, `CodeBlockLabels`, `CopyButtonLabels`, `ColorLabels`,
`PhoneFieldLabels`, `PasswordFieldLabels`, `NumberFieldLabels`,
`FileFieldLabels`, `TextareaLabels`, `SliderLabels` and `MenuLabels`.
`DateLocale` holds the month and weekday names and every date and time
component's labels.

## Switching at runtime

The provider reads its props once. Switch later through
`use_localization_handle()` and `use_formats_handle()`, and every component that
reads them follows.

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
