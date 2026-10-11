//! `use_fcm` with placeholder Firebase options; `#probe` sends a token and a message
//! down the service's path (debug builds), the spans echo the handle.

use dioxus::prelude::*;
use libero::hooks::{FcmOptions, use_fcm};

use crate::Routes;

pub const ROUTES: Routes = &[("/fcm", || rsx! { FcmPage {} })];

#[component]
fn FcmPage() -> Element {
    let mut fcm = use_fcm(FcmOptions {
        application_id: "1:1:android:1".into(),
        api_key: "placeholder".into(),
        project_id: "libero-e2e".into(),
        sender_id: "1".into(),
    });
    let mut probe = use_signal(String::new);
    let message = fcm
        .last_message()
        .map(|message| {
            format!(
                "{} {}",
                message.title.unwrap_or_default(),
                message.data.get("probe").cloned().unwrap_or_default()
            )
        })
        .unwrap_or_default();
    rsx! {
        button { id: "register", onclick: move |_| fcm.register(), "Register" }
        button {
            id: "probe",
            onclick: move |_| probe.set(run_probe()),
            "Probe"
        }
        span { id: "supported", "{fcm.is_supported()}" }
        span { id: "pending", "{fcm.is_pending()}" }
        span { id: "token", "{fcm.token().unwrap_or_default()}" }
        span { id: "message", "{message}" }
        span { id: "error", "{fcm.error():?}" }
        span { id: "probed", "{probe}" }
    }
}

fn run_probe() -> String {
    #[cfg(debug_assertions)]
    libero::platform::fcm_probe("probe-token", "hello");
    match cfg!(debug_assertions) {
        true => "sent".to_string(),
        false => "no probe in a release build".to_string(),
    }
}
