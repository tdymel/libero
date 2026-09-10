# Platform

Crate: `libero`
Import: `use libero::platform::{ClockApi, DocumentApi, ElementApi, KeyboardApi, ScrollApi, TimerApi, clock, document, keyboard, scroll, timer};`
Index: [index.md](index.md) - every other component's markdown page
Description: The platform traits and their accessors - timers, document-level keys, scroll, focus and viewport, the clock - each an `Option` that is `None` where the renderer cannot do it.

Everything that reaches the machine underneath dioxus goes through
`libero::platform`: one trait per capability, and one accessor that returns
`Option<&'static dyn Api>`. `None` means the running renderer cannot do it, so a
caller branches once and never unwraps: a feature that has no platform
underneath it is absent, not broken.

The callback-shaped capabilities return a subscription, and dropping it is how
you stop: a dropped timer never fires, a dropped key subscription hears nothing
more. The callback runs outside every scope, so what it writes must be a signal
that outlives the moment.

## The accessors

| Accessor | Methods | `None` where |
|---|---|---|
| `timer() -> Option<&'static dyn TimerApi>` | `after(delay, callback)`, `every(interval, callback)` - each returns a `TimerSubscription`. | Some on every renderer; `None` only outside a dioxus runtime. Inert during a server render. |
| `keyboard() -> Option<&'static dyn KeyboardApi>` | `on_key(callback)`, `on_key_unfiltered(callback)` - each returns a `KeySubscription`. The callback gets a `KeyChord` and returns `true` to prevent the default. | Off the web. |
| `scroll() -> Option<&'static dyn ScrollApi>` | `on_scroll(callback)` - anything scrolling, not just the page. Returns a `ScrollSubscription`. | Off the web. |
| `document() -> Option<&'static dyn DocumentApi>` | `active_element()`, `viewport()`. | Where the renderer exposes no document: a webview, and any headless build. |
| `clock() -> Option<&'static dyn ClockApi>` | `today()` - the local calendar day. | Some on every renderer: JS `Date` on the web, the system clock and time zone off it. Call it after mount, never while rendering. |

## A subscription

Press G anywhere on the page but in a text field - `on_key` never hears a key
the user is typing. Never return `true` for Tab or Shift+Tab: the listener runs
before everything else, and keyboard focus would have nowhere to go.

```rust
use dioxus::prelude::*;
use libero::{
    components::Text,
    platform::{KeyChord, keyboard},
};

#[component]
fn GoCounter() -> Element {
    let presses = use_signal(|| 0);
    // Dropping the subscription unsubscribes, so the signal holds it for
    // as long as the component lives. Off the web it is None, and the
    // shortcut is simply not there.
    let mut shortcut = use_signal(move || {
        keyboard().map(|keyboard| {
            keyboard.on_key(Box::new(move |chord: KeyChord| {
                let hit = chord.key == Key::Character("g".into()) && !chord.repeat;
                if hit {
                    let mut presses = presses;
                    presses += 1;
                }
                hit
            }))
        })
    });
    use_drop(move || shortcut.set(None));

    rsx! {
        Text { "G pressed {presses} times" }
    }
}
```

`on_key_unfiltered` hears the presses `on_key` drops, those inside a text field
too. Use it for one chord, Escape being the usual one, and return `false` to
everything else.

## Elements

An element has no accessor, because there is no portable way to name one.
`use_element()` returns a handle you mount with `onmounted: handle.mount()`,
and the handle implements `ElementApi`. Commands - `focus()`, `scroll_to()`,
`set_pointer_capture()` - return at once. Reads - `dimensions()`,
`client_offset()`, `scroll_offset()` - are futures: start one in the event
handler and await it in a `spawn`. What a renderer cannot serve answers
`PlatformError::Unsupported` rather than being missing from the type.

## API

### `TimerApi`

| Method | Returns | Description |
|---|---|---|
| `after(delay: Duration, callback: Box<dyn FnOnce()>)` | `Box<dyn TimerSubscription>` | Calls `callback` once, `delay` from now, unless the subscription is dropped first. |
| `every(interval: Duration, callback: Box<dyn Fn()>)` | `Box<dyn TimerSubscription>` | Calls `callback` every `interval` until the subscription is dropped. |

### `KeyboardApi`

| Method | Returns | Description |
|---|---|---|
| `on_key(callback: Box<dyn Fn(KeyChord) -> bool>)` | `Box<dyn KeySubscription>` | Every press in the document except one aimed at a text input, a `textarea`, a `select` or a `contenteditable`. `true` prevents the default. |
| `on_key_unfiltered(callback: Box<dyn Fn(KeyChord) -> bool>)` | `Box<dyn KeySubscription>` | Every press, wherever it landed. |

`KeyChord` has `key: Key`, `modifiers: Modifiers` and `repeat: bool` - whether
the platform is repeating a held key. A toggle should ignore a repeat.

### `ScrollApi`

| Method | Returns | Description |
|---|---|---|
| `on_scroll(callback: Box<dyn Fn()>)` | `Box<dyn ScrollSubscription>` | Called whenever anything scrolls. No coordinates: re-measure in the callback. |

### `DocumentApi`

| Method | Returns | Description |
|---|---|---|
| `active_element()` | `Option<Box<dyn ElementApi>>` | Whatever has focus. Read from an event handler, it is the element the user acted on. |
| `viewport()` | `Read<Dimensions>` | The visible viewport, in CSS pixels. |

### `ClockApi`

| Method | Returns | Description |
|---|---|---|
| `today()` | `NaiveDate` | The local calendar day. |

### `ElementApi`

Reached through `use_element()`, or a query off another `ElementApi`; never
stored.

| Method | Returns | Description |
|---|---|---|
| `focus()`, `blur()`, `click()` | `Result<(), PlatformError>` | Commands. |
| `reset()`, `request_submit()` | `Result<(), PlatformError>` | A `<form>`'s own reset and submit. |
| `is_focused()`, `is_connected()` | `bool` | Synchronous. A renderer that cannot tell `is_connected` answers `true`. |
| `dimensions()`, `scroll_size()`, `natural_size()` | `Read<Dimensions>` | Rendered size, scrollable size, an `<img>`'s intrinsic size. |
| `client_offset()`, `scroll_offset()` | `Read<(f64, f64)>` | Top-left in viewport coordinates, and the scroll offset. |
| `computed_px(property)` | `Read<Option<f64>>` | A CSS property's computed value in pixels; `None` when it is not a length. Web only. |
| `scroll_to(x, y)` | `Result<(), PlatformError>` | Sets the scroll offset. |
| `scroll_into_view(smooth: bool)` | `Result<(), PlatformError>` | Scrolls the nearest scrollable ancestor just far enough. |
| `set_files(files: &[FileData])` | `Result<(), PlatformError>` | Replaces an `<input type="file">`'s file list. Web only. |
| `set_pointer_capture(pointer_id: i32)` | `Result<(), PlatformError>` | Routes that pointer's events here until release. |
| `query_selector(selector)`, `query_selector_all(selector)` | `Result<Box<dyn ElementApi>, PlatformError>`, `Result<Vec<..>, PlatformError>` | Descendants, in DOM order. |

`Read<T>` is `Pin<Box<dyn Future<Output = Result<T, PlatformError>>>>`.
