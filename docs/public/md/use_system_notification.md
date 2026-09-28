# System notifications

Crate: `libero`
Import: `use libero::hooks::{PermissionState, PushEndpoint, PushError, PushOptions, PushSubscription, SystemNotification, SystemNotificationError, SystemNotifier, use_push_subscription, use_system_notification};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/system_notification.rs>
Index: [index.md](index.md) lists every other page
Description: Notifications the operating system draws and a web push subscription; never prompt on mount.

`use_system_notification() -> SystemNotifier` shows notifications the
operating system draws, outside the page; the in-app toasts are
`Notifications`. `request()` asks for the permission,
`show(SystemNotification)` shows one, `close(tag)` closes it. Read
`permission()`, `error()`, `is_pending()` and `is_supported()`; all are
reactive.

`use_push_subscription(PushOptions) -> PushSubscription` registers your
service worker and subscribes with your server's VAPID public key.
`subscription()` is the `PushEndpoint` your server stores and pushes to; the
worker shows what arrives. Sending, VAPID signing, FCM and APNs stay on your
server. The demo's key is a throwaway public key: no private key exists
anywhere in the repo, so nothing ever pushes to it. A sample worker is
`docs/public/sw.js`.

Web: a secure context (HTTPS or localhost). Desktop WebViews: system
notifications go through the page's `Notification` where the WebView has one
(Linux WebKitGTK denies, macOS and Windows are untested); push is web only.
Android, Blitz and a server render: `is_supported()` stays false and calls
fail with `Unsupported`.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text},
    hooks::{
        PushOptions, SystemNotification, SystemNotificationError, use_push_subscription,
        use_system_notification,
    },
};

#[component]
fn Notify() -> Element {
    let mut notifier = use_system_notification();
    let mut push = use_push_subscription(PushOptions {
        service_worker: "/sw.js".into(),
        vapid_public_key: "<your server's VAPID public key>".into(),
    });
    let mut clicks = use_signal(|| 0);
    let clicked = use_callback(move |()| clicks += 1);
    // The page says it too: a system notification is never the only channel.
    let status = match notifier.error() {
        Some(SystemNotificationError::Denied) => "Notifications refused".to_string(),
        Some(SystemNotificationError::Unsupported) => "No system notifications here".to_string(),
        Some(SystemNotificationError::Failed) => "The notification did not show".to_string(),
        None if clicks() > 0 => format!("Notification clicked {} times", clicks()),
        None => String::new(),
    };

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Flex { gap: "sm", wrap: "wrap",
                Button { onclick: move |_| notifier.request(), "Allow notifications" }
                Button {
                    onclick: move |_| notifier.show(SystemNotification {
                        body: Some("Sent from the libero docs".into()),
                        tag: Some("demo".into()),
                        on_click: Some(clicked),
                        ..SystemNotification::new("Hello")
                    }),
                    "Notify"
                }
                Button { variant: "outlined", onclick: move |_| notifier.close("demo"), "Close it" }
                Button {
                    variant: "outlined",
                    onclick: move |_| if push.subscription().is_some() { push.unsubscribe() } else { push.subscribe() },
                    if push.subscription().is_some() { "Unsubscribe from push" } else { "Subscribe to push" }
                }
            }
            div { role: "status", "{status}" }
            if let Some(subscription) = push.subscription() {
                // POST it to the app's server, which pushes with its VAPID private key.
                Text { size: "sm", "Push endpoint: {subscription.endpoint}" }
            }
        }
    }
}
```

## API

```rust,ignore
pub fn use_system_notification() -> SystemNotifier
pub fn use_push_subscription(options: PushOptions) -> PushSubscription

pub struct SystemNotification {
    pub title: String,
    pub body: Option<String>,
    pub icon: Option<String>,
    pub tag: Option<String>,
    pub silent: bool,
    pub on_click: Option<Callback<()>>,
}

pub struct PushOptions {
    pub service_worker: String,
    pub vapid_public_key: String,
}

pub struct PushEndpoint {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}

pub enum SystemNotificationError { Unsupported, Denied, Failed }
pub enum PushError { Unsupported, Denied, Failed }
pub enum PermissionState { Granted, Denied, Prompt, Unknown, Unsupported }
```

