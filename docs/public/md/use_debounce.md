# Debounce and throttle

Crate: `libero`
Import: `use libero::hooks::{use_debounced_callback, use_debounced_value, use_throttled_callback, use_throttled_value};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/debounce.rs>
Index: [index.md](index.md) lists every other page
Description: Signals and callbacks that follow their source once it settles or at most once per period.

`use_debounced_value(value: ReadSignal<T>, ms: u64) -> ReadSignal<T>` follows a
signal once it has stopped changing for `ms`: the right copy for a search
request that should not fire per key. `use_throttled_value` takes the same
arguments but follows at once, then at most once per `ms`, ending on the last
value: the right copy for a pointer position or a scroll offset.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Flex, Text, TextField},
    hooks::{use_debounced_callback, use_debounced_value, use_throttled_value},
};

#[component]
fn LiveSearch() -> Element {
    let mut query = use_signal(String::new);
    let settled = use_debounced_value(query.into(), 400);
    let throttled = use_throttled_value(query.into(), 400);

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            TextField {
                label: "Search",
                value: query(),
                oninput: move |next| query.set(next),
            }
            Text { "Typed: {query}" }
            Text { "Debounced: {settled}" }
            Text { "Throttled: {throttled}" }
        }
    }
}

#[component]
fn Autosave() -> Element {
    let mut saved = use_signal(String::new);
    let save = use_debounced_callback(move |draft: String| saved.set(draft), 800);

    rsx! {
        textarea { oninput: move |event| save.call(event.value()) }
        p { role: "status", "Saved: {saved}" }
    }
}
```

## Callbacks

`use_debounced_callback(callback, ms) -> Callback<A>` and
`use_throttled_callback` do the same for a call: the returned callback takes the
argument and runs yours later, with the last one. A throttled callback runs its
first call at once.

## Signals

Pass a signal as `query.into()`. The first value shows at once, and a change
that is undone inside the delay never shows. A change lands from an effect, one
render after its source. It runs on the same timer on the web and natively; a
server render never follows a change.

## API

```rust,ignore
pub fn use_debounced_value<T: Clone + PartialEq + 'static>(value: ReadSignal<T>, ms: u64) -> ReadSignal<T>
pub fn use_throttled_value<T: Clone + PartialEq + 'static>(value: ReadSignal<T>, ms: u64) -> ReadSignal<T>
pub fn use_debounced_callback<A: 'static>(callback: impl FnMut(A) + 'static, ms: u64) -> Callback<A>
pub fn use_throttled_callback<A: 'static>(callback: impl FnMut(A) + 'static, ms: u64) -> Callback<A>
```

| Hook | Runs | Ends on |
|---|---|---|
| `use_debounced_value` | `ms` after the last change | the last value |
| `use_throttled_value` | at once, then at most once per `ms` | the last value |
| `use_debounced_callback` | `ms` after the last call | the last call's argument |
| `use_throttled_callback` | the first call at once, then at most once per `ms` | the last call's argument |

A pending trailing call is dropped when the component unmounts. Call every hook
unconditionally, in the same order every render; the delay is read when a change
arrives, so a new `ms` applies from the next one.

## Accessibility

### Libero handles

- A pending update is dropped when the component unmounts.

### You must

- Announce results that arrive late from a debounced search in a live region
  (`role="status"`); a screen reader user gets no other sign that the list
  changed.
- Keep the field itself bound to the live signal, as the demo does: delaying the
  text a person is typing makes the field lag behind their keys.

### Example

A product search that fetches on `use_debounced_value` of the query, 300 ms
after the last key: the field follows every key, and a `role="status"` line
says "12 results" once the late results arrive.
