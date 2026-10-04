# System notifications

Crate: `libero`
Import: `use libero::hooks::{NotificationAction, PermissionState, PushEndpoint, PushError, PushOptions, PushSubscription, SystemNotification, SystemNotificationError, SystemNotifier, use_push_subscription, use_system_notification};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/system_notification.rs>
Index: [index.md](index.md) lists every other page
Description: Notifications the operating system draws and a web push subscription; never prompt on mount.

`use_system_notification() -> SystemNotifier` shows notifications the
operating system draws, outside the page; the in-app toasts are
`Notifications`. `request()` asks for the permission,
`show(SystemNotification)` shows one, `close(tag)` closes it; its `actions`
are buttons whose id comes back in `on_action`. Read
`permission()`, `error()`, `is_pending()` and `is_supported()`; all are
reactive.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text},
    hooks::{
        NotificationAction, PermissionState, PushError, PushOptions, SystemNotification,
        SystemNotificationError, use_push_subscription, use_system_notification,
    },
    sx::sx,
};

#[component]
fn Notify() -> Element {
    let mut notifier = use_system_notification();
    let mut push = use_push_subscription(PushOptions {
        service_worker: "/sw.js".into(),
        vapid_public_key: "<your server's VAPID public key>".into(),
    });
    let mut last = use_signal(String::new);
    let clicked = use_callback(move |()| last.set("Notification clicked".into()));
    let acted = use_callback(move |action: String| last.set(format!("Action pressed: {action}")));
    // The page says it too: a system notification is never the only channel.
    let status = match notifier.error() {
        Some(SystemNotificationError::Denied) => "Notifications refused".to_string(),
        Some(SystemNotificationError::Unsupported) => "No system notifications here".to_string(),
        Some(SystemNotificationError::Failed) => "The notification did not show".to_string(),
        None => last(),
    };
    let permission = match notifier.permission() {
        PermissionState::Granted => "granted",
        PermissionState::Denied => "denied",
        PermissionState::Prompt => "not asked yet",
        PermissionState::Unknown => "unknown",
        PermissionState::Unsupported => "unsupported here",
    };
    let push_status = match (push.is_supported(), push.error()) {
        (false, _) | (true, Some(PushError::Unsupported)) => "unsupported here",
        (true, Some(PushError::Denied)) => "refused, notifications are not allowed",
        (true, Some(PushError::Failed)) => "the subscription failed",
        (true, None) if push.subscription().is_some() => "subscribed",
        (true, None) => "not subscribed",
    };

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Flex { gap: "sm", wrap: "wrap",
                Button { onclick: move |_| notifier.request(), "Allow notifications" }
                Button {
                    onclick: move |_| notifier.show(SystemNotification {
                        body: Some("3 tests failed in the nightly run".into()),
                        icon: Some("/icon.png".into()),
                        tag: Some("demo".into()),
                        on_click: Some(clicked),
                        actions: vec![
                            NotificationAction::new("report", "Open report"),
                            NotificationAction::new("retry", "Run again"),
                        ],
                        on_action: Some(acted),
                        ..SystemNotification::new("Build finished")
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
                Text {
                    size: "sm",
                    sx: sx().with("overflow-wrap", "anywhere"),
                    "Push endpoint: {subscription.endpoint}"
                }
            }
            Text { size: "sm", "Permission: {permission}. Push: {push_status}." }
        }
    }
}
```

## Push

`use_push_subscription(PushOptions) -> PushSubscription` registers your
service worker and subscribes with your server's VAPID public key.
`subscription()` is the `PushEndpoint` your server stores and pushes to; the
worker shows what arrives. Sending, VAPID signing, FCM and APNs stay on your
server. The demo's key is a throwaway public key: no private key exists
anywhere in the repo, so nothing ever pushes to it. A sample worker is
`docs/public/sw.js`. Push on Android is Firebase Cloud Messaging in the app
itself, with no libero code: see the recipe below.

## Platforms

