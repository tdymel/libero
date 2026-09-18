# FloatingWindow

Crate: `libero`
Import: `use libero::{components::{FloatingWindowOptions, WindowRect}, hooks::{use_floating_window, FloatingWindowHandle}};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/use_floating_window.rs>
Index: [index.md](index.md) lists every other page
Description: A non-modal window over the page that drags, moves by keyboard and resizes from a corner, opened through a hook.

A non-modal window over the page, with a title bar that drags, an optional
resize corner, a menu that moves and resizes it without a drag, and a close
button. `use_floating_window` returns a `Copy` handle that opens and closes it.
There is no overlay or focus trap, so the page stays usable. A drag re-renders
the whole window, so keep its body shallow.

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

## Accessibility

A window takes focus when it opens. Tab reaches the title bar, where `←` `↑` `→`
`↓` move it 10px and Shift + arrow 1px. On the resize handle the arrows resize,
and Home End ask for the smallest and largest size allowed. The title bar's menu
offers Move, Resize and Reset. Move and Resize show step buttons, one click per
step, so neither needs a drag. Done or Esc hides them. Esc closes the window and
returns focus to its trigger. F6 moves focus between the page and the topmost
window. The page behind a window still takes Tab, so pick a placement that does
not cover its controls.

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

## Theme defaults

`FloatingWindowDefaults` on the theme: `placement`, `radius`, `shadow`,
`move_step` and `resize_step`. The chrome is `Paper`'s. The names of the
handles, the menu and the step buttons are `FloatingWindowLabels` in the
[localization](localization.md), and the close button's is `common.close`.
