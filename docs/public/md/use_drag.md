# use_drag

Crate: `libero`
Import: `use libero::hooks::{Drag, DragMove, DragOptions, DragPoint, DragStart, drag_handle_sx, use_drag};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/drag.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Pointer plumbing for a drag. Capture, a start point, deltas against it, and one end path for release and cancel.

`use_drag(options: DragOptions) -> Drag` is the pointer plumbing for a drag. It
captures the pointer, keeps the start point and reports each move as a delta
against it. Release and cancel end the same way. It knows no axes and no units,
so convert the delta yourself.

The returned `Drag` holds four handlers and a `dragging` signal. `onpointerdown`
goes on the grab handle, the other three on the `capture` element, which owns
the geometry. Measure in `onstart`, and call its `cancel` to refuse the drag.
Give the handle `drag_handle_sx()`, or a touch scrolls the page and the handle
never moves.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::Box,
    hooks::{DragMove, DragOptions, DragStart, drag_handle_sx, use_drag, use_element},
    sx::sx,
};

#[component]
fn Knob() -> Element {
    let track = use_element();
    let mut x = use_signal(|| 0.0);
    let mut from = use_signal(|| 0.0);
    let drag = use_drag(DragOptions {
        capture: track,
        onstart: Callback::new(move |_: DragStart| from.set(x())),
        onmove: Callback::new(move |step: DragMove| {
            x.set((from() + step.delta().x).clamp(0.0, 200.0));
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
            Box {
                onpointerdown: move |event| drag.onpointerdown.call(event),
                // The keyboard path a drag needs: focusable, named, and on the arrows.
                tabindex: 0,
                role: "slider",
                aria_label: "Knob position",
                aria_valuemin: 0,
                aria_valuemax: 200,
                aria_valuenow: "{x}",
                onkeydown: move |event: KeyboardEvent| {
                    let next = match event.key() {
                        Key::ArrowLeft | Key::ArrowDown => x() - 10.0,
                        Key::ArrowRight | Key::ArrowUp => x() + 10.0,
                        Key::Home => 0.0,
                        Key::End => 200.0,
                        _ => return,
                    };
                    event.prevent_default();
                    x.set(next.clamp(0.0, 200.0));
                },
                sx: drag_handle_sx().position("absolute").width("32px").height("32px")
                    .border_radius("16px").background("primary.6").cursor("grab")
                    // Inset the focus ring: outside a filled knob it turns white on white.
                    .focus_visible(sx().outline_offset("-4px")),
                style: "left: {x}px",
            }
        }
    }
}
```

## Accessibility

A pointer is not a keyboard, so anything a drag sets needs a second way in. The
knob is a focusable, named slider that takes the arrow keys, Home and End. A
right or middle button never starts a drag, and a second finger is ignored.

## Web and native

The hook cancels the press's default, so on the web it focuses the pressed tab
stop itself. Natively, or to focus something else, focus it in `onstart`. Blitz
and a webview have no pointer capture; Blitz follows the pointer instead, and in
a webview the drag stops once the pointer leaves the capture element.

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
