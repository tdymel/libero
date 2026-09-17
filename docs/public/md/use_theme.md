# use_theme

Crate: `libero`
Import: `use libero::hooks::use_theme;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/theme.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: The active theme, for the values CSS cannot carry.

`use_theme() -> &'static Theme` returns the active theme. Read it for a value
CSS cannot carry, such as a number a hook computes with. The component
re-renders when the theme changes. [Theming](theming.md) explains the theme
itself.

## Usage

```rust
use dioxus::prelude::*;
use libero::{components::Text, hooks::use_theme};

#[component]
fn PopoverSpacing() -> Element {
    let theme = use_theme();

    rsx! {
        Text {
            "Popovers sit {theme.popover.gap}px from their anchor, "
            "with {theme.popover.padding}px of padding."
        }
    }
}
```

For a colour or a size in your own styles, reach for `sx` and the theme's CSS
variables instead. They follow a scheme switch without a re-render.

## API

```rust,ignore
pub fn use_theme() -> &'static Theme
```

Call it under `LiberoProvider`.
