# use_notifications_with

Crate: `libero`
Import: `use libero::components::use_notifications_with;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/feedback/notifications.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Shows notifications of your own data type, drawn by your own template.

`use_notifications_with(template) -> NotificationHandle<T>` shows
notifications of your own data type, drawn by your own template. The host
keeps the list, the live region, the timers and the hover pause.
[Notifications](notifications.md) has the full story.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, NotificationScope, Paper, Text, use_notifications_with},
    sx::sx,
};

#[derive(Clone, PartialEq)]
struct Reminder {
    text: &'static str,
}

// A `fn`, not a capturing closure: everything it draws travels in `Reminder`.
fn reminder(s: NotificationScope<Reminder>) -> Element {
    rsx! {
        Paper { shadow: "md", sx: sx().padding("md"),
            Flex { direction: "row", justify: "space-between", gap: "sm",
                Text { "{s.args().text}" }
                if s.closable() {
                    Button { variant: "text", onclick: move |_| s.close(), "Dismiss" }
                }
            }
        }
    }
}

#[component]
fn RemindMe() -> Element {
    let reminders = use_notifications_with(reminder);

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| { reminders.show(Reminder { text: "Stand up and stretch." }); },
            "Remind me"
        }
    }
}
```

The template is a `fn`, not a closure that captures. A notification outlives
the component that raised it, so everything it draws travels in `T`.
`update(id, value)` redraws one with new data, such as a progress bar.

## Accessibility

The template draws the content only. Give it a close control when
`s.closable()` says so, because not every reader can wait for the timer.

## API

```rust,ignore
pub fn use_notifications_with<T: 'static>(
    template: fn(NotificationScope<T>) -> Element,
) -> NotificationHandle<T>
```

The handle is the one [use_notifications](use_notifications.md) returns.
`NotificationScope<T>` has `args()`, `close()`, `closable()` and `id()`, and is
`Copy`.
