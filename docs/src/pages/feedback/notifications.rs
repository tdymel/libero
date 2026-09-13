use std::{cell::Cell, rc::Rc, time::Duration};

use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, UNSET, Wrap, prop, props};
use crate::icons::DismissIcon;
use dioxus::prelude::*;
use libero::{
    components::{
        ActionIcon, Box, Button, Code, Flex, Input, Kbd, NotificationData, NotificationLive,
        NotificationOptions, NotificationScope, Notifications, Paper, ProgressBar, Text, Variant,
        use_notifications, use_notifications_with,
    },
    platform::{TimerSubscription, timer},
    sx::sx,
    theme::{AutoClose, Placement, Theme},
};

/// What the variant control starts on, and prints nothing for.
fn default_variant() -> &'static str {
    Theme::DEFAULT.alert.variant.as_str()
}

/// The preview's own template for the card option, printed as it is written
/// below.
// snippet: item #[component] fn DismissIcon() -> Element { rsx! {} }
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

/// The upload option's type and template, printed above the hook. The ticker
/// that drives it is printed by `wrap_demo`.
// snippet: item #[component] fn DismissIcon() -> Element { rsx! {} }
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
}"#;

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

/// `AutoClose` as the caller would type it.
fn auto_close_code(value: &str) -> String {
    match value {
        "never" => "AutoClose::Never".to_string(),
        ms => format!("AutoClose::After({ms})"),
    }
}

/// The whole snippet: the host (contained here, app-wide near the root), the
/// handle, and the one `show` the button makes.
fn wrap_demo(values: &DemoValues, _: &str) -> String {
    let contained = values.str("contained") == "true";
    let template = values.str("template");
    let position = values.str("placement");
    let auto_close = values.str("auto_close");
    let upload = template == "upload";

    // The host. Contained, it takes the three defaults as props; app-wide it
    // is one line near the root and the notification names its own.
    let mut host = String::new();
    if contained {
        let mut fields = vec!["contained: true".to_string()];
        if position != "bottom-end" {
            fields.push(format!("placement: {position:?}"));
        }
        let limit = values.str("limit");
        if limit != "5" {
            fields.push(format!("limit: {limit}"));
        }
        if auto_close != "4000" {
            fields.push(format!("auto_close: {}", auto_close_code(&auto_close)));
        }
        let fields: String = fields
            .iter()
            .map(|field| format!("        {field},\n"))
            .collect();
        host.push_str(&format!(
            "// A contained host draws its stacks in its own box and keeps them to\n\
             // itself: every handle made below it - in `Examples` here - feeds it.\n\
             Box {{ sx: sx().width(\"100%\"),\n    \
                 Notifications {{\n{fields}        Examples {{}}\n    }}\n\
             }}\n\n"
        ));
    } else {
        host.push_str(
            "// App-wide, `contained` is left out: one host near the root is the\n\
             // outlet for every handle below it, and its `placement`, `limit` and\n\
             // `auto_close` are the defaults for every notification.\n\
             LiberoProvider {\n    \
                 Router::<Route> {}\n    \
                 Notifications {}\n\
             }\n\n",
        );
    }

    // The handle and its data.
    let hook = match template.as_str() {
        "card" => "use_notifications_with(card_notification)",
        "upload" => "use_notifications_with(upload_notification)",
        _ => "use_notifications()",
    };
    let mut fields = Vec::new();
    if values.str("title") == "true" {
        fields.push("title: Some(\"Saved\".into())".to_string());
    }
    if template == "alert" {
        let color = values.str("color");
        if color != UNSET {
            fields.push(format!("color: {color:?}.into()"));
        }
        let variant = values.str("variant");
        if variant != default_variant() {
            fields.push(format!("variant: {variant:?}.into()"));
        }
    }
    let data = if upload {
        "Upload { file: \"archive.zip\", percent: 0.0 }".to_string()
    } else if fields.is_empty() {
        "\"Your changes are safe.\"".to_string()
    } else {
        let fields: String = fields
            .iter()
            .map(|field| format!("        {field},\n"))
            .collect();
        format!(
            "NotificationData {{\n        message: \"Your changes are safe.\".into(),\n{fields}        ..Default::default()\n    }}"
        )
    };

    // The options. A contained host answers `placement` and `auto_close`, so
    // only the app-wide case names them per notification.
    let mut options = Vec::new();
    if !contained && position != "bottom-end" {
        options.push(format!("placement: Some({position:?}.into())"));
    }
    if upload {
        options.push("// A progress notification waits for its own end.".to_string());
        options.push("auto_close: Some(AutoClose::Never)".to_string());
    } else if !contained && auto_close != "4000" {
        options.push(format!(
            "auto_close: Some({})",
            auto_close_code(&auto_close)
        ));
    }
    if values.str("closable") == "false" {
        options.push("closable: false".to_string());
    }
    if values.str("live") == "assertive" {
        options.push("// Interrupts a screen reader; polite waits its turn.".to_string());
        options.push("live: NotificationLive::Assertive".to_string());
    }

    let show = if options.is_empty() {
        format!("notify.show({data});")
    } else {
        let options: String = options
            .iter()
            .map(|option| match option.starts_with("//") {
                true => format!("        {option}\n"),
                false => format!("        {option},\n"),
            })
            .collect();
        format!(
            "notify.show_with(\n    {data},\n    NotificationOptions {{\n{options}        ..Default::default()\n    }},\n);"
        )
    };

    let mut code = host;
    if template == "card" {
        code.push_str(CARD_EXAMPLE);
        code.push_str("\n\n");
    }
    if upload {
        code.push_str(TEMPLATE_EXAMPLE);
        code.push_str("\n\n");
    }
    let binding = match upload {
        true => "let id = ",
        false => "",
    };
    code.push_str(&format!("let notify = {hook};\n{binding}{show}"));
    if upload {
        code.push_str(
            "\n\n// ...as it progresses. `update` redraws that one notification.\nnotify.update(id, Upload { file: \"archive.zip\", percent: 40.0 });",
        );
    }
    code
}

