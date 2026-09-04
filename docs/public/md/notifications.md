# Notifications

Crate: `libero`
Import: `use libero::components::{Notifications, NotificationData, NotificationOptions, use_notifications};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/feedback/notifications.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A hook plus a host. Render `Notifications {}` once, and `use_notifications()` shows messages from anywhere, as an `Alert` or as your own template over your own data.

Notifications are a hook plus a host. Render `Notifications {}` once near the
root. `use_notifications()` then returns a `Copy` handle that shows
notifications from any handler, task or child. A notification lives in a store
at the app's root, so it outlives the component that raised it.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{NotificationData, NotificationOptions, Notifications, Placement, use_notifications},
    theme::AutoClose,
};

// Once, near the root - the one outlet for every handle.
fn App() -> Element {
    rsx! {
        LiberoProvider {
            Router::<Route> {}
            Notifications {}
        }
    }
}

// Anywhere below it.
#[component]
fn SaveButton() -> Element {
    let notify = use_notifications();

    rsx! {
        Button {
            onclick: move |_| {
                notify.show("Saved.");
                notify.show(NotificationData {
                    title: Some("Upload failed".into()),
                    message: "archive.zip is over the 10 MB limit.".into(),
                    color: "error".into(),
                    ..Default::default()
                });
                notify.show_with("Copied", NotificationOptions {
                    position: Some(Placement::TopCenter),
                    auto_close: Some(AutoClose::After(2000)),
                    ..Default::default()
                });
            },
            "Save"
        }
    }
}
```

Render the host **once**. It is the one outlet for every handle, so a second
host would draw every notification twice. It is portaled, so its place in the
tree does not matter. Without a host, `show` queues notifications that nobody
draws.

## A contained host

`Notifications { contained: true, children }` is a host for one region of the
page instead of the window. It draws its nine stacks inside its own
`position: relative` box, around its children, and it gives the handles
created below it a queue of their own: a `use_notifications()` inside it shows
its notifications there, not in the app's host. The docs preview is one.

```rust
Notifications { contained: true, position: "top-end",
    SaveButton {} // its `use_notifications()` shows them in this box
}
```

`contained` is read once, when the host mounts. When the host unmounts, its
queue goes with it, and a handle that outlives it does nothing.

## Your own template

`use_notifications_with` takes your own data type and a template that draws it.
The template draws the content, and the host keeps the list item, the live
region, the timers and the hover pause.

```rust
#[derive(Clone, PartialEq)]
struct Upload {
    file: &'static str,
    percent: f64,
}

fn upload_notification(s: NotificationScope<Upload>) -> Element {
    let upload = s.args();
    rsx! {
        Paper { shadow: "md", sx: sx().padding("md"),
            Text { "Uploading {upload.file}" }
            ProgressBar { value: upload.percent, "aria-label": "{upload.file}" }
            ActionIcon { aria_label: "Dismiss", onclick: move |_| s.close(), DismissIcon {} }
        }
    }
}

let uploads = use_notifications_with(upload_notification);
let id = uploads.show_with(
    Upload { file: "archive.zip", percent: 0.0 },
    NotificationOptions { auto_close: Some(AutoClose::Never), ..Default::default() },
);
uploads.update(id, Upload { file: "archive.zip", percent: 40.0 });
```

**The template is a `fn`, not a closure that captures.** A notification outlives
the component that raised it, so a captured signal or handler could be dropped
while the notification still draws with it. Everything the template needs
travels in `T`. A closure that captures nothing coerces to the `fn`. The
template is called in its notification's own scope on every render, so it may
call hooks, as long as it calls the same ones every time. That scope redraws
when `update` changes its data, and no other one does.

The store is type-erased. Each `T` adds one small closure, and the queue, the
timers and the host compile once.

## Queue, limit and timing

- **`show` queues. It does not promise that the notification is visible.** Each
  stack shows at most `limit` notifications (5 by default). The rest wait in
  order, and a waiting notification's timer starts only once it is shown.
- A notification closes after `auto_close`: 4 seconds by default,
  `AutoClose::Never` to stay, or `AutoClose::After(ms)`.
- **Hovering or focusing any notification pauses every timer.** When the
  pointer or the focus leaves, each timer starts over with its full time. This
  is the WCAG 2.2.1 (Timing Adjustable) mechanism. It pauses on focus too, so a
  close button you tabbed to does not disappear.
- Closing runs a short fade (`transition_duration`, 200 ms). The notification
  leaves the accessibility tree when the fade ends, then it is removed. Under
  `prefers-reduced-motion` there is no animation.

## Accessibility

- Each of the nine stacks is an `ol` pair: one `aria-live="polite"` and one
  `aria-live="assertive"`. **All eighteen are always mounted, even when empty**,
  because a live region has to be in the document before content is added to
  it, or nothing is announced. `NotificationOptions::live` picks the region. It
  is `Polite` unless you say otherwise, and it is never derived from a colour.
