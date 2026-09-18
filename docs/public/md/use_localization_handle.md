# use_localization_handle

Crate: `libero`
Import: `use libero::hooks::use_localization_handle;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/localization.rs>
Index: [index.md](index.md) lists every other page
Description: Switches the language libero's components speak at runtime.

`use_localization_handle() -> LocalizationHandle` switches the language at
runtime. A language picker is built on it. [Localization](localization.md)
covers the languages that ship and how to change the words.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Text},
    hooks::use_localization_handle,
    localization::Localization,
};

#[component]
fn LanguageSwitch() -> Element {
    let localization = use_localization_handle();
    let german = *localization.get() == Localization::GERMAN;

    rsx! {
        Button {
            variant: "outlined",
            lang: if german { "en" } else { "de" },
            onclick: move |_| {
                localization.set(if german { &Localization::ENGLISH } else { &Localization::GERMAN });
            },
            if german { "English" } else { "Deutsch" }
        }
        Text { "Close buttons say \"{localization.get().common.close}\"." }
    }
}
```

Every component that reads the localization re-renders on a switch. The
provider reads its `localization` prop once, at mount, so switch through the
handle.

## Accessibility

The button is labelled in the language it switches to, and its `lang` says so,
so a screen reader pronounces "Deutsch" in German.

## API

```rust,ignore
pub fn use_localization_handle() -> LocalizationHandle
```

| Method | Returns | Description |
|---|---|---|
| `get()` | `&'static Localization` | The active localization. |
| `set(localization: &'static Localization)` | `()` | Switches it. A catalogue loaded at runtime is leaked once per language with `Box::leak`. |

`Copy`.
