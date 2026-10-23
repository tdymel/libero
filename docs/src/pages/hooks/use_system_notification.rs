use crate::components::{Demo, DemoFile, DemoValues, DocPage, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Text};

mod demo;
use demo::Notify;

#[component]
pub fn UseSystemNotificationPage() -> Element {
    rsx! {
        DocPage {
            title: "System notifications",
            source: "libero/src/hooks/system_notification.rs",
            markdown: "/md/use_system_notification.md",
            accessibility: a11y()
                .handles([
                    "Mounting never prompts: the browser asks only on request, show after a grant, or subscribe, which you call from a user's action.",
                    "Unmounting stops click handling; shown notifications stay for the user to dismiss.",
                    "It announces nothing: the notification is outside the page, with no live region.",
                ])
                .must([
                    "Never make a system notification the only channel: say the same in the page, where screen reader and keyboard users already are.",
                    "Never ask for the permission on page load; ask from a visible control whose label says what you will notify about.",
                    "Announce a refusal or failure once in a status region, as the demo does, and say how to re-enable notifications in the browser settings.",
                    "Let the user stop push from the page as well as in the browser: unsubscribe and tell your server.",
                ])
                .example("A \"Notify me when the export is done\" switch that asks for the permission on press: when the export ends, the system notification shows and a status line in the page says the same.")
                .limits([
                    "A denial is usually permanent for the site: the browser does not ask again, and libero cannot open its settings.",
                    "Android reads Prompt until the first request and Denied after a refusal, also after a restart; it cannot tell a dismissed dialog from a refusal.",
                    "Android shows icon as the large icon, the status bar the app's libero_notification drawable, else the launcher icon. A tap on a notification from before a restart only opens the app.",
                    "An action on Android reopens the app, as a tap does. The web shows actions only through a service worker, which must post the press back; without one the notification shows without them.",
                    "Blitz cannot raise the window on a click: on_click runs, the window stays where it is.",
                    "Where only a service worker may show notifications (Chrome on Android), on_click runs only if the app's worker posts the click back, as the sample sw.js does.",
                    "A click after the page closed runs nothing in the page: only a worker can open a tab then.",
                    "A desktop WebView raises its window on a click only with libero's desktop feature; without it, window.focus() may not.",
                    "Web needs a secure context (HTTPS or localhost); WebView2 counts dioxus's page as insecure and denies it until the app marks the page secure. Android needs `notifications = { description = \"..\" }` under `[permissions]` in Dioxus.toml.",
                    "iOS's WebView, Blitz on macOS and Windows, and a server render have no notifications: `is_supported()` stays false and calls fail with `Unsupported`. Push is web only; sending, VAPID signing, FCM and APNs stay on your server.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_system_notification() -> SystemNotifier" }
                    " shows notifications the operating system draws, outside the page; the in-app toasts are "
                    Code { source: "Notifications" }
                    ". "
                    Code { source: "request()" }
                    " asks for the permission, "
                    Code { source: "show(SystemNotification)" }
                    " shows one, "
                    Code { source: "close(tag)" }
                    " closes it; its "
                    Code { source: "actions" }
                    " are buttons whose id comes back in "
                    Code { source: "on_action" }
                    ". Read "
                    Code { source: "permission()" }
                    ", "
                    Code { source: "error()" }
                    ", "
                    Code { source: "is_pending()" }
                    " and "
                    Code { source: "is_supported()" }
                    "; all are reactive. "
                    Code { source: "use_push_subscription(PushOptions)" }
                    " registers your service worker and subscribes with your server's VAPID public key; its "
                    Code { source: "endpoint()" }
                    " is what your server pushes to."
                }
            },

            // snippet: ignore - the file names the site's logo; the page compiles it as its own module
            Demo {
                component: "Notify",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Notify {} },
                file: DemoFile(include_str!("use_system_notification/demo.rs")),
            }
        }
    }
}
