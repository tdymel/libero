# FloatingWindow

Crate: `libero`
Import: `use libero::{components::{FloatingWindowOptions, WindowRect}, hooks::{use_floating_window, FloatingWindowHandle}};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/use_floating_window.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A non-modal window over the page that drags, moves by keyboard and resizes from a corner, opened through a hook.

A non-modal window over the page, with a title bar that drags, an optional
resize corner, a menu that moves and resizes it without a drag, and a close
button. `use_floating_window` returns a `Copy` handle that opens and closes it.
There is no overlay or focus trap, so the page stays usable.

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

Call the hook under `LiberoProvider`, in a component that outlives every
trigger. The window is portaled from there and sits on the viewport.

One callback can report the geometry for both events.

```rust,ignore
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

| `FloatingWindowHandle` | Description |
|---|---|
| `open()` | Shows the window. Does nothing when it is open. |
| `close()` | Hides it and returns focus to what opened it, unless focus had already left the window. |
| `toggle()` | Opens or closes it. |
| `is_open()` | Whether it is open. Reading it subscribes. |

| `FloatingWindowOptions` field | Type | Default | Description |
|---|---|---|---|
| `title` | `Option<String>` | - | The title bar's heading, and the window's accessible name. |
| `aria_label` | `Option<String>` | - | Names the window instead of `title`. |
| `placement` | `Input<Placement>` | `center-center` | Where it first appears. Once dragged, it stays where it was put, inside the viewport. |
| `resizable` | `bool` | `false` | Draws the corner resize handle. |
| `pinned` | `bool` | `false` | Keeps it at `placement`, with no drag, keyboard move or Move menu item. |
| `z_index` | `Input<ThemeAwareValue>` | - | Overrides the stacking. Unset, windows sit below overlays and modals. |
| `sx` | `Input<Sx>` | - | Styles the window. `min_width`, `max_width`, `min_height` and `max_height` here limit a resize. The window never grows past the viewport. |
| `onmove` | `Option<Callback<WindowRect>>` | - | Called after a drag, a keyboard or button move, or a Reset, in viewport pixels. |
| `onresize` | `Option<Callback<WindowRect>>` | - | Called after a resize by pointer, keyboard or button, or a Reset. |

`WindowRect { x, y, width, height }` is in viewport pixels.

## Geometry

- The window stays at `placement` until moved, then where it was put, always
  inside the viewport. A window whose `min_width` is wider than the viewport
  pins to the top-left.
- The title bar's menu button ("Window menu") offers Move, Resize (with
  `resizable`) and Reset. Move and Resize show step buttons under the title
  bar, one click per `move_step` or `resize_step`, until Done or Escape. Reset
  puts the window back at `placement` at its content's size. Without
  `resizable` and with `pinned`, there is no menu.
- A drag re-renders the whole window, so keep the body shallow.

## Stacking

Windows sit on their own layer, `theme.z_index.window` (250), above the page's
dropdowns and below overlays and modals. A modal opened from a window covers
it. Clicking or focusing a window brings it to the front.

## Accessibility

- Give it a `title`, which names the window.
- It takes focus on open. Escape, the close button or `close()` returns focus
  to the trigger. A `close()` from elsewhere on the page leaves focus there.
- The title bar is a tab stop. Arrow keys move it 10px (`move_step`),
  Shift+Arrow 1px. Its description says "Use arrow keys to move the window".
- On the resize handle, Arrow resizes by `resize_step`, Shift+Arrow by 1px, and
  Home and End ask for the smallest and largest size allowed.
- Moving and resizing need no drag (WCAG 2.5.7). The menu's step buttons do
  both with single clicks. Choosing Move or Resize focuses the first button,
  and Done or Escape returns focus to the menu button.
- F6 moves focus between the page and the topmost window, but not while a
  modal or popover is open. Tab alone reaches the window last.
- The page behind a window still takes Tab. Pick a `placement` that does not
  cover the page's controls, such as a corner away from the header (WCAG
  2.4.11).

## Theme defaults

`FloatingWindowDefaults` on the theme: `placement`, `radius`, `shadow`,
`move_step` and `resize_step`. The chrome is `Paper`'s. The names of the
handles, the menu and the step buttons are `FloatingWindowLabels` in the
[localization](localization.md), and the close button's is `common.close`.
