// page-snippets: fcm
use crate::components::{Demo, DemoFile, DemoValues, DocPage, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Text};

mod demo;
use demo::FcmDemo;

#[component]
pub fn UseFcmPage() -> Element {
    rsx! {
        DocPage {
            title: "Firebase messaging",
            source: "libero/src/hooks/fcm.rs",
            markdown: "/md/use_fcm.md",
            accessibility: a11y()
                .handles([
                    "Nothing on screen: the hook announces nothing and draws nothing.",
                ])
                .must([
                    "Show a message the user should see with `use_system_notification`, or in a status region that is already mounted.",
                ])
                .example("A chat app registers after sign-in, posts the token to its server, and shows each `last_message()` title in a status line.")
                .limits([
                    "A real token and a received message are unverified: libero's tests run without a Firebase project, through a debug-only path that sends what the service would.",
                    "A message that arrives while no app process runs reaches `last_message()` only if the process lives on until the app opens; a notification payload is shown by Firebase itself.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_fcm(FcmOptions)" }
                    " gives an Android app its Firebase Cloud Messaging token and the messages "
                    "that arrive while it runs. The app's server sends them. "
                    Code { source: "register()" }
                    " starts Firebase with the four values of the app's "
                    Code { source: "google-services.json" }
                    " (no Gradle plugin needed), then "
                    Code { source: "token()" }
                    ", "
                    Code { source: "last_message()" }
                    ", "
                    Code { source: "is_pending()" }
                    " and "
                    Code { source: "error()" }
                    " follow. Elsewhere it is unsupported."
                }
                Text {
                    "Behind the "
                    Code { source: "fcm" }
                    " cargo feature, off by default: it adds firebase-messaging and libero's "
                    Code { source: "LiberoFcmService" }
                    " to every Android APK of the app. The demo uses placeholder options, so "
                    "Android shows Firebase's refusal and the web says "
                    Code { source: "Unsupported" }
                    "."
                }
            },

            Demo {
                component: "FcmDemo",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { FcmDemo {} },
                file: DemoFile(include_str!("use_fcm/demo.rs")),
            }
        }
    }
}
