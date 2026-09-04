use std::{cell::Cell, rc::Rc, time::Duration};

use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, UNSET, Wrap, prop, props};
use crate::icons::DismissIcon;
use dioxus::prelude::*;
use libero::{
    components::{
        ActionIcon, Box, Button, ButtonVariant, Code, CodeBlock, Flex, Input, NotificationData,
        NotificationLive, NotificationOptions, NotificationScope, Notifications, Paper,
        ProgressBar, Text, use_notifications, use_notifications_with,
    },
    platform::{TimerSubscription, timer},
    sx::sx,
    theme::{AutoClose, Theme},
};

/// What the variant control starts on, and prints nothing for.
fn default_variant() -> &'static str {
    Theme::DEFAULT.alert.variant.as_str()
}

const SETUP_EXAMPLE: &str = r#"// Once, near the root - the one outlet for every handle.
LiberoProvider {
    Router::<Route> {}
    Notifications {}
}

// Anywhere below it.
let notify = use_notifications();
notify.show("Saved.");
notify.show_with(
    NotificationData {
        title: Some("Upload failed".into()),
        message: "archive.zip is over the 10 MB limit.".into(),
        color: "error".into(),
        ..Default::default()
    },
    // Interrupts a screen reader; everything else waits its turn.
    NotificationOptions { live: NotificationLive::Assertive, ..Default::default() },
);"#;

/// The preview's own template for the custom-template switch, printed as it
/// is written below.
const CARD_EXAMPLE: &str = r#"// A `fn`, not a capturing closure: everything it draws travels in the data.
fn card_notification(s: NotificationScope<NotificationData>) -> Element {
    let data = s.args();

    rsx! {
        Paper { shadow: "md", sx: sx().padding("md"),
            Flex { direction: "row", align: "center", justify: "space-between", gap: "sm",
                Flex { direction: "column", gap: "xs",
                    if let Some(title) = data.title {
                        Text { sx: sx().font_weight("600"), "{title}" }
                    }
                    Text { "{data.message}" }
                }
                if s.closable() {
                    ActionIcon { variant: "standard", size: "sm", aria_label: "Dismiss",
                        onclick: move |_| s.close(), DismissIcon {} }
                }
            }
        }
    }
}"#;

/// A plain notification drawn by your own template instead of an `Alert`.
fn card_notification(s: NotificationScope<NotificationData>) -> Element {
    let data = s.args();

    rsx! {
        Paper { shadow: "md", sx: sx().padding("md"),
            Flex {
                direction: "row",
                align: "center",
                justify: "space-between",
                gap: "sm",
                Flex { direction: "column", gap: "xs",
                    if let Some(title) = data.title {
                        Text { sx: sx().font_weight("600"), "{title}" }
                    }
                    Text { "{data.message}" }
                }
                if s.closable() {
                    ActionIcon {
                        variant: "standard",
                        size: "sm",
                        aria_label: "Dismiss",
                        onclick: move |_| s.close(),
                        DismissIcon {}
                    }
                }
            }
        }
    }
}

const TEMPLATE_EXAMPLE: &str = r#"#[derive(Clone, PartialEq)]
struct Upload {
    file: &'static str,
    percent: f64,
}

// A `fn`, not a capturing closure: everything it draws travels in `Upload`.
fn upload_notification(s: NotificationScope<Upload>) -> Element {
    let upload = s.args();
    let done = upload.percent >= 100.0;

    rsx! {
        Paper { shadow: "md", sx: sx().padding("md"),
            Flex { direction: "column", gap: "sm",
                Flex { direction: "row", align: "center", justify: "space-between",
                    Text { if done { "Uploaded {upload.file}" } else { "Uploading {upload.file}" } }
                    if done {
                        ActionIcon { variant: "standard", size: "sm", aria_label: "Dismiss",
                            onclick: move |_| s.close(), DismissIcon {} }
                    }
                }
                ProgressBar { value: upload.percent, "aria-label": "{upload.file}" }
            }
        }
    }
}

let uploads = use_notifications_with(upload_notification);
let id = uploads.show_with(Upload { file: "archive.zip", percent: 0.0 },
    NotificationOptions { auto_close: Some(AutoClose::Never), ..Default::default() });
