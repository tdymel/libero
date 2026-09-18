# use_formats

Crate: `libero`
Import: `use libero::hooks::use_formats;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/formats.rs>
Index: [index.md](index.md) lists every other page
Description: How the active region writes dates, times and numbers.

`use_formats() -> &'static Formats` returns how the active region writes dates,
times and numbers: the first weekday, the date and time patterns, and the range
and decimal separators. [Localization](localization.md) describes each field.

## Usage

```rust
use dioxus::prelude::*;
use libero::{components::Text, hooks::use_formats};

#[component]
fn FileSize() -> Element {
    let formats = use_formats();

    rsx! {
        Text { "report.pdf, 5{formats.decimal_separator}4 MB" }
    }
}
```

The docs site runs in `Formats::GERMAN`, so the size reads with a comma. The
component re-renders when the formats are switched with
[use_formats_handle](use_formats_handle.md).

## API

```rust,ignore
pub fn use_formats() -> &'static Formats
```

Call it under `LiberoProvider`. Unset, the provider's `formats` is
`Formats::AMERICAN`.