- The default template is an `Alert` with `role="group"`, not `role="alert"`.
  The list around it is already the live region, and a live region nested in
  another can be announced twice.
- **Focus is never moved**, on show or on close (WCAG 2.4.3). A close button is
  a real `<button>`, reached by Tab in document order. Escape does nothing,
  because nothing here takes focus. After you close the notification that held
  focus, focus falls back to the page.
- An assertive notification sits above the polite ones in the same stack.

## API

### `use_notifications` / `use_notifications_with`

```rust
pub fn use_notifications() -> NotificationHandle<NotificationData>
pub fn use_notifications_with<T: 'static>(
    template: fn(NotificationScope<T>) -> Element,
) -> NotificationHandle<T>
```

### `NotificationHandle<T>`

| Method | Returns | Description |
|---|---|---|
| `show(args: impl Into<T>)` | `NotificationId` | Queues one with the default options. |
| `show_with(args: impl Into<T>, options: NotificationOptions)` | `NotificationId` | Queues one with its own options. |
| `update(id, args: impl Into<T>)` | `()` | Replaces its data. It keeps its place and its timer. Does nothing once it is gone. |
| `hide(id)` | `()` | Closes it with its exit. A queued one is removed at once. |
| `clear()` | `()` | Removes every notification, from every template, at once. |

`Copy`.

### `NotificationScope<T>`

| Method | Returns | Description |
|---|---|---|
| `args()` | `T` | The data it was shown with, or last updated to. Needs `T: Clone`. |
| `close()` | `()` | Starts its exit. |
| `closable()` | `bool` | `NotificationOptions::closable`: whether to draw a close control. |
| `id()` | `NotificationId` | Its id. |

`Copy`, so several handlers in one template can each hold it.

### `NotificationData`

| Field | Type | Description |
|---|---|---|
| `title` | `Option<String>` | The `Alert`'s title and accessible name. |
| `message` | `String` | The message. Empty renders no message slot. |
| `color` | `Input<ThemeAwareValue>` | The `Alert`'s colour; unset is `theme.alert.color`. |
| `variant` | `Input<ButtonVariant>` | The `Alert`'s variant; unset is `theme.alert.variant`. |
| `icon` | `Option<Element>` | A glyph. It is drawn by the host later, so it must not carry event handlers. |

`From<&str>` and `From<String>` fill `message`.

### `NotificationOptions`

| Field | Type | Default | Description |
|---|---|---|---|
| `position` | `Option<Placement>` | `None` | The stack. `None` is the host's `position`. |
| `auto_close` | `Option<AutoClose>` | `None` | `None` is the host's `auto_close`. |
| `closable` | `bool` | `true` | Whether the template draws a close control. |
| `live` | `NotificationLive` | `Polite` | `Polite` or `Assertive`: which live region announces it. |

## Props

`Notifications`, the host:

| Prop | Type | Default | Description |
|---|---|---|---|
| `position` | `Placement` | `bottom-end` | The stack a notification joins unless it names its own. |
| `limit` | `Option<usize>` | `5` | Shown at once per stack. |
| `auto_close` | `Option<AutoClose>` | `After(4000)` | Unless a notification says otherwise. |
| `contained` | `bool` | `false` | Draw the stacks in this host's own box, and give the handles below it a queue of their own. Read once, at mount. |
| `children` | `Option<Element>` | - | Rendered inside a contained host, before its stacks. |

## Theme defaults

`NotificationDefaults` on the theme, as `notification`.

| Field | Type | Description |
|---|---|---|
| `position` | `Placement` | `BottomEnd`. |
| `auto_close` | `AutoClose` | `After(4000)`. |
| `limit` | `usize` | `5`. |
| `width` | `&'static str` | `360px`, capped at the viewport's (or the contained host's) width minus both offsets. |
| `gap` | `Size` | `Sm`, between two notifications. |
| `offset` | `Size` | `Md`, from the viewport's (or the contained host's) edge. |
| `transition_duration` | `u32` | `200` ms, the entry and the exit. |
| `close_label` | `&'static str` | `Close`, the default template's close button. |

The stacks sit on `z_index.notification` (2100), above modals and the
dropdowns opened inside them.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-notification-width` | From `NotificationDefaults::width`. |
| `--lsx-notification-gap` | From `NotificationDefaults::gap`. |
| `--lsx-notification-offset` | From `NotificationDefaults::offset`. |
| `--lsx-notification-transition` | From `NotificationDefaults::transition_duration`. |
| `--lsx-z-index-notification` | From `ZIndexDefaults::notification`. |

## Data attributes

| Attribute | On |
|---|---|
| `data-state="leaving"` | A notification's `li` while it fades out. |