// ...as it progresses:
uploads.update(id, Upload { file: "archive.zip", percent: 40.0 });"#;

#[derive(Clone, PartialEq)]
struct Upload {
    file: &'static str,
    percent: f64,
}

/// The page's own template - the code beside it is what drew it.
fn upload_notification(s: NotificationScope<Upload>) -> Element {
    let upload = s.args();
    let done = upload.percent >= 100.0;

    rsx! {
        Paper { shadow: "md", sx: sx().padding("md"),
            Flex { direction: "column", gap: "sm",
                Flex {
                    direction: "row",
                    align: "center",
                    justify: "space-between",
                    Text {
                        if done {
                            "Uploaded {upload.file}"
                        } else {
                            "Uploading {upload.file}"
                        }
                    }
                    if done {
                        ActionIcon {
                            variant: "standard",
                            size: "sm",
                            aria_label: "Dismiss",
                            onclick: move |_| s.close(),
                            DismissIcon {}
                        }
                    }
                }
                ProgressBar { value: upload.percent, "aria-label": "{upload.file}" }
            }
        }
    }
}

/// The host's `auto_close` for a control value: milliseconds, or `never`.
fn auto_close_of(value: &str) -> AutoClose {
    value.parse().map_or(AutoClose::Never, AutoClose::After)
}

/// The host with the controls' props, the one `show` the plain buttons make,
/// and the template when the switch picks one. The other buttons differ from
/// that `show` only in their data and options, all printed further down.
fn wrap_demo(values: &DemoValues, _: &str) -> String {
    let mut host = vec!["contained: true".to_string()];
    let position = values.str("position");
    if position != "bottom-end" {
        host.push(format!("position: {position:?}"));
    }
    let limit = values.str("limit");
    if limit != "5" {
        host.push(format!("limit: {limit}"));
    }
    match values.str("auto_close").as_str() {
        "4000" => {}
        "never" => host.push("auto_close: AutoClose::Never".to_string()),
        ms => host.push(format!("auto_close: AutoClose::After({ms})")),
    }
    let host: String = host
        .iter()
        .map(|line| format!("        {line},\n"))
        .collect();

    let template = values.str("template") == "true";
    let hook = if template {
        "use_notifications_with(card_notification)"
    } else {
        "use_notifications()"
    };
    let mut fields = Vec::new();
    if !template {
        let color = values.str("color");
        if color != UNSET {
            fields.push(format!("color: {color:?}.into()"));
        }
        let variant = values.str("variant");
        if variant != default_variant() {
            fields.push(format!("variant: {variant:?}.into()"));
        }
    }
    let data = if fields.is_empty() {
        "\"Saved.\"".to_string()
    } else {
        let fields: String = fields
            .iter()
            .map(|field| format!("        {field},\n"))
            .collect();
        format!(
            "NotificationData {{\n        message: \"Saved.\".into(),\n{fields}        ..Default::default()\n    }}"
        )
    };
    let show = if values.str("closable") == "true" {
        format!("notify.show({data});")
    } else {
        format!(
            "notify.show_with(\n    {data},\n    NotificationOptions {{ closable: false, ..Default::default() }},\n);"
        )
    };

    let mut code = format!(
        "// The host draws its stacks in its own box, and every handle created\n\
         // below it - in `Examples` here - shows notifications there.\n\
         Box {{ sx: sx().width(\"100%\"),\n    \
             Notifications {{\n{host}        Examples {{}}\n    }}\n\
         }}\n\n\
         // In `Examples`, a 420px-tall column of the buttons:\n\
         let notify = {hook};\n\
         {show}"
    );
    if template {
        code.push_str("\n\n");
        code.push_str(CARD_EXAMPLE);
    }
    code
}

