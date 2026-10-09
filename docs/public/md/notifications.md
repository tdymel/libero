# Notifications

Crate: `libero`
Import: `use libero::components::{Notifications, NotificationData, NotificationOptions, SnackbarData, snackbar, use_notifications};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/feedback/notifications/notifications.rs>
Index: [index.md](index.md) lists every other page
Description: A hook plus a host: render `Notifications {}` once, and `use_notifications()` shows messages from anywhere, as an `Alert` or your own template over your data.

A hook and a host. Render `Notifications {}` once near the root.
`use_notifications()` then returns a handle that shows notifications from
anywhere, and each one outlives the component that raised it.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        Button, NotificationData, NotificationOptions, Notifications, Placement, use_notifications,
    },
    theme::AutoClose,
};

// Once, near the root. It serves every handle.
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
                    placement: Some(Placement::TopCenter),
                    auto_close: Some(AutoClose::After(2000)),
                    ..Default::default()
                });
            },
            "Save"
        }
    }
}
#
# #[derive(Clone, PartialEq, Routable)]
# enum Route {
#     #[route("/0")]
#     Home {},
# }
# #[component] fn Home() -> Element { rsx! {} }
```

Render the host once. A second one would draw every notification twice.
Without a host, `show` queues notifications that nobody draws.

A contained host serves the handles created below it:

```rust,ignore
Notifications { contained: true, placement: "top-end",
    SaveButton {} // its `use_notifications()` shows them in this box
}
```

Your own template draws the content. The host keeps the list item, the live
region, the timers and the hover pause. The template may call hooks, as long as
it calls the same ones every time.

```rust,ignore
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

## Snackbar

`snackbar` is a ready made template: a message and at most one action, such as
Undo. Pressing the action runs your handler, then closes the snackbar. Show it
with `AutoClose::Never`, as a keyboard reader needs `F8` to reach the action.
The handler runs in the notification's scope, not the component's, so it
writes state that outlives the component: a signal from a context provided
near the root, not a `use_signal` of the component.

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, NotificationOptions, SnackbarData, snackbar, use_notifications_with},
    theme::AutoClose,
};

#[component]
fn ArchiveButton() -> Element {
    let snacks = use_notifications_with(snackbar);
    // Provided near the root with `use_context_provider(|| Signal::new(true))`.
    let mut archived = use_context::<Signal<bool>>();

    rsx! {
        Button {
            onclick: move |_| {
                archived.set(true);
                snacks.show_with(
                    SnackbarData::new("Message archived.")
                        .action("Undo", move || archived.set(false)),
                    NotificationOptions {
                        auto_close: Some(AutoClose::Never),
                        ..Default::default()
                    },
                );
            },
            "Archive"
        }
    }
}
```

## Your own data

`use_notifications_with` takes your own data type and a template to draw it.
The template is a `fn`, not a capturing closure, so everything it draws travels
in the data. `update(id, data)` redraws one notification in place. `snackbar`
is such a template, ready made.

## Stacks and timers

Each stack shows up to `limit` at once, and the rest wait. Hovering or focusing
one pauses every timer, and each starts over when you leave. A `contained` host
draws its stacks in its own box and keeps a queue for the handles below it.

## API

### `use_notifications` / `use_notifications_with`

