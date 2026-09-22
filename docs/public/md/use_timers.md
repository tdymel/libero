# use_timeout, use_interval

Crate: `libero`
Import: `use libero::hooks::{IntervalHandle, TimeoutHandle, use_interval, use_timeout};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/timers.rs>
Index: [index.md](index.md) lists every other page
Description: Runs a callback once or on a period, started and stopped from code, cancelled when the component unmounts.

`use_timeout(callback, ms) -> TimeoutHandle` runs a callback once, `ms` after
`start()`. `use_interval(callback, ms) -> IntervalHandle` runs it every `ms`
between `start()` and `stop()`. Neither starts itself: call `start()` from a
handler, or from `use_hook` to run from mount. `start()` on a running timer
restarts it, `toggle()` flips an interval, and `pending()` and `active()` are
reactive.

One timer serves the web, Blitz and a webview. The callback runs in your
component's scope, so it may write signals, spawn or read elements. A server
render never fires it. Natively every interval tick costs a thread, so keep the
period to a second or more.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text},
    hooks::{use_interval, use_timeout},
};

#[component]
fn Stopwatch() -> Element {
    let mut seconds = use_signal(|| 0);
    let mut saved = use_signal(|| false);
    let interval = use_interval(move || seconds += 1, 1000);
    let flash = use_timeout(move || saved.set(false), 2000);

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Text { "{seconds} s" }
            Flex { gap: "sm",
                Button {
                    onclick: move |_| interval.toggle(),
                    if interval.active() {
                        "Stop"
                    } else {
                        "Start"
                    }
                }
                Button {
                    variant: "outlined",
                    onclick: move |_| {
                        saved.set(true);
                        flash.start();
                    },
                    "Save lap"
                }
            }
            div { role: "status",
                if saved() {
                    "Lap saved"
                }
            }
        }
    }
}
```

## Accessibility

### Libero handles

- Both cancel when the component unmounts, so a callback never runs into a
  screen the reader has left.

### You must

- Announce what a timer changes with a live region (`role="status"`), as the
  demo does for "Lap saved": a sighted user sees it appear, a screen reader
  hears nothing otherwise.
- Give the reader a way to stop anything that moves on its own for more than
  five seconds (WCAG 2.2.2). A `use_interval` that starts itself needs a Stop
  control.

### Limits

- A tick that lands while the component is still rendering the previous one is
  skipped, so a callback that counts should read a clock, not add one per tick,
  if the count must stay exact.

## API

```rust,ignore
pub fn use_timeout(callback: impl FnMut() + 'static, ms: u64) -> TimeoutHandle
pub fn use_interval(callback: impl FnMut() + 'static, ms: u64) -> IntervalHandle
```

| Method | Returns | Description |
|---|---|---|
| `TimeoutHandle::start()` | `()` | Starts the countdown, or restarts one under way. |
| `TimeoutHandle::stop()` | `()` | Cancels a countdown; the callback does not run. |
| `TimeoutHandle::pending()` | `bool` | Whether a countdown is under way. Reactive. |
| `IntervalHandle::start()` | `()` | Starts ticking, or restarts the period of one under way. |
| `IntervalHandle::stop()` | `()` | Stops ticking. |
| `IntervalHandle::toggle()` | `()` | Stops a running interval, starts a stopped one. |
| `IntervalHandle::active()` | `bool` | Whether it is ticking. Reactive. |

Both handles are `Copy`. The callback of the newest render is the one that
runs, so it sees current state.
