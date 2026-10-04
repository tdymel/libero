# Media query

Crate: `libero`
Import: `use libero::hooks::{use_is_mobile, use_media_query};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/media_query.rs>
Index: [index.md](index.md) lists every other page
Description: Whether a CSS media query or the 768px mobile breakpoint matches, live, with a documented default where nothing can measure.

`use_media_query(query: &str) -> ReadSignal<bool>` answers whether a CSS media
query matches, and keeps answering as the viewport or the reader's settings
change. `use_is_mobile() -> ReadSignal<bool>` is `(max-width: 767px)`: the 768px
breakpoint.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Flex, Text},
    hooks::{use_is_mobile, use_media_query},
};

#[component]
fn Layout() -> Element {
    let mobile = use_is_mobile();
    let wide = use_media_query("(min-width: 1024px)");

    rsx! {
        Flex { direction: "column", gap: "sm",
            Text {
                if mobile() {
                    "Mobile: under 768px"
                } else {
                    "Not mobile"
                }
            }
            Text {
                if wide() {
                    "Wide: 1024px or more"
                } else {
                    "Narrower than 1024px"
                }
            }
        }
    }
}
```

## First render

Both answer `false` on the first render and on a server render, then the real
answer once the component is mounted. Where the platform cannot answer, as on
native Blitz, they stay `false`. Passing a different query re-subscribes.

## API

```rust,ignore
pub fn use_media_query(query: &str) -> ReadSignal<bool>
pub fn use_is_mobile() -> ReadSignal<bool>
```

| Platform | Answer |
|---|---|
| Web | `matchMedia`, live. |
| WebView (desktop, Android) | `matchMedia` over the page's script, after its first reply. |
| Native Blitz, server render | `false`, always. |

Call both hooks unconditionally, in the same order every render.

## Accessibility

### Libero handles

- The answer follows the viewport and the reader's own settings live, so a
  zoomed page that narrows past a breakpoint switches layout.

### You must

- Keep the content and every control reachable in both layouts: a breakpoint may
  rearrange a page, never drop what a reader needs.
- Prefer CSS `@media` rules for pure styling. Use the hook when the component
  tree itself differs, such as a drawer in place of a sidebar.

### Limits

- The first render answers `false` and the real answer lands after mount, so a
  layout chosen by the hook flashes its default once. Native Blitz has no media
  queries and always answers `false`.
