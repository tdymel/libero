# use_scroller

Crate: `libero`
Import: `use libero::components::use_scroller;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/scroller.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A handle that steps a Scroller from controls of your own.

`use_scroller() -> ScrollerHandle` steps a `Scroller` from controls of your
own, by the same amount its built-in arrows would. Pass it as the scroller's
`handle` from the first render. [Scroller](scroller.md) documents the
component.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Chip, Flex, Scroller, use_scroller},
    sx::sx,
};

const NAMES: [&str; 6] = ["Rust", "Dioxus", "WebAssembly", "Accessibility", "Layout", "Theming"];

#[component]
fn Tags() -> Element {
    let strip = use_scroller();

    rsx! {
        Box { sx: sx().max_width("16rem"),
            Scroller { aria_label: "Tags", controls: "never", handle: strip,
                Flex { direction: "row", gap: "sm", wrap: "nowrap",
                    for name in NAMES {
                        Chip { key: "{name}", "{name}" }
                    }
                }
            }
        }
        Flex { direction: "row", gap: "sm",
            Button { variant: "outlined", onclick: move |_| strip.step_back(), "Back" }
            Button { variant: "outlined", onclick: move |_| strip.step_forward(), "Forward" }
        }
    }
}
```

A step starts from where the strip is now, so a touch scroll in between is
kept. A call before the scroller has mounted does nothing.

## API

```rust,ignore
pub fn use_scroller() -> ScrollerHandle
```

| Method | Returns | Description |
|---|---|---|
| `step_forward()` | `()` | One step towards the end, as the forward control does. |
| `step_back()` | `()` | One step towards the start, as the backward control does. |

`Copy`.
