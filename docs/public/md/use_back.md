# Back button

Crate: `libero`
Import: `use libero::hooks::use_back;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/dismiss.rs>
Index: [index.md](index.md) lists every other page
Description: Runs a handler on Android's Back button instead of leaving the app, below any overlay opened later; does nothing elsewhere.

`use_back(active, onback)` runs `onback` when Android's Back button is
pressed, instead of leaving the app, while `active` is true. Use it to step
back through a wizard, undo, or leave an editing mode. The newest active hook
or overlay takes the press. On the web and the desktop it does nothing.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text},
    hooks::use_back,
};

#[component]
fn Steps() -> Element {
    let mut step = use_signal(|| 1);
    // On Android, Back goes one step back while there is one; at the first, it leaves the app.
    use_back(step() > 1, Callback::new(move |()| step -= 1));

    rsx! {
        Flex { direction: "column", gap: "sm", align: "flex-start",
            Text { "Step {step} of 3" }
            Button { onclick: move |_| step += 1, "Next" }
        }
    }
}
```

## API

```rust,ignore
pub fn use_back(active: bool, onback: Callback<()>)
```

| Platform | Support |
|---|---|
| Android (WebView) | Back runs `onback`, checked by the e2e suite on the emulator. |
| Web, desktop, Blitz | Does nothing. |

## Accessibility

### Libero handles

- A `Modal`, `Drawer`, menu, popover, open field list or fullscreen opened
  after your hook takes Back first, as Escape closes the newest layer first.

### You must

- Keep a visible control for the same step, such as a Previous button: web,
  desktop and keyboard users have no Back button.

### Limits

- Android only. Elsewhere the hook does nothing.
- Activated without a tap (on mount, from a timer), the first Back may still
  leave the app: Android's WebView skips a history entry pushed without a user
  gesture.
