use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text},
    hooks::{FcmOptions, use_fcm},
};

#[component]
pub fn FcmDemo() -> Element {
    // Placeholders: Firebase refuses them on Android, and the web has no bridge.
    let mut fcm = use_fcm(FcmOptions {
        application_id: "1:123:android:abc".into(),
        api_key: "<current_key>".into(),
        project_id: "my-app".into(),
        sender_id: "123".into(),
    });
    let token = fcm.token().unwrap_or_else(|| "none".into());
    let message = fcm
        .last_message()
        .and_then(|message| message.title)
        .unwrap_or_else(|| "none".into());
    let error = fcm
        .error()
        .map(|error| format!("{error:?}"))
        .unwrap_or_else(|| "none".into());
    rsx! {
        Flex { direction: "column", gap: "xs", align: "start",
            Button { onclick: move |_| fcm.register(), "Register" }
            div { role: "status",
                Text { size: "sm", "Supported: {fcm.is_supported()}" }
                Text { size: "sm", "Token: {token}" }
                Text { size: "sm", "Last message: {message}" }
                Text { size: "sm", "Error: {error}" }
            }
        }
    }
}