/// The buttons, below the contained host so their handles feed it. A
/// component of its own: `Demo` calls `render` in its own scope, where the
/// hooks would be invisible.
#[component]
fn Examples(color: String, variant: String, closable: bool, template: bool) -> Element {
    let notify = if template {
        use_notifications_with(card_notification)
    } else {
        use_notifications()
    };
    let uploads = use_notifications_with(upload_notification);
    // Each upload's ticker. Dropped with the preview, which is what stops them.
    let tickers = use_hook(|| {
        Rc::new(std::cell::RefCell::new(Vec::<
            std::boxed::Box<dyn TimerSubscription>,
        >::new()))
    });

    let color: Input<_> = match color.as_str() {
        UNSET => Input::None,
        color => Input::from(color),
    };
    // Every `Alert` the buttons raise takes the variant; only the plain ones
    // take the colour, since the others name their own.
    let variant = Input::<ButtonVariant>::from(variant.as_str()).copied_or(ButtonVariant::Tonal);
    let plain = move |message: &str| NotificationData {
        message: message.into(),
        color: color.clone(),
        variant: Input::Value(variant),
        ..Default::default()
    };
    let options = move |options: NotificationOptions| NotificationOptions {
        closable,
        ..options
    };

    let start_upload = move |_| {
        let id = uploads.show_with(
            Upload {
                file: "archive.zip",
                percent: 0.0,
            },
            NotificationOptions {
                auto_close: Some(AutoClose::Never),
                ..Default::default()
            },
        );
        let Some(api) = timer() else {
            return;
        };
        let percent = Cell::new(0.0);
        // The callback runs outside every scope, so it only writes - `update`
        // is a signal write and nothing else.
        let ticker = api.every(
            Duration::from_millis(300),
            std::boxed::Box::new(move || {
                if percent.get() < 100.0 {
                    percent.set(percent.get() + 10.0);
                    uploads.update(
                        id,
                        Upload {
                            file: "archive.zip",
                            percent: percent.get(),
                        },
                    );
                }
            }),
        );
        tickers.borrow_mut().push(ticker);
    };

    rsx! {
        Flex {
            direction: "column",
            align: "center",
            justify: "center",
            gap: "sm",
            sx: sx().height("420px"),
            Flex {
                direction: "row",
                justify: "center",
                gap: "sm",
                wrap: "wrap",
                Button {
                    variant: "outlined",
                    onclick: {
                        let plain = plain.clone();
                        move |_| {
                            notify.show_with(plain("Saved."), options(Default::default()));
                        }
                    },
                    "Show one"
                }
                Button {
                    variant: "outlined",
                    color: "success",
                    onclick: move |_| {
                        notify.show_with(
                            NotificationData {
                                title: Some("Published".into()),
                                message: "Your post is live.".into(),
                                color: "success".into(),
                                variant: Input::Value(variant),
                                ..Default::default()
                            },
                            options(Default::default()),
                        );
                    },
                    "Success"
                }
                Button {
                    variant: "outlined",
                    color: "error",
                    onclick: move |_| {
                        notify.show_with(
                            NotificationData {
                                title: Some("Upload failed".into()),
                                message: "archive.zip is over the 10 MB limit.".into(),
                                color: "error".into(),
                                variant: Input::Value(variant),
                                ..Default::default()
                            },
                            options(NotificationOptions {
                                live: NotificationLive::Assertive,
                                ..Default::default()
                            }),
                        );
                    },
                    "Error, announced at once"
                }
            }
            Flex {
                direction: "row",
                justify: "center",
                gap: "sm",
                wrap: "wrap",
                Button {
                    variant: "outlined",
                    onclick: {
                        let plain = plain.clone();
                        move |_| {
                            notify.show_with(
                                plain("Stays until you close it."),
                                options(NotificationOptions {
                                    auto_close: Some(AutoClose::Never),
                                    ..Default::default()
                                }),
                            );
                        }
                    },
                    "Sticky"
                }
                Button {
                    variant: "outlined",
                    onclick: move |_| {
                        notify.show_with(
                            plain("Gone in two seconds."),
                            options(NotificationOptions {
                                auto_close: Some(AutoClose::After(2000)),
                                ..Default::default()
                            }),
                        );
                    },
                    "Two seconds"
                }
                Button { variant: "outlined", onclick: start_upload, "Upload" }
                Button { variant: "text", onclick: move |_| notify.clear(), "Clear all" }
            }
        }
    }
}