/// The trigger, below the host when there is one, so its handle feeds it. A
/// component of its own: `Demo` calls `render` in its own scope, where the
/// hooks would be invisible.
#[component]
#[allow(clippy::too_many_arguments)]
fn Examples(
    color: String,
    variant: String,
    title: bool,
    closable: bool,
    live: String,
    template: String,
    position: String,
    auto_close: String,
    contained: bool,
) -> Element {
    // One hook either way: `use_notifications()` is the default template.
    let notify = if template == "card" {
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
    let variant = Input::<Variant>::from(variant.as_str()).copied_or(Variant::Tonal);
    let live = match live.as_str() {
        "assertive" => NotificationLive::Assertive,
        _ => NotificationLive::Polite,
    };
    // A contained host answers both, so only the app-wide case names them per
    // notification - the one this site renders near the root has the defaults.
    let host_answers = contained;
    let placement = Placement::from(position.as_str());
    let auto_close = auto_close_of(&auto_close);
    let options = move |sticky: bool| NotificationOptions {
        placement: (!host_answers).then_some(placement),
        auto_close: match sticky {
            true => Some(AutoClose::Never),
            false => (!host_answers).then_some(auto_close),
        },
        closable,
        live,
    };

    let show_upload = move || {
        let id = uploads.show_with(
            Upload {
                file: "archive.zip",
                percent: 0.0,
            },
            options(true),
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

    let upload = template == "upload";
    let show = move |_| {
        if upload {
            show_upload();
            return;
        }
        notify.show_with(
            NotificationData {
                title: title.then(|| "Saved".into()),
                message: "Your changes are safe.".into(),
                color: color.clone(),
                variant: Input::Value(variant),
                ..Default::default()
            },
            options(false),
        );
    };

    rsx! {
        // At the top: the stacks default to the bottom edge, so on a phone the
        // buttons leave them room.
        Flex {
            direction: "column",
            align: "center",
            justify: "start",
            gap: "sm",
            sx: sx().height(match contained {
                true => "420px",
                false => "auto",
            }),
            Flex { direction: "row", justify: "center", gap: "sm", wrap: "wrap",
                Button { variant: "outlined", onclick: show, "Show one" }
                Button { variant: "text", onclick: move |_| notify.clear(), "Clear all" }
            }
            Text { size: "sm",
                if contained {
                    "This host is contained, so its stacks are drawn in its own box and "
                    "every handle below it feeds that one. The preview needs that; an app "
                    "does not."
                } else {
                    "No host here: contained is left out, so this went to the one "
                    "app-wide host this site renders near the root - look at the edge of "
                    "the window. That host answers placement, limit and auto_close for "
                    "every handle; this notification names its own."
                }
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
                    prop("placement", "Input<Placement>")
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
                    prop("placement", "Option<Placement>").doc("The stack it joins; `None` is the host's."),
                    prop("auto_close", "Option<AutoClose>").doc("`None` is the host's."),
                    prop("closable", "bool").default("true").doc("Whether the template draws a close control."),
                    prop("live", "NotificationLive").default("Polite").doc("Which live region announces it."),
                ]),
                props("NotificationHandle", vec![
                    prop("show(args)", "NotificationId").doc("Shows one with the host's options."),
                    prop("show_with(args, options)", "NotificationId").doc("Shows one with its own options."),
                    prop("update(id, args)", "()").doc("Redraws that one notification with new data."),
                    prop("hide(id)", "()").doc("Runs its exit, then removes it."),
                    prop("clear()", "()").doc("Removes every notification, shown and queued."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "Notifications are a hook and a host. Render "
                    Code { source: "Notifications {{}}" }
                    " once near the root, and "
                    Code { source: "use_notifications()" }
                    " hands back a handle that shows them from anywhere - a notification "
                    "outlives the component that raised it. "
                    Code { source: "use_notifications_with" }
                    " takes your own data type and a "
                    Code { source: "fn" }
                    " template instead, which is what "
                    Code { source: "update(id, data)" }
                    " redraws in place. Hovering or focusing one pauses every timer, and "
                    "each starts over from its full time when you leave."
                }
            },
            // snippet: item #[derive(Clone, PartialEq, Routable)] enum Route { #[route("/")] Home {} }
            // snippet: item #[component] fn Home() -> Element { rsx! {} }
            // snippet: item #[component] fn Examples() -> Element { rsx! {} }
            Demo {
                component: "Notifications",
                children_text: "",
                controls: vec![
                    Control::select("placement", [
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
                    // A host prop, and the preview only has a host of its own
                    // while it is contained.
                    Control::slider("limit", ["1", "2", "3", "4", "5"])
                        .default("5")
                        .hidden_when(|values| values.str("contained") != "true"),
                    Control::toggle("auto_close", ["2000", "4000", "never"])
                        .labels(["2s", "4s", "Never"])
                        .default("4000")
                        .hidden_when(|values| values.str("template") == "upload"),
                    // The default template is an `Alert`; your own draws none,
                    // so both of its controls go with it.
                    Control::toggle("template", ["alert", "card", "upload"])
                        .labels(["Alert", "Card", "Upload"])
                        .default("alert"),
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard"])
                    .default(default_variant())
                    .hidden_when(|values| values.str("template") != "alert"),
                    Control::color("color")
                        .with_unset()
                        .hidden_when(|values| values.str("template") != "alert"),
                    Control::toggle("live", ["polite", "assertive"])
                        .labels(["Polite", "Assertive"])
                        .default("polite"),
                    Control::switch("title").hidden_when(|values| values.str("template") == "upload"),
                    // The upload template draws Dismiss once done, whatever this says.
                    Control::switch("closable")
                        .default("true")
                        .hidden_when(|values| values.str("template") == "upload"),
                    Control::switch("contained"),
                ],
                render: move |values: DemoValues| {
                    let contained = values.str("contained") == "true";
                    let examples = rsx! {
                        Examples {
                            color: values.str("color"),
                            variant: values.str("variant"),
                            title: values.str("title") == "true",
                            closable: values.str("closable") == "true",
                            live: values.str("live"),
                            template: values.str("template"),
                            position: values.str("placement"),
                            auto_close: values.str("auto_close"),
                            contained,
                        }
                    };
                    rsx! {
                        Box { sx: sx().width("100%"),
                            if contained {
                                Notifications {
                                    contained: true,
                                    placement: values.str("placement"),
                                    limit: values.str("limit").parse::<usize>().ok(),
                                    auto_close: auto_close_of(&values.str("auto_close")),
                                    {examples}
                                }
                            } else {
                                {examples}
                            }
                        }
                    }
                },
                wrap: Wrap(wrap_demo),
                wide_preview: true,
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "Showing one takes no focus. Each stack holds a polite and an assertive live "
                    "region, both mounted before anything is announced into them, and "
                    Code { source: "live" }
                    " picks which one. A close button is reached with "
                    Kbd { "Tab" }
                    " in document order, after the rest of the page, and focusing one pauses "
                    "every timer. One with an action of its own, such as Undo, needs "
                    Code { source: "AutoClose::Never" }
                    ": a keyboard user would not reach it in time. Closing the "
                    "focused one moves focus to the next close button in its stack, the "
                    "previous one after the last, and back where it came from once the "
                    "stack is empty. Your own "
                    "template draws that button itself: read "
                    Code { source: "s.closable()" }
                    " and give it an "
                    Code { source: "aria_label" }
                    ", the way the Card option here does."
                }
            }
        }
    }
}
