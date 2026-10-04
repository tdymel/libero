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
| `sx` | `Input<Sx>` | - | Styles the window. `min_width`, `max_width`, `min_height` and `max_height` here limit a resize. Unset, the minimum is 12rem by 6rem, room for the title bar and Close. The window never grows past the viewport. |
| `onmove` | `Option<Callback<WindowRect>>` | - | Called after a drag, a keyboard or button move, or a Reset, in viewport pixels. |
| `onresize` | `Option<Callback<WindowRect>>` | - | Called after a resize by pointer, keyboard or button, or a Reset. |
| `parts` | `Input<Parts<FloatingWindowPart>>` | - | Styles for the inner parts in the Style API tab, under `sx`: `Parts::new().part(FloatingWindowPart::Body, sx().padding("lg"))`. |
| `menu_parts` | `Input<Parts<MenuPart>>` | - | The title-bar menu's `parts`, the `Menu` page's Style API table. The menu opens in a portal, out of `parts`; `FloatingWindowPart::Menu` styles its button. |

`WindowRect { x, y, width, height }` is in viewport pixels.

## Style API

Style a part with the `parts` option, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `FloatingWindowPart::TitleBar` | `title-bar` | The row holding the move handle, the menu button and the close button. |
| `FloatingWindowPart::Handle` | `handle` | The move handle around the title. |
| `FloatingWindowPart::Title` | `title` | The title heading. |
| `FloatingWindowPart::Menu` | `menu` | The Move, Resize and Reset menu's button. The menu itself is portaled: style it with `menu_parts`. |
| `FloatingWindowPart::Close` | `close` | The close button. |
| `FloatingWindowPart::Steps` | `steps` | The step buttons Move or Resize shows. |
| `FloatingWindowPart::Body` | `body` | The scrolling content. |
| `FloatingWindowPart::Resize` | `resize` | The corner resize handle, when `resizable`. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | Reaches the title bar. |
| `Left`, `Up`, `Right` or `Down` | On the title bar: moves the window 10px. |
| `Shift+Left`, `Shift+Up`, `Shift+Right` or `Shift+Down` | On the title bar: moves the window 1px. |
| `Left`, `Up`, `Right` or `Down` | On the resize handle: resizes the window. |
| `Home` or `End` | On the resize handle: asks for the smallest or largest size allowed. |
| `Escape` | Hides the Move or Resize step buttons. Otherwise closes the window and returns focus to its trigger. |
| `F6` | Moves focus between the page and the topmost window. |

### Libero handles

- A window takes focus when it opens.
- The title bar's menu offers Move, Resize and Reset. Move and Resize show
  step buttons, one click per step, so neither needs a drag. Done or Escape
  hides them. Their group is named "Move window by steps" or "Resize window by
  steps", apart from the handles.

### You must

- Pick a `placement` that does not cover the page's controls: the page behind
  a window still takes Tab.

### Example

An inspector window with the title "Inspector": it takes focus when it opens,
the arrows on its title bar move it, F6 goes back to the page, and Escape
closes it and returns focus to its trigger.

### Limits

- In a desktop WebView or on Android, F6 does not move focus, and the Move and
  Resize step buttons do not take focus when they appear: Tab reaches them.

## Theme defaults

`FloatingWindowDefaults` on the theme: `placement`, `radius`, `shadow`,
`move_step` and `resize_step`. The chrome is `Paper`'s. The names of the
handles, the menu and the step buttons are `FloatingWindowLabels` in the
[localization](localization.md), and the close button's is `common.close`.
