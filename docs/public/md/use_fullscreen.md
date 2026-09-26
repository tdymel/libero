# Fullscreen

Crate: `libero`
Import: `use libero::hooks::{FullscreenHandle, use_fullscreen};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/fullscreen.rs>
Index: [index.md](index.md) lists every other page
Description: Puts one of your elements in fullscreen, natively or drawn as a fixed box where the platform refuses it; the engine behind `Video`'s fullscreen.

`use_fullscreen(element) -> FullscreenHandle` puts one of your elements in
fullscreen. Spread its `attributes()` on the element and mount it with
`use_element`. Where the page refuses the Fullscreen API (a headless browser,
an iframe without `allowfullscreen`), the handle draws it: it sets `data-fullscreen="drawn"` and
your CSS turns the element into a fixed box over the page. [Video](video.md) is
built on it.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Text},
    hooks::{use_element, use_fullscreen},
};

#[component]
fn Panel() -> Element {
    let panel = use_element();
    let fullscreen = use_fullscreen(panel);

    rsx! {
        style { "#panel[data-fullscreen='drawn'] {{ position: fixed; inset: 0; z-index: 1000; }}" }
        div { id: "panel", onmounted: panel.mount(), ..fullscreen.attributes(),
            Text { "A chart, a map or a slide deck." }
            Button {
                onclick: move |_| fullscreen.toggle(),
                if fullscreen.is_fullscreen() { "Exit fullscreen" } else { "Fullscreen" }
            }
        }
    }
}
```

## API

```rust,ignore
pub fn use_fullscreen(element: ElementHandle) -> FullscreenHandle
```

| Method of `FullscreenHandle` | Description |
|---|---|
| `attributes()` | Spread on the element: the element handle's own attributes, `data-fullscreen` and the listeners that leave the drawn fullscreen. |
| `enter()`, `exit()`, `toggle()` | `enter()` asks for native fullscreen and draws it where that is refused; `exit()` leaves either. |
| `is_fullscreen()` | Native or drawn, reactive. |
| `is_drawn()` | Only the drawn fullscreen, reactive: your CSS or style must draw it. |

The element carries `data-fullscreen="native"` or `"drawn"` while it fills the
screen. `FullscreenHandle` is `Copy`.

| Platform | Support |
|---|---|
| Web | Native where the browser grants it, else drawn. |
| Linux desktop (WebKitGTK) | Native, checked by the e2e suite; drawn when refused. |
| Android (WebView) | Native, checked by the e2e suite on the emulator; drawn when refused. |
| macOS, Windows desktop | Untested. |
| Blitz, server render | Always drawn. |

## Accessibility

### Libero handles

- Escape leaves the drawn fullscreen; native fullscreen leaves on Escape by
  itself.
- Focus leaving the element leaves the drawn fullscreen, so the covered page is
  never focused unseen.

### You must

- Give the toggle a name that says what it does now, such as Fullscreen or Exit
  fullscreen.
- Keep a visible way out inside the element: a touch screen has no Escape.

### Limits

- Blitz and a server render have no Fullscreen API: the handle always draws it.
- The drawn fullscreen keeps the browser's bars and the device's status bar.
