# use_scroll_area

Crate: `libero`
Import: `use libero::components::use_scroll_area;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/scroll_area/handle.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A handle that scrolls a ScrollArea from code.

`use_scroll_area() -> ScrollAreaHandle` scrolls a `ScrollArea` from code. Pass
it as the area's `handle`. [ScrollArea](scroll_area.md) documents the
component and its scroll events.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Flex, ScrollArea, Text, use_scroll_area},
    sx::sx,
};

#[component]
fn Log() -> Element {
    let area = use_scroll_area();

    rsx! {
        Box { sx: sx().height("120px").border("1px solid var(--lsx-muted-3)"),
            ScrollArea { aria_label: "Log", handle: area,
                for i in 1..=20 {
                    Text { key: "{i}", "Entry {i}" }
                }
            }
        }
        Flex { direction: "row", gap: "sm",
            Button { variant: "outlined", onclick: move |_| area.scroll_to_percent(None, Some(0.0)), "Top" }
            Button { variant: "outlined", onclick: move |_| area.scroll_to_percent(None, Some(100.0)), "Bottom" }
        }
    }
}
```

`scroll_to(x, y)` takes pixels from the inline start. `scroll_to_percent(x, y)`
takes percentages, and `None` leaves that axis where it is. Unlike the percent
props, every call scrolls, even back to a position asked for before.

## API

```rust,ignore
pub fn use_scroll_area() -> ScrollAreaHandle
```

| Method | Returns | Description |
|---|---|---|
| `scroll_to(x: f64, y: f64)` | `()` | Scrolls to an offset in px. `x` counts from the inline start. |
| `scroll_to_percent(x: Option<f64>, y: Option<f64>)` | `()` | Scrolls to a percent (0-100) of each axis's range. `None` keeps that axis. |

`Copy`. A call before the bound `ScrollArea` has mounted does nothing.
