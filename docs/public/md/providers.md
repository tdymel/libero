# Providers

Crate: `libero`
Import: `use libero::{IconProvider, IconSet, IconSlot, LiberoProvider};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/context/libero/mod.rs>
Index: [index.md](index.md) lists every other page
Description: LiberoProvider (themes, localization, formats, direction) and IconProvider (glyphs): what each provides, and how different parts of one page can sit below different providers.

`LiberoProvider` is the root every app renders once: themes, localization,
formats, stylesheets and portals. `IconProvider` swaps libero's glyphs. Both are
plain context providers, so different parts of one page can sit below different
ones, and the nearest wins.

## Different providers in one page

A `LiberoProvider` inside another gives its own subtree its own `localization`
and `formats`. Everything outside keeps the outer ones. The live demo's outer
calendar follows the site, English words and German formats, so only the words
differ there.

```rust
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    chrono::NaiveDate,
    components::DatePicker,
    localization::{Formats, Localization},
};

#[component]
fn Calendars() -> Element {
    let day = NaiveDate::from_ymd_opt(2026, 3, 14);

    rsx! {
        LiberoProvider {
            DatePicker { value: day, onchange: |_| {} }
            // German words and formats, only in here.
            LiberoProvider {
                localization: &Localization::GERMAN,
                formats: &Formats::GERMAN,
                DatePicker { value: day, onchange: |_| {} }
            }
        }
    }
}
```

Three limits:

- The theme is a sheet on the document root, so a nested provider's `themes`
  do not scope to its subtree.
- `direction` also sets the document root. For one subtree, put a `dir="rtl"`
  attribute on an element around it.
- `use_localization_handle` and `use_formats_handle` switch the nearest provider
  only.

## Icon providers merge

Nested `IconProvider`s merge: the inner one wins per slot, the rest comes from
the outer one, then lucide.

```rust
use dioxus::prelude::*;
use libero::{IconProvider, IconSet, IconSlot, LiberoProvider, components::Checkbox};
use pictogram_icons_lucide as lucide;

#[component]
fn Checks() -> Element {
    rsx! {
        LiberoProvider {
            IconProvider {
                icons: IconSet::new().with(IconSlot::CheckboxCheck, lucide::check_check::outlined),
                Checkbox { label: "Outer", checked: true, onchange: |_| {} }
                IconProvider {
                    icons: IconSet::new().with(IconSlot::CheckboxCheck, lucide::square_check::outlined),
                    Checkbox { label: "Inner", checked: true, onchange: |_| {} }
                }
            }
        }
    }
}
```

## Props

### LiberoProvider

| Prop | Type | Default | Description |
|---|---|---|---|
| `themes` | `ThemeSet` | `ThemeSet::DEFAULT` | Every theme the app ships; the light and dark halves share one sheet. A lone `&'static Theme` is a set of one. |
| `localization` | `&'static Localization` | `Localization::ENGLISH` | Every string libero shows a reader. Read at mount; switch it later with `use_localization_handle`. |
| `formats` | `&'static Formats` | `Formats::AMERICAN` | How dates, times and numbers are written, whatever the language. Read at mount; switch it later with `use_formats_handle`. |
| `direction` | `Option<Direction>` | `None` | The start text direction, set as the document root's `dir`. A choice made through `use_direction` is kept on the web and wins. |

### IconProvider

| Prop | Type | Default | Description |
|---|---|---|---|
| `icons` | `IconSet` | required | The slots to swap. An empty slot keeps the outer provider's glyph, then libero's lucide default. |