#[component]
pub fn NotificationsPage() -> Element {
    rsx! {
        DocPage {
            title: "Notifications",
            source: "libero/src/components/feedback/notifications.rs",
            markdown: "/md/notifications.md",
            properties: vec![
                props("Notifications", vec![
                    prop("position", "Input<Placement>")
                        .default("theme: bottom-end")
                        .doc("The stack a notification joins unless it names its own."),
                    prop("limit", "Option<usize>")
                        .default("theme: 5")
                        .doc("Shown at once per stack; the rest wait their turn."),
                    prop("auto_close", "Option<AutoClose>")
                        .default("theme: After(4000)")
                        .doc("Unless a notification says otherwise."),
                    prop("contained", "bool")
                        .default("false")
                        .doc("Draw the stacks in this host's own box, and give the handles below it a queue of their own. Read once, at mount."),
                    prop("children", "Option<Element>")
                        .doc("Rendered inside a contained host, before its stacks."),
                ]),
                props("NotificationOptions", vec![
                    prop("position", "Option<Placement>").doc("The stack it joins; `None` is the host's."),
                    prop("auto_close", "Option<AutoClose>").doc("`None` is the host's."),
                    prop("closable", "bool").default("true").doc("Whether the template draws a close control."),
                    prop("live", "NotificationLive").default("Polite").doc("Which live region announces it."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "Notifications are a hook and a host. Render "
                    Code { source: "Notifications {{}}" }
                    " once near the root, and "
                    Code { source: "use_notifications()" }
                    " hands back a handle that shows them from anywhere. A notification outlives "
                    "the component that raised it. Hovering or focusing one pauses every timer, "
                    "and each starts over from its full time when you leave. Nothing takes focus: "
                    "a close button is reached by Tab, and each stack has a polite and an "
                    "assertive live region, both mounted before anything is announced into them."
                }
            },
            Demo {
                component: "Notifications",
                children_text: "",
                controls: vec![
                    Control::select("position", [
                        "top-start",
                        "top-center",
                        "top-end",
                        "center-start",
                        "center-center",
                        "center-end",
                        "bottom-start",
                        "bottom-center",
                        "bottom-end",
                    ])
                    .default("bottom-end"),
                    Control::slider("limit", ["1", "2", "3", "4", "5"]).default("5"),
                    Control::toggle("auto_close", ["2000", "4000", "never"])
                        .labels(["2s", "4s", "Never"])
                        .default("4000"),
                    // The `Alert`'s kind. Your own template draws no `Alert`,
                    // so both controls go with it.
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard"])
                    .default(default_variant())
                    .hidden_when(|values| values.str("template") == "true"),
                    // The plain buttons' colour; the others name their own.
                    Control::color("color")
                        .with_unset()
                        .hidden_when(|values| values.str("template") == "true"),
                    Control::switch("closable").default("true"),
                    Control::switch("template"),
                ],
                render: move |values: DemoValues| rsx! {
                    Box { sx: sx().width("100%"),
                        Notifications {
                            contained: true,
                            position: values.str("position"),
                            limit: values.str("limit").parse::<usize>().ok(),
                            auto_close: auto_close_of(&values.str("auto_close")),
                            Examples {
                                color: values.str("color"),
                                variant: values.str("variant"),
                                closable: values.str("closable") == "true",
                                template: values.str("template") == "true",
                            }
                        }
                    }
                },
                wrap: Wrap(wrap_demo),
            }
            DocSection {
                title: "In an app",
                Text {
                    "The preview's host is contained, so it keeps its notifications to itself. "
                    "An app renders one host that is not, near the root, and every handle "
                    "shows its notifications at the edge of the window."
                }
                CodeBlock { source: SETUP_EXAMPLE, language: "rust" }
            }
            DocSection {
                title: "Updating in place",
                Text {
                    Code { source: "use_notifications_with" }
                    " takes your data type and a template. The template is a "
                    Code { source: "fn" }
                    ", so a notification that outlives its caller holds nothing of the caller's. "
                    Code { source: "update(id, data)" }
                    " redraws that one notification: the Upload button above runs this."
                }
                CodeBlock { source: TEMPLATE_EXAMPLE, language: "rust" }
            }
        }
    }
}