Web: a secure context (HTTPS or localhost). Desktop WebView on Linux: with
libero's `desktop` feature, the desktop's notification server over D-Bus, as
for Blitz; without it WebKitGTK denies every request. macOS and Windows
WebViews go through the page's `Notification`, untested. Push is web only.
Android: the system's notifications over JNI, after
`notifications = { description = ".." }` under `[permissions]` in the app's
`Dioxus.toml`; a tap reopens the app and runs `on_click`. The status bar icon
is a drawable named `libero_notification` in the app's resources, a white
shape on transparent; without it the launcher icon shows. Actions are untested
on macOS and Windows. Blitz on Linux: the
desktop's notification server over D-Bus, no permission to ask. Blitz on macOS
and Windows, and a server render: `is_supported()` stays false and calls fail
with `Unsupported`.

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
    pub actions: Vec<NotificationAction>,
    pub on_action: Option<Callback<String>>,
}

pub struct NotificationAction {
    pub id: String,
    pub title: String,
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
| `icon` | `None` | An image URL. Android shows it as the large icon. |
| `tag` | `None` | Showing another with the same tag replaces it; `close(tag)` closes it. |
| `silent` | `false` | Asks for no sound or vibration; a hint. |
| `on_click` | `None` | Runs on a click while the component is mounted, after the click focused the page's window. |
| `actions` | `[]` | Buttons, each a `NotificationAction::new(id, title)`. The web shows them only through a service worker. |
| `on_action` | `None` | Runs with the pressed action's `id`, as `on_click` runs. |

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
| Linux desktop (WebKitGTK) | With libero's `desktop` feature, the desktop's notification server, always `Granted`; a click raises the window and runs `on_click`. Without it every request is `Denied`. | `Unsupported`. |
| macOS, Windows desktop | Untested; a click runs `on_click` and, with libero's `desktop` feature, raises the window. | `Unsupported`. |
| Android (WebView) | Full, through the system's `NotificationManager`, with `notifications` under `[permissions]`; one channel named after the app. `icon` is the large icon, drawn to a PNG by the WebView; the status bar shows the `libero_notification` drawable, else the launcher icon. An action reopens the app. | `Unsupported`; FCM needs app-level Kotlin (recipe below). |
| Blitz on Linux | Full, through the desktop's notification server; always `Granted`. A click runs `on_click` without raising the window. | `Unsupported`. |
| Blitz on macOS and Windows, server render | `Unsupported`. | `Unsupported`. |

Where the page has no `Notification` constructor, or the notification has
actions, `show` goes through the page's service worker, and the click reaches
the worker, not the page. The notification's `data.libero` names it; post it
back to the open tabs and `on_click` or `on_action` runs (the sample `sw.js`
does this):

```js
self.addEventListener('notificationclick', (event) => {
  const libero = event.notification.data?.libero;
  const message = event.action
    ? { libero, event: 'action', action: event.action }
    : { libero, event: 'click' };
  event.waitUntil(self.clients.matchAll({ type: 'window' }).then((tabs) => {
    for (const tab of tabs) tab.postMessage(message);
    return tabs[0]?.focus();
  }));
});
```

Post `event: 'close'` from `notificationclose` the same way.

### Push on Android (FCM)

The Android WebView has no Web Push, and libero ships no Firebase code. The
app adds Firebase Cloud Messaging itself through `dx`'s Android knobs. A
recipe, not run in libero's CI:

1. Build once, then copy the manifest `dx` generated
   (`target/dx/<app>/debug/android/app/app/src/main/AndroidManifest.xml`) to
   `android/AndroidManifest.xml`. A custom manifest replaces the generated one
   as is, so keep its `<uses-permission>` lines, including
   `android.permission.POST_NOTIFICATIONS`, and add the service inside
   `<application>`:

   ```xml
   <service android:name="dev.dioxus.main.PushService" android:exported="false">
       <intent-filter>
           <action android:name="com.google.firebase.MESSAGING_EVENT" />
       </intent-filter>
   </service>
   ```

2. Point `Dioxus.toml` at it, at your own `MainActivity.kt`, and at the
   Firebase library:

