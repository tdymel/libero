# use_theme_set

Crate: `libero`
Import: `use libero::hooks::use_theme_set;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/theme.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Reads and swaps the active theme set, which a theme picker is built on.

`use_theme_set() -> ThemeSetHandle` reads and swaps the active theme set, the
light and dark pair the app is drawn in. A theme picker is built on it.
[Theming](theming.md) covers theme sets and the catalogue.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex},
    hooks::use_theme_set,
    theme::ThemeSet,
};

#[component]
fn ThemePicker() -> Element {
    let themes = use_theme_set();

    rsx! {
        Flex { direction: "row", gap: "sm",
            for set in ThemeSet::CATALOGUE.iter().take(4) {
                Button {
                    variant: if themes.name() == set.name() { "filled" } else { "outlined" },
                    aria_pressed: themes.name() == set.name(),
                    onclick: {
                        let themes = themes.clone();
                        move |_| themes.set((*set).clone())
                    },
                    "{set.name()}"
                }
            }
        }
    }
}
```

A swap rebuilds the stylesheet, because the sheet carries the set's pair. The
colour scheme setting survives it, so a reader who pinned dark stays in dark.

## API

```rust,ignore
pub fn use_theme_set() -> ThemeSetHandle
```

| Method | Returns | Description |
|---|---|---|
| `get()` | `ThemeSet` | The active set, cloned. |
| `name()` | `&'static str` | What the active set calls itself. |
| `set(themes: ThemeSet)` | `()` | Swaps the set. |

`Clone`, not `Copy`: clone it into each handler.
