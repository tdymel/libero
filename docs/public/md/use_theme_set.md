# Theme set

Crate: `libero`
Import: `use libero::hooks::use_theme_set;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/theme.rs>
Index: [index.md](index.md) lists every other page
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

## Swapping

A swap rebuilds the stylesheet, because the sheet carries the set's pair. The
colour scheme setting survives it, so a reader who pinned dark stays in dark.

## Accessibility

### Libero handles

- A swap keeps the reader's colour scheme setting, so a reader who pinned dark
  stays in dark.

### You must

- Mark the active set on its control, as the demo's buttons do with
  `aria_pressed`.
- Check the sets you offer against your own colours: in Kanagawa, Kanagawa
  Dragon and Vague light `muted.6` falls just under 3:1, the minimum for borders
  and icons (WCAG 1.4.11).

### Example

A theme picker with one button per set, each with `aria_pressed` on the active
one: a screen reader reads "Kanagawa, pressed", and a reader who pinned dark
stays in dark after the swap.

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