| Field of `SystemNotification` | Default | Description |
|---|---|---|
| `title` | | Set by `SystemNotification::new(title)`. |
| `body` | `None` | The text under the title. |
| `icon` | `None` | An image URL. |
| `tag` | `None` | Showing another with the same tag replaces it; `close(tag)` closes it. |
| `silent` | `false` | Asks for no sound or vibration; a hint. |
| `on_click` | `None` | Runs on a click while the component is mounted, after the click focused the page's window. |

| Method of `SystemNotifier` | Description |
|---|---|
| `request()` | Asks for the permission, prompting if the user has not answered. Does nothing while one is pending. |
| `show(notification)` | Shows it; without a grant it fails with `Denied`. |
| `close(tag)` | Closes the shown notifications with this tag. |
| `permission()` | `Prompt`, `Granted` or `Denied`, following the Permissions API's changes; `Unsupported` without a Notifications API. |
| `error()` | Why the last request or show failed; the next success clears it. |
| `is_supported()`, `is_pending()` | `false` until mounted, then whether the API exists; a request awaits its answer. |

| Method of `PushSubscription` | Description |
|---|---|
| `subscribe()` | Registers `service_worker`, prompts for the notification permission if needed, subscribes. |
| `unsubscribe()` | Ends the subscription; tell your server to forget the endpoint. |
| `subscription()` | The current `PushEndpoint`, read after mount; POST it to your server. |
| `permission()`, `error()`, `is_supported()`, `is_pending()` | As on `SystemNotifier`. `Failed`: the worker did not register or the push service refused. |

Both handles are `Copy`. `PushOptions` apply to the next `subscribe`.

| Platform | System notifications | Push |
|---|---|---|
| Web | Full, in a secure context. Chrome on Android shows through the page's service worker, which must forward clicks (below). | Full, in a secure context; iOS Safari only for a Home Screen web app. |
| Linux desktop (WebKitGTK) | Every request is `Denied`. | `Unsupported`. |
| macOS, Windows desktop | Untested; a click runs `on_click` but may not raise the window. | `Unsupported`. |
| Android (WebView) | `Unsupported`: the WebView has no Notifications API. Native notifications and FCM need app-level Kotlin. | `Unsupported`. |
| Blitz, server render | `Unsupported`. | `Unsupported`. |

Where the page has no `Notification` constructor, `show` goes through the
page's service worker, and the click reaches the worker, not the page. The
notification's `data.libero` names it; post it back to the open tabs and
`on_click` runs (the sample `sw.js` does this):

```js
self.addEventListener('notificationclick', (event) => {
  const libero = event.notification.data?.libero;
  event.waitUntil(self.clients.matchAll({ type: 'window' }).then((tabs) => {
    for (const tab of tabs) tab.postMessage({ libero, event: 'click' });
    return tabs[0]?.focus();
  }));
});
```

Post `event: 'close'` from `notificationclose` the same way.

## Accessibility

### Libero handles

- Mounting never prompts: the browser asks only on request, show after a
  grant, or subscribe, which you call from a user's action.
- Unmounting stops click handling; shown notifications stay for the user to
  dismiss.
- It announces nothing: the notification is outside the page, with no live
  region.

### You must

- Never make a system notification the only channel: say the same in the
  page, where screen reader and keyboard users already are.
- Never ask for the permission on page load; ask from a visible control whose
  label says what you will notify about.
- Announce a refusal or failure once in a status region, as the demo does, and
  say how to re-enable notifications in the browser settings.
- Let the user stop push from the page as well as in the browser: unsubscribe
  and tell your server.

### Limits

- A denial is usually permanent for the site: the browser does not ask again,
  and libero cannot open its settings.
- The Android WebView has no Notifications API, and libero declares no
  POST_NOTIFICATIONS: native notifications and FCM need app-level Kotlin.
- Where only a service worker may show notifications (Chrome on Android),
  `on_click` runs only if the app's worker posts the click back.
- A click after the page closed runs nothing in the page: only a worker can
  open a tab then.
- In a desktop WebView, `window.focus()` may not raise the app's window.
