# Firebase messaging

Crate: `libero`, feature `fcm`
Import: `use libero::hooks::{Fcm, FcmError, FcmMessage, FcmOptions, use_fcm};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/fcm.rs>
Index: [index.md](index.md) lists every other page
Description: Android's Firebase Cloud Messaging token and the messages received while running, behind the `fcm` feature.

`use_fcm(FcmOptions)` gives an Android app its Firebase Cloud Messaging token
and the messages that arrive while it runs. The app's server sends them.
`register()` starts Firebase with the four values of the app's
`google-services.json` (no Gradle plugin needed), then `token()`,
`last_message()`, `is_pending()` and `error()` follow. Elsewhere it is
unsupported.

Behind the `fcm` cargo feature, off by default: it adds firebase-messaging and
libero's `LiberoFcmService` to every Android APK of the app. The demo uses
placeholder options, so Android shows Firebase's refusal and the web says
`Unsupported`.

```toml
libero = { version = "0.5", features = ["fcm"] }
```

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text},
    hooks::{FcmOptions, use_fcm},
};

#[component]
fn FcmDemo() -> Element {
    let mut fcm = use_fcm(FcmOptions {
        application_id: "1:123:android:abc".into(),
        api_key: "<current_key>".into(),
        project_id: "my-app".into(),
        sender_id: "123".into(),
    });
    let token = fcm.token().unwrap_or_else(|| "none".into());

    rsx! {
        Flex { direction: "column", gap: "xs", align: "start",
            Button { onclick: move |_| fcm.register(), "Register" }
            div { role: "status",
                // POST the token to the app's server.
                Text { size: "sm", "Token: {token}" }
                if let Some(title) = fcm.last_message().and_then(|message| message.title) {
                    Text { size: "sm", "{title}" }
                }
            }
        }
    }
}
```

## API

```rust,ignore
pub fn use_fcm(options: FcmOptions) -> Fcm

impl Fcm {
    pub fn register(&mut self);
    pub fn is_supported(&self) -> bool;
    pub fn token(&self) -> Option<String>;
    pub fn last_message(&self) -> Option<FcmMessage>;
    pub fn is_pending(&self) -> bool;
    pub fn error(&self) -> Option<FcmError>;
}

pub struct FcmOptions {
    pub application_id: String, // mobilesdk_app_id
    pub api_key: String,        // current_key
    pub project_id: String,     // project_id
    pub sender_id: String,      // project_number
}

pub struct FcmMessage {
    pub data: BTreeMap<String, String>,
    pub title: Option<String>,
    pub body: Option<String>,
}

pub enum FcmError {
    Unsupported,
    Failed(String),
}
```

| Platform | Support |
|---|---|
| Android (WebView) | Firebase through libero's Gradle module; a new token from Firebase updates `token()`. |
| Web, desktop, Blitz, server | `is_supported()` stays `false`; `register()` sets `Unsupported`. |

## Accessibility

### Libero handles

- Nothing on screen: the hook announces nothing and draws nothing.

### You must

- Show a message the user should see with `use_system_notification`, or in a
  status region that is already mounted.

### Example

A chat app registers after sign-in, posts the token to its server, and shows
each `last_message()` title in a status line.

### Limits

- A real token and a received message are unverified: libero's tests run
  without a Firebase project, through a debug-only path that sends what the
  service would.
- A message that arrives while no app process runs reaches `last_message()`
  only if the process lives on until the app opens; a notification payload is
  shown by Firebase itself.