```rust,ignore
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
| `hide(id)` | `()` | Fades it out, then removes it. A queued one goes at once. |
| `clear()` | `()` | Removes every notification, from every template, at once. |

`Copy`.

### `NotificationScope<T>`

| Method | Returns | Description |
|---|---|---|
| `args()` | `T` | The data it was shown with, or last updated to. Needs `T: Clone`. |
| `close()` | `()` | Starts its exit. |
| `closable()` | `bool` | Whether to draw a close button (`NotificationOptions::closable`). |
| `id()` | `NotificationId` | Its id. |

`Copy`, so several handlers in one template can each hold it.

### `NotificationData`

| Field | Type | Description |
|---|---|---|
| `title` | `Option<String>` | The `Alert`'s title and accessible name. |
| `message` | `String` | The message. Empty renders no message slot. |
| `color` | `Input<ThemeAwareValue>` | The `Alert`'s color. Unset is the theme's. |
| `variant` | `Input<Variant>` | The `Alert`'s variant. Unset is the theme's. |
| `icon` | `Option<Element>` | An icon. The host draws it later, so it must not carry event handlers. |

`From<&str>` and `From<String>` fill `message`.

### `SnackbarData`

The data of the `snackbar` template.

| Field | Type | Description |
|---|---|---|
| `message` | `String` | The text. Empty draws none. |
| `action` | `Option<SnackbarAction>` | One button, such as Undo. Pressing it runs the handler, then closes the snackbar. |

`SnackbarData::new(message).action(label, handler)` builds both. `From<&str>`
and `From<String>` fill `message`. The handler is an `FnMut()`, kept in an
`Rc`, so it may outlive the component that showed the snackbar.

### `NotificationOptions`

| Field | Type | Default | Description |
|---|---|---|---|
| `placement` | `Option<Placement>` | `None` | The stack it joins. `None` is the host's. |
| `auto_close` | `Option<AutoClose>` | `None` | When it closes. `None` is the host's. |
| `closable` | `bool` | `true` | Whether the template draws a close button. Off, keep a timer or an action that closes it: Escape does not, and `F8` only focuses it. |
| `live` | `NotificationLive` | `Polite` | `Polite` or `Assertive`, how it is announced. |

## Props

### `Notifications`

The host.

| Prop | Type | Default | Description |
|---|---|---|---|
| `placement` | `Placement` | `bottom-end` | The stack a notification joins unless it names its own. |
| `limit` | `usize` | `5` | How many show at once per stack. The rest wait. |
| `auto_close` | `AutoClose` | `After(6000)` | When a notification closes, unless it says otherwise. |
| `contained` | `bool` | `false` | Draws the stacks in this host's own box and gives the handles below it their own queue. Read once, at mount. |
| `hotkey` | `Key` | `F8` | Focuses the newest notification from anywhere, pressed without Ctrl, Alt or Meta. |
| `children` | `Element` | - | Rendered inside a contained host. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `F8` | Focuses the newest notification from anywhere. The host's `hotkey` sets the key. |

### Libero handles

- Showing one takes no focus. `live` picks a polite or an assertive
  announcement.
- Without the hotkey, a close button comes after the rest of the page in `Tab`
  order.
- A focused notification never closes on its own.
- Closing the focused one moves focus to the next notification's close button
  in its stack (the previous one's if it was last; the first focusable element
  when that one has no close button), and back to where `F8` was pressed once
  the stack is empty.

### You must

- Give one with an action, such as Undo, `AutoClose::Never`: it is safer.
- In your own template, draw the close button yourself: read `s.closable()`
  and give the button an `aria_label`, as the Card option does.
- Leave `closable` on for one that never closes on its own, unless it has an
  action that closes it. Neither Escape nor `F8` closes a notification, so the
  demo hides the switch for such an alert.

### Example

A "Message archived" notification with an Undo action and `AutoClose::Never`:
it takes no focus when it shows, F8 jumps to it, and once it closes focus
returns to where F8 was pressed.

## Theme defaults

`NotificationsDefaults` on the theme, as `notifications`.

| Field | Type | Description |
|---|---|---|
| `placement` | `Placement` | `BottomEnd`. |
| `auto_close` | `AutoClose` | `After(6000)`. |
| `limit` | `usize` | `5`. |
| `width` | `&'static str` | `360px`, capped at the viewport's (or the contained host's) width minus both offsets. |
| `gap` | `Size` | `Sm`, between two notifications. |
| `offset` | `Size` | `Md`, from the viewport's (or the contained host's) edge. |
| `transition_duration` | `u32` | `200` ms, the entry and the exit. |

The default template's close button is named by the localization's
`common.close`, and the region by `notifications.region` (`{key}` is the
hotkey).

The stacks sit on `z_index.notification` (2100), above modals and the
dropdowns opened inside them.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-notification-width` | From `NotificationsDefaults::width`. |
| `--lsx-notification-gap` | From `NotificationsDefaults::gap`. |
| `--lsx-notification-offset` | From `NotificationsDefaults::offset`. |
| `--lsx-notification-transition` | From `NotificationsDefaults::transition_duration`. |
| `--lsx-z-index-notification` | From `ZIndexDefaults::notification`. |

## Data attributes

| Attribute | On |
|---|---|
| `data-state="leaving"` | A notification's `li` while it fades out. |