   ```toml
   [application]
   android_manifest = "android/AndroidManifest.xml"
   android_main_activity = "android/MainActivity.kt"

   [android]
   gradle_dependencies = ["com.google.firebase:firebase-messaging:24.1.0"]
   ```

3. Write `android/MainActivity.kt`. `dx` compiles only this one Kotlin file,
   so the service lives in it too. Firebase starts from options in code, as
   the `google-services` Gradle plugin needs a classpath line `dx` does not
   write:

   ```kotlin
   package dev.dioxus.main

   import android.app.NotificationChannel
   import android.app.NotificationManager
   import android.app.PendingIntent
   import android.content.Intent
   import android.os.Build
   import android.os.Bundle
   import com.google.firebase.FirebaseApp
   import com.google.firebase.FirebaseOptions
   import com.google.firebase.messaging.FirebaseMessaging
   import com.google.firebase.messaging.FirebaseMessagingService
   import com.google.firebase.messaging.RemoteMessage

   // Your `[android] identifier`, as in the generated file.
   typealias BuildConfig = com.example.app.BuildConfig

   class MainActivity : WryActivity() {
       override fun onCreate(savedInstanceState: Bundle?) {
           super.onCreate(savedInstanceState)
           if (FirebaseApp.getApps(this).isEmpty()) {
               // From the Firebase console's project settings.
               val options = FirebaseOptions.Builder()
                   .setApplicationId("<mobilesdk_app_id>")
                   .setApiKey("<api key>")
                   .setProjectId("<project id>")
                   .setGcmSenderId("<project number>")
                   .build()
               FirebaseApp.initializeApp(this, options)
           }
           FirebaseMessaging.getInstance().token.addOnSuccessListener { token -> sendToServer(token) }
       }
   }

   class PushService : FirebaseMessagingService() {
       override fun onNewToken(token: String) = sendToServer(token)

       // Data messages always land here; notification messages only while the app is in front.
       override fun onMessageReceived(message: RemoteMessage) {
           val manager = getSystemService(NotificationManager::class.java)
           // libero's channel, so the user finds all of the app's notifications in one place.
           val builder = if (Build.VERSION.SDK_INT >= 26) {
               val label = applicationInfo.loadLabel(packageManager)
               manager.createNotificationChannel(
                   NotificationChannel("libero", label, NotificationManager.IMPORTANCE_DEFAULT))
               android.app.Notification.Builder(this, "libero")
           } else {
               @Suppress("DEPRECATION") android.app.Notification.Builder(this)
           }
           val open = PendingIntent.getActivity(this, 0,
               Intent(this, MainActivity::class.java), PendingIntent.FLAG_IMMUTABLE)
           val notification = builder
               .setSmallIcon(applicationInfo.icon)
               .setContentTitle(message.notification?.title ?: message.data["title"])
               .setContentText(message.notification?.body ?: message.data["body"])
               .setContentIntent(open)
               .setAutoCancel(true)
               .build()
           manager.notify(message.messageId, 0, notification)
       }
   }

   // POST the token to your server, which sends through the FCM HTTP v1 API.
   fun sendToServer(token: String) { /* ... */ }
   ```

`use_system_notification().request()` still asks for the notification
permission, from a control in the page. A tap on a pushed notification opens
the app; `on_click` runs only for notifications the page showed itself. The
server, its service account and the sending stay outside libero.

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
- Android reads `Prompt` until the first request and `Denied` after a refusal,
  also after a restart; it cannot tell a dismissed dialog from a refusal.
- Android shows `icon` as the large icon, the status bar the app's
  `libero_notification` drawable, else the launcher icon. A tap on a
  notification from before a restart only opens the app.
- An action on Android reopens the app, as a tap does. The web shows actions
  only through a service worker, which must post the press back; without one
  the notification shows without them.
- Blitz cannot raise the window on a click: `on_click` runs, the window stays
  where it is.
- Where only a service worker may show notifications (Chrome on Android),
  `on_click` runs only if the app's worker posts the click back.
- A click after the page closed runs nothing in the page: only a worker can
  open a tab then.
- A desktop WebView raises its window on a click only with libero's `desktop`
  feature; without it, `window.focus()` may not.
