# Drag

Crate: `libero`
Import: `use libero::hooks::{Drag, DragMove, DragOptions, DragPoint, DragStart, drag_handle_sx, use_drag};`
Feature: `use-drag` (on by default, turns on `use-element`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/drag.rs>
Index: [index.md](index.md) lists every other page
Description: Pointer plumbing for a drag. Capture, a start point, deltas against it, and one end path for release and cancel.

`use_drag(options: DragOptions) -> Drag` is the pointer plumbing for a drag. It
captures the pointer, keeps the start point and reports each move as a delta
against it. Release and cancel end the same way. It knows no axes and no units,
so convert the delta yourself. `onpointerdown` goes on the grab handle, the
other handlers on the `capture` element. Give the handle `drag_handle_sx()`, or
a touch scrolls the page instead.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::Box,
    hooks::{
        DragMove, DragOptions, DragStart, drag_handle_sx, use_direction, use_drag, use_element,
    },
    sx::sx,
};

#[component]
fn Knob() -> Element {
    let track = use_element();
    let rtl = use_direction().is_rtl();
    // Under RTL the knob starts on the right, so a drag or arrow to the left adds.
    let forward = if rtl { -1.0 } else { 1.0 };
    let mut x = use_signal(|| 0.0);
    let mut from = use_signal(|| 0.0);
    let drag = use_drag(DragOptions {
        capture: track,
        onstart: Callback::new(move |_: DragStart| from.set(x())),
        onmove: Callback::new(move |step: DragMove| {
            x.set((from() + forward * step.delta().x).clamp(0.0, 200.0));
        }),
        onend: Callback::new(|()| {}),
    });

    rsx! {
        Box {
            onmounted: track.mount(),
            onpointermove: move |event| drag.onpointermove.call(event),
            onpointerup: move |event| drag.onpointerup.call(event),
            onpointercancel: move |event| drag.onpointercancel.call(event),
            sx: sx().position("relative").width("232px").height("32px")
                .background("muted.1").border_radius("16px"),
            // A 44 px hit area round the 32 px knob, as a touch target.
            Box {
                onpointerdown: move |event| drag.onpointerdown.call(event),
                tabindex: 0,
                role: "slider",
                aria_label: "Knob position",
                aria_valuemin: 0,
                aria_valuemax: 200,
                aria_valuenow: "{x}",
                aria_valuetext: "{x} px",
                onkeydown: move |event: KeyboardEvent| {
                    let step = match event.key() {
                        Key::Named(NamedKey::ArrowLeft) => -10.0 * forward,
                        Key::Named(NamedKey::ArrowRight) => 10.0 * forward,
                        Key::Named(NamedKey::ArrowDown) => -10.0,
                        Key::Named(NamedKey::ArrowUp) => 10.0,
                        Key::Named(NamedKey::Home) => -200.0,
                        Key::Named(NamedKey::End) => 200.0,
                        _ => return,
                    };
                    event.prevent_default();
                    x.set((x() + step).clamp(0.0, 200.0));
                },
                sx: drag_handle_sx().position("absolute").top("-6px").width("44px").height("44px")
                    .display("grid").place_items("center").border_radius("22px").cursor("grab")
                    .focus_visible(sx().outline_offset("-10px")),
                style: "inset-inline-start: calc({x}px - 6px)",
                Box {
                    aria_hidden: "true",
                    sx: sx().width("32px").height("32px").border_radius("16px")
                        .background("primary.6"),
                }
            }
        }
    }
}
```

## API

```rust,ignore
pub fn use_drag(options: DragOptions) -> Drag
pub fn drag_handle_sx() -> Sx
```

`DragOptions`:

| Field | Type | Description |
|---|---|---|
| `capture` | `ElementHandle` | The element that takes pointer capture and carries the move, up and cancel handlers. |
| `onstart` | `Callback<DragStart>` | The pointer went down. `DragStart` has `client: DragPoint` and `cancel: Callback<()>`. |
| `onmove` | `Callback<DragMove>` | `DragMove` has `start` and `client`, and `delta()`. |
| `onend` | `Callback<()>` | Released or cancelled. Not called after `cancel`. |

`Drag` is `Copy`: `dragging: Signal<bool>`, and the four handlers
`onpointerdown`, `onpointermove`, `onpointerup`, `onpointercancel`, each a
`Callback<Event<PointerData>>`. `DragPoint` has `x` and `y` in client
coordinates. `drag_handle_sx()` sets `touch-action: none`.

## Accessibility

### Libero handles

- A right or middle button never starts a drag, and a second finger is
  ignored.

### You must

- Give anything a drag sets a second way in: a pointer is not a keyboard. The
  demo's knob is a focusable, named slider that takes the arrow keys, Home and
  End. Its `aria-valuetext` says the unit, and under RTL it starts on the right
  with the arrows mirrored.

### Example

A volume knob dragged with `use_drag`: the knob is also a focusable slider
named "Volume", so a keyboard user turns it with the arrows, Home and End.

### Limits

- The hook cancels the press's default, so natively, or to focus something other
  than the pressed tab stop, focus it in `onstart`.
- Blitz and a webview have no pointer capture: Blitz follows the pointer
  instead, and in a webview the drag stops once the pointer leaves the capture
  element.
