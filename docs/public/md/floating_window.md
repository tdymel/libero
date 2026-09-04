# Floating window

Crate: `libero`
Import: `use libero::{components::{FloatingWindowOptions, WindowRect}, hooks::{use_floating_window, FloatingWindowHandle}};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/floating_window.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A non-modal window over the page that drags, moves by keyboard and resizes from a corner; a hook owns whether it exists.

A non-modal window over the page: a title bar that drags, an optional corner
resize handle, and a close button. `use_floating_window` owns whether it exists
and returns a `Copy` handle; the window owns where it is and how big. No overlay,
no focus trap - the page stays usable underneath.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, FloatingWindowOptions, Text},
    hooks::use_floating_window,
    sx::sx,
};

#[component]
fn Demo() -> Element {
    let inspector = use_floating_window(
        FloatingWindowOptions {
            title: Some("Inspector".into()),
            placement: "bottom-end".into(),
            resizable: true,
            sx: sx().min_width("16rem").max_width("40rem").into(),
            ..Default::default()
        },
        |window| rsx! {
            Text { "Drag the title bar, or focus it and use the arrow keys." }
            Button { onclick: move |_| window.close(), "Done" }
        },
    );

    rsx! { Button { onclick: move |_| inspector.toggle(), "Inspector" } }
}
```

Call it under `LiberoProvider`, in a component that outlives every trigger: the
window is portaled from there.

The docs page's preview opens three windows - Inspector (`bottom-end`), Notes
(`top-end`, which opens a modal) and Layers (`bottom-start`) - one at a time or
all at once, to show stacking. Its switches set `resizable`, `pinned` and
`onmove`/`onresize` on all three, and the code block prints the example opened
last. The windows sit on the viewport, not in the preview: a window is
`position: fixed` and clamped to the viewport.

Reporting the geometry, with one callback for both events:

```rust
let mut last = use_signal(|| None::<WindowRect>);
let report = use_callback(move |rect: WindowRect| last.set(Some(rect)));
let inspector = use_floating_window(
    FloatingWindowOptions {
        title: Some("Inspector".into()),
        onmove: Some(report),
        onresize: Some(report),
        ..Default::default()
    },
    |window| rsx! { Button { onclick: move |_| window.close(), "Done" } },
);
```

## API

`use_floating_window(options, render) -> FloatingWindowHandle`. `render` gets the
handle, so the body can close its own window.

| `FloatingWindowHandle` | What |
|---|---|
| `open()` | Show it; remembers where focus was. No-op when open |
| `close()` | Hide it; focus returns to whatever opened it |
| `toggle()` | One or the other |
| `is_open()` | Subscribed read |

| `FloatingWindowOptions` field | Type | Default | What |
|---|---|---|---|
| `title` | `Option<String>` | none | Title bar heading and the accessible name |
| `aria_label` | `Option<String>` | none | Overrides `title` as the name |
| `placement` | `Input<Placement>` | `theme.floating_window.placement` (`center-center`) | Where it first appears |
| `resizable` | `bool` | `false` | Draws the corner resize handle |
| `pinned` | `bool` | `false` | No drag, no keyboard move |
| `z_index` | `Input<ThemeAwareValue>` | stacked from `theme.z_index.window` | Overrides the stacking |
| `sx` | `Input<Sx>` | - | On the window; put `min_width`/`max_width`/`min_height`/`max_height` here |
| `onmove` | `Option<Callback<WindowRect>>` | - | After a drag or keyboard move |
| `onresize` | `Option<Callback<WindowRect>>` | - | After a resize |

`WindowRect { x, y, width, height }` is in viewport pixels.

## Geometry

- Built on `Float { fixed: true }`. It stays at `placement` until moved, then
  sits where it was put, clamped into the viewport by CSS so it re-clamps when
  its own size changes. A window whose `min_width` is wider than the viewport
  pins to the top-left.
- The resize handle asks for a size; your `sx` constraints clamp it. The
  viewport caps it on top of them, always: a `max_width` of `40rem` is still
  no wider than a phone.
- A drag re-renders the whole window, `children` included. Keep the body shallow.

## Stacking

Windows sit on their own layer, `theme.z_index.window` (250): above the page's
dropdowns, below overlays and modals, so a modal opened from a window covers it.
Clicking or focusing a window brings it to the front. The z-indices are a dense
run from 250, capped below the overlay layer however many windows are open.

## Accessibility

- Give it a `title`: it names the window.
- It takes focus on open; Escape, the close button or `close()` hands focus
  back to the trigger.
- The title bar is a tab stop: Arrow moves 10px (`move_step`), Shift+Arrow 1px.
- On the resize handle Arrow resizes by `resize_step`, Shift+Arrow by 1px, and
  Home/End ask for the smallest/largest size the constraints allow.

## Theme

`theme.floating_window`: `placement`, `radius`, `shadow`, `move_step`,
`resize_step`, `move_label`, `resize_label`, `close_label`. Plain values read
from Rust; the chrome is `Paper`'s.
