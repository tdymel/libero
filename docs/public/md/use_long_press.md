# Long press

Crate: `libero`
Import: `use libero::hooks::{LongPress, LongPressOptions, use_long_press};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/long_press.rs>
Index: [index.md](index.md) lists every other page
Description: Pointer handlers that call back once a press is held, without breaking a tap or a scroll.

`use_long_press(on_long_press, options) -> LongPress` calls `on_long_press` once
a pointer stays down for `options.ms` (400 by default). It moves no more than
`move_tolerance` px (10) in that time. Spread the returned callbacks onto one
element: `onpointerdown`, `onpointermove`, `onpointerup`, `onpointerleave`,
`onpointercancel`, `oncontextmenu` and `onclick`.

Read `pressing`, a signal that is true while a press is held and not yet fired,
for a hold cue.

Give the element `user-select: none` and `-webkit-touch-callout: none` so
holding does not select text. It runs on the web, Blitz and a WebView, on one
timer.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex},
    hooks::{LongPressOptions, use_long_press},
};

#[component]
fn HoldToCount() -> Element {
    let mut taps = use_signal(|| 0);
    let mut holds = use_signal(|| 0);
    let press = use_long_press(
        Callback::new(move |()| holds += 1),
        LongPressOptions::default(),
    );

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Button {
                onpointerdown: move |event| press.onpointerdown.call(event),
                onpointermove: move |event| press.onpointermove.call(event),
                onpointerup: move |event| press.onpointerup.call(event),
                onpointerleave: move |event| press.onpointerleave.call(event),
                onpointercancel: move |event| press.onpointercancel.call(event),
                oncontextmenu: move |event| press.oncontextmenu.call(event),
                onclick: move |event| {
                    if !press.onclick.call(event) {
                        taps += 1;
                    }
                },
                if (press.pressing)() {
                    "Keep holding"
                } else {
                    "Tap, or hold"
                }
            }
            // The same action without holding (WCAG 2.5.1, 2.1.1).
            Button { variant: "outlined", onclick: move |_| holds += 1, "Count a hold" }
            div { role: "status", "{taps} taps, {holds} holds" }
        }
    }
}
```

## API

```rust,ignore
pub fn use_long_press(on_long_press: Callback, options: LongPressOptions) -> LongPress

pub struct LongPressOptions {
    pub ms: u64,
    pub move_tolerance: f64,
}
```

| Option | Default | Description |
|---|---|---|
| `ms` | `400` | How long the pointer must stay down, in milliseconds. |
| `move_tolerance` | `10.0` | How far, in CSS px, the pointer may move before the press is a drag. |

| Field of `LongPress` | Type | Description |
|---|---|---|
| `onpointerdown` | `Callback<PointerEvent>` | Starts the press. |
| `onpointermove` | `Callback<PointerEvent>` | Drops the press beyond the tolerance. |
| `onpointerup`, `onpointerleave`, `onpointercancel` | `Callback<PointerEvent>` | Drop the press. |
| `oncontextmenu` | `Callback<MouseEvent>` | Suppresses the browser's menu, only after the press fired. |
| `onclick` | `Callback<MouseEvent, bool>` | Call it first in your `onclick`: `true` for the click a fired press ends with, which you should skip. |
| `pressing` | `ReadSignal<bool>` | `true` from the pointer going down until the press fires or is dropped. |

`LongPress` is `Copy`. Pass the same `LongPressOptions` each render; the delay
of the newest render is the one used.

## Accessibility

### Libero handles

- A tap that ends before the delay runs no callback and keeps its click.
- The press is dropped when the pointer moves beyond the tolerance, leaves, is
  cancelled (a touch that starts to scroll), or a second finger lands.
- The browser's own long-press menu is suppressed only after the press fired,
  and the click that follows the release is reported so you can skip it.

### You must

- Offer the same action without holding: a long press is a gesture with no
  keyboard or switch equivalent (WCAG 2.5.1, 2.1.1). Put it on a context menu,
  a key such as Shift+F10, or a visible button, as the demo's "Count a hold"
  does.
- Announce what the press did with a live region, as the demo does.

### Limits

- It reacts to pointer events only. Keyboard and screen reader activation arrive
  as clicks and do not count as a press.
