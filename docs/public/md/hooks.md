# Hooks

Crate: `libero`
Import: `use libero::hooks::{Drag, DragMove, DragOptions, DragStart, drag_handle_sx, use_clipboard, use_drag, use_element, use_focus_return, use_id, use_portal, use_presence, use_root_id};`
Index: [index.md](index.md) - every other component's markdown page
Description: The public hooks libero's components are built from - `use_drag` for pointer drags, `use_clipboard` for copying, and `use_root_id` for ids that respect a caller's own.

The hooks below are what libero's own components are built from, and they are
public for yours. Like every dioxus hook they are positional: call them
unconditionally, in the same order every render. The overlay hooks have pages of
their own: [`use_popover`](popover.md), [`use_modal`](modal.md),
[`use_drawer`](drawer.md), [`use_lightbox`](lightbox.md) and
[`use_floating_window`](floating_window.md).

## use_drag

Pointer plumbing for a drag: capture, the start point, a delta against it, and
one end path for both release and cancel. It knows no axes and no units -
convert the delta yourself. `onpointerdown` goes on the grab handle, the other
three on the `capture` element, which owns the geometry. Measure in `onstart`,
and call its `cancel` to refuse the drag. Give the handle `drag_handle_sx()`, or
a touch scrolls the page and never moves.

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
                .background("grey.1").border_radius("16px"),
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
                    // The fill publishes its contrast colour, white, for what is
                    // drawn *on* it - which overrides the ring's stripe and
                    // leaves it the same white as the halo, so the ring drawn
                    // outside vanishes rather than reading against itself.
                    // Inset it into the fill: 4.86:1 rather than 1.11:1.
                    .focus_visible(sx().outline_offset("-4px")),
                style: "left: {x}px",
            }
        }
    }
}
```

A pointer is not a keyboard, so anything a drag sets needs a second way in. The
knob is a focusable, named slider that takes the arrow keys, Home and End.

## use_clipboard

`copy(text)` writes to the clipboard, and `copied()` turns true once the
platform confirms the write - a denied permission leaves it false and logs a
warning. The flag stays up until you call `reset()`. Say the result in a status
region that is already mounted; a button whose own label changes is not
announced.

```rust
use dioxus::prelude::*;
use libero::{components::Button, hooks::use_clipboard};

#[component]
fn CopyLink() -> Element {
    let mut clipboard = use_clipboard();

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| clipboard.copy("https://github.com/tdymel/libero"),
            onblur: move |_| clipboard.reset(),
            "Copy link"
        }
        // Mounted before it has anything to say, so the change is announced.
        span { role: "status", if clipboard.copied() { "Copied" } }
    }
}
```

## use_root_id

An id for a component that also spreads the caller's `attributes`: the caller's
own `id` when it passed one, a generated one otherwise. Build the ids of the
inner parts on it, so the aria wiring still holds when the caller names the
root. `use_id()` alone would render a second, different `id`, and the browser
keeps the caller's.

```rust
use dioxus::prelude::*;
use libero::hooks::use_root_id;

#[component]
fn Disclosure(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    // The caller's `id` if it gave one, a generated one otherwise.
    let id = use_root_id(&attributes);
    let mut open = use_signal(|| false);

    rsx! {
        div { id: "{id}", ..attributes,
            button {
                aria_controls: "{id}-panel",
                aria_expanded: "{open}",
                onclick: move |_| open.toggle(),
                "Details"
            }
            div { id: "{id}-panel", hidden: !open(), {children} }
        }
    }
}
```

## Other hooks

| Hook | What it is for | Used on |
|---|---|---|
| `use_element() -> ElementHandle` | A handle to one of the component's own elements, mounted with `onmounted: handle.mount()`. It implements `ElementApi`. | [Platform](platform.md) |
| `use_id() -> Signal<String>` | A process-unique id, stable for the component's lifetime, for the aria wiring between one instance's parts. | [Popover](popover.md) |
| `use_portal(content: Option<Element>)` | Renders the content at the document root, out of any clipping or stacking ancestor. `None` takes it away. | [Float](float.md) |
| `use_presence(open, property) -> Presence` | Keeps closing content mounted until its exit transition on `property` ends: `mounted()` and `visible()` drive the markup, `on_mounted()` and `on_transition_end(event)` go on the element. | [Collapse](collapse.md) |
| `use_focus_return() -> FocusReturn` | Remembers where focus came from, with `remember(event)` on a trigger's `onmounted` or `remember_active()` as it opens, and `restore()` puts it back - onto `fallback(handle)` if the trigger is gone. | [Collapse](collapse.md) |

## API

### `use_drag`

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
`Callback<Event<PointerData>>`. A right or middle button never starts a drag,
and a second pointer is ignored.

### `use_clipboard`

```rust,ignore
pub fn use_clipboard() -> Clipboard
```

| Method | Returns | Description |
|---|---|---|
| `copy(text: impl Into<String>)` | `()` | Writes `text`; `copied()` flips once the platform confirms. |
| `copied()` | `bool` | Whether the last write succeeded and nothing has reset it. Reactive. |
| `reset()` | `()` | Clears the flag. |

`Clipboard` is `Copy`. Where the target has no clipboard, `copy` logs a warning
and does nothing.

### `use_root_id`

```rust,ignore
pub fn use_root_id(attributes: &[Attribute]) -> Signal<String>
```

The first `id` in `attributes`, or a process-unique `lsx-N`. Follows the caller's
`id` if it changes.

### `use_element`

```rust,ignore
pub fn use_element() -> ElementHandle
```

| Method | Returns | Description |
|---|---|---|
| `mount()` | `impl FnMut(Event<MountedData>)` | The `onmounted` handler that fills the handle in. |
| `is_mounted()` | `bool` | Reactive: an effect reading it re-runs once the element mounts. |

`ElementHandle` is `Copy` and implements `ElementApi` - see [platform.md](platform.md).
Every call answers `PlatformError::Unsupported` until the element is mounted.

### `use_id`

```rust,ignore
pub fn use_id() -> Signal<String>
```

A process-unique `lsx-N`, stable for the component's lifetime.

### `use_portal`

```rust,ignore
pub fn use_portal(content: Option<Element>)
```

Takes already-rendered content, not a closure. Deregisters on drop. The portaled
content inherits no context from the call site; provide what it needs inside it.

### `use_presence`

```rust,ignore
pub fn use_presence(open: bool, property: &'static str) -> Presence
```

| Method | Returns | Description |
|---|---|---|
| `mounted()` | `bool` | Whether to render the content at all - true until the exit ends. |
| `visible()` | `bool` | Whether to render it in its open state. |
| `on_mounted()` | `()` | Call from the content's `onmounted`, so the entry transition runs. |
| `on_transition_end(event: &Event<TransitionData>)` | `()` | Call from its `ontransitionend`; unmounts once `property`'s exit ends. |

`property` is the CSS property carrying the exit transition. The closed state
must also hide the content from the accessibility tree (`visibility: hidden` or
`inert`): until the exit ends it is still mounted.

### `use_focus_return`

```rust,ignore
pub fn use_focus_return() -> FocusReturn
```

| Method | Returns | Description |
|---|---|---|
| `remember(event: Event<MountedData>)` | `()` | Use as the trigger's `onmounted`. |
| `remember_active()` | `()` | Remembers whatever has focus now. Call it synchronously in the handler that opens the overlay. |
| `fallback(element: ElementHandle)` | `()` | Where focus goes if the remembered element is gone. Appends: call it once per tier, nearest first. |
| `restore()` | `()` | Puts focus back. |
