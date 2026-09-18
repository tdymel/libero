# use_formats_handle

Crate: `libero`
Import: `use libero::hooks::use_formats_handle;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/formats.rs>
Index: [index.md](index.md) lists every other page
Description: Switches the date, time and number formats at runtime, independent of the language.

`use_formats_handle() -> FormatsHandle` switches the date, time and number
formats at runtime, independent of the language. A region picker is built on
it. [Localization](localization.md) shows how to write formats of your own.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Text},
    hooks::use_formats_handle,
    localization::Formats,
};

#[component]
fn RegionSwitch() -> Element {
    let formats = use_formats_handle();
    let german = *formats.get() == Formats::GERMAN;
    let first = formats.get().first_weekday;

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| formats.set(if german { &Formats::AMERICAN } else { &Formats::GERMAN }),
            if german { "Use American formats" } else { "Use German formats" }
        }
        Text { "Weeks start on {first}, and 5.4 MB reads 5{formats.get().decimal_separator}4 MB." }
    }
}
```

Every calendar, date field and time picker re-renders on a switch. The
provider reads its `formats` prop once, at mount, so switch through the handle.

## API

```rust,ignore
pub fn use_formats_handle() -> FormatsHandle
```

| Method | Returns | Description |
|---|---|---|
| `get()` | `&'static Formats` | The active formats. |
| `set(formats: &'static Formats)` | `()` | Switches them. |

`Copy`.
