# use_notifications

Crate: `libero`
Import: `use libero::components::use_notifications;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/feedback/notifications.rs>
Index: [index.md](index.md) lists every other page
Description: Shows notifications drawn as an Alert, in the host placed once near the root.

`use_notifications() -> NotificationHandle<NotificationData>` shows
notifications drawn as an `Alert`. They appear in the `Notifications` host
placed once near the root. [Notifications](notifications.md) covers the host,
placement, timing and titles.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Button, use_notifications};

#[component]
fn SaveButton() -> Element {
    let notify = use_notifications();

    rsx! {
        Button { variant: "outlined", onclick: move |_| { notify.show("Saved."); }, "Save" }
    }
}
```

`show` takes a message, or a `NotificationData` with a title, a colour and an
icon, and returns an id for `update` and `hide`.

## Accessibility

A notification is announced politely unless its options say `Assertive`.
Showing one never moves focus. F8 focuses the newest one from anywhere.

## API

```rust,ignore
pub fn use_notifications() -> NotificationHandle<NotificationData>
```

| Method | Returns | Description |
|---|---|---|
| `show(args: impl Into<T>)` | `NotificationId` | Queues one with the default options. |
| `show_with(args: impl Into<T>, options: NotificationOptions)` | `NotificationId` | Queues one with its own options. |
| `update(id, args: impl Into<T>)` | `()` | Replaces its data. |
| `hide(id)` | `()` | Closes it. |
| `clear()` | `()` | Removes every notification at once. |

`Copy`.
