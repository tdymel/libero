# use_localization

Crate: `libero`
Import: `use libero::hooks::use_localization;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/localization.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: The words libero's components say on their own, in the active language.

`use_localization() -> &'static Localization` returns the words libero's
components say on their own, in the active language. Read it where your
component says the same thing, so it switches language with them.
[Localization](localization.md) lists what it holds.

## Usage

```rust
use dioxus::prelude::*;
use libero::{components::Text, hooks::use_localization};

#[component]
fn CloseLabel() -> Element {
    let words = use_localization();

    rsx! {
        Text { "Every close button is named \"{words.common.close}\"." }
    }
}
```

The component re-renders when the language is switched with
[use_localization_handle](use_localization_handle.md).

## API

```rust,ignore
pub fn use_localization() -> &'static Localization
```

Call it under `LiberoProvider`. Unset, the provider's `localization` is
`Localization::ENGLISH`.
