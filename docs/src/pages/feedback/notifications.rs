use libero::components::Pictogram;
use pictogram_icons_lucide as lucide;
use std::{cell::Cell, rc::Rc, time::Duration};

use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, prop, props};

use dioxus::prelude::*;
use libero::{
    components::{
        ActionIcon, Avatar, Box, Button, Code, Flex, Input, NotificationData, NotificationLive,
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
const CARD_EXAMPLE: &str = r#"#[derive(Clone, PartialEq)]
struct Message {
    sender: &'static str,
    initials: &'static str,
    title: Option<String>,
    text: &'static str,
}

// A `fn`, not a capturing closure. Everything it draws travels in `Message`.
fn card_notification(s: NotificationScope<Message>) -> Element {
    let data = s.args();

    rsx! {
        Paper { shadow: "md", sx: sx().padding("md"),
            Flex { direction: "row", align: "start", gap: "md",
                Avatar { name: data.sender, initials: data.initials, color: "primary" }
                Flex { direction: "column", gap: "sm", sx: sx().flex("1").min_width("0"),
                    Flex { direction: "row", align: "start", justify: "space-between", gap: "sm",
                        Flex { direction: "column", gap: "xs",
                            if let Some(title) = data.title {
                                Text { sx: sx().font_weight("600"), "{title}" }
                            }
                            Text { "{data.text}" }
                        }
                        if s.closable() {
                            ActionIcon { variant: "standard", size: "sm", aria_label: "Dismiss",
                                onclick: move |_| s.close(), Pictogram { icon: lucide::x::outlined } }
                        }
                    }
                    Flex { direction: "row", gap: "sm",
                        Button { variant: "outlined", size: "xs", onclick: move |_| s.close(), "Reply" }
                        Button { variant: "text", size: "xs", onclick: move |_| s.close(), "Mute" }
                    }
                }
            }
        }
    }
}"#;

#[derive(Clone, PartialEq)]
struct Message {
    sender: &'static str,
    initials: &'static str,
    title: Option<String>,
    text: &'static str,
}

/// The senders the demo button cycles through.
const SENDERS: [(&str, &str, &str); 3] = [
    ("Ada Lovelace", "AL", "The engine is ready for review."),
    ("Grace Hopper", "GH", "I found the bug in the compiler."),
    ("Alan Turing", "AT", "Lunch at noon?"),
];

/// A notification drawn by your own template instead of an `Alert`: an avatar beside the
/// text, with an action row below.
fn card_notification(s: NotificationScope<Message>) -> Element {
    let data = s.args();

    rsx! {
        Paper { shadow: "md", sx: sx().padding("md"),
            Flex { direction: "row", align: "start", gap: "md",
                Avatar {
                    name: data.sender,
                    initials: data.initials,
                    color: "primary",
                }
                Flex {
                    direction: "column",
                    gap: "sm",
                    sx: sx().flex("1").min_width("0"),
                    Flex {
                        direction: "row",
                        align: "start",
                        justify: "space-between",
                        gap: "sm",
                        Flex { direction: "column", gap: "xs",
                            if let Some(title) = data.title {
                                Text { sx: sx().font_weight("600"), "{title}" }
                            }
                            Text { "{data.text}" }
                        }
                        if s.closable() {
                            ActionIcon {
                                variant: "standard",
                                size: "sm",
                                aria_label: "Dismiss",
                                onclick: move |_| s.close(),
                                Pictogram { icon: lucide::x::outlined }
                            }
                        }
                    }
                    Flex { direction: "row", gap: "sm",
                        Button {
                            variant: "outlined",
                            size: "xs",
                            onclick: move |_| s.close(),
                            "Reply"
                        }
                        Button {
                            variant: "text",
                            size: "xs",
                            onclick: move |_| s.close(),
                            "Mute"
                        }
                    }
                }
            }
        }
    }
}

/// The upload option's type and template, printed above the hook. The ticker
/// that drives it is printed by `wrap_demo`.
const TEMPLATE_EXAMPLE: &str = r#"#[derive(Clone, PartialEq)]
struct Upload {
    file: &'static str,
    percent: f64,
}

// A `fn`, not a capturing closure. Everything it draws travels in `Upload`.
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
                            onclick: move |_| s.close(), Pictogram { icon: lucide::x::outlined } }
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
                            Pictogram { icon: lucide::x::outlined }
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
        if auto_close != "6000" {
            fields.push(format!("auto_close: {}", auto_close_code(&auto_close)));
        }
        let fields: String = fields
            .iter()
            .map(|field| format!("        {field},\n"))
            .collect();
        host.push_str(&format!(
            "// A contained host draws its stacks in its own box. Every handle made\n\
             // below it, in `Examples` here, feeds it.\n\
             Box {{ sx: sx().width(\"100%\"),\n    \
                 Notifications {{\n{fields}        Examples {{}}\n    }}\n\
             }}\n\n"
        ));
    } else {
        host.push_str(
            "// App-wide, one host near the root serves every handle below it. Its\n\
             // `placement`, `limit` and `auto_close` are every notification's defaults.\n\
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
    if values.str("title") == "true" && template != "card" {
        fields.push("title: Some(\"Saved\".into())".to_string());
    }
    if template == "alert" {
        let color = values.str("color");
        if color != Theme::DEFAULT.alert.color {
            fields.push(format!("color: {color:?}.into()"));
        }
        let variant = values.str("variant");
        if variant != default_variant() {
            fields.push(format!("variant: {variant:?}.into()"));
        }
    }
    let data = if upload {
        "Upload { file: \"archive.zip\", percent: 0.0 }".to_string()
    } else if template == "card" {
        let title = match values.str("title") == "true" {
            true => "Some(\"Saved\".into())",
            false => "None",
        };
        format!(
            "// Each message carries its own sender.\nMessage {{\n        sender: \"Ada Lovelace\",\n        initials: \"AL\",\n        title: {title},\n        text: \"The engine is ready for review.\",\n    }}"
        )
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
    } else if !contained && auto_close != "6000" {
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

/// The trigger, below the host so its handle feeds it. Its own component: `Demo` calls
/// `render` in its own scope, where the hooks would be invisible.
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
    let notify = use_notifications();
    let cards = use_notifications_with(card_notification);
    let next_sender = use_hook(|| Rc::new(Cell::new(0)));
    let uploads = use_notifications_with(upload_notification);
    // Each upload's ticker. Dropped with the preview, which is what stops them.
    let tickers = use_hook(|| {
        Rc::new(std::cell::RefCell::new(Vec::<
            std::boxed::Box<dyn TimerSubscription>,
        >::new()))
    });

    let color: Input<_> = Input::from(color.as_str());
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
        if template == "card" {
            let (sender, initials, text) = SENDERS[next_sender.get() % SENDERS.len()];
            next_sender.set(next_sender.get() + 1);
            cards.show_with(
                Message {
                    sender,
                    initials,
                    title: title.then(|| "Saved".into()),
                    text,
                },
                options(false),
            );
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
                Button { variant: "text", onclick: move |_| {
                        notify.clear();
                        cards.clear();
                    },
                    "Clear all" }
            }
            Text { size: "sm",
                if contained {
                    "This host is contained, so it draws its stacks in its own box. The "
                    "preview needs that, an app does not."
                } else {
                    "Without contained, this goes to the one app-wide host near the root. "
                    "Look at the edge of the window."
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
            source: "libero/src/components/feedback/notifications/notifications.rs",
            markdown: "/md/notifications.md",
            properties: vec![
                props("Notifications", vec![
                    prop("placement", "Placement")
                        .default("bottom-end")
                        .doc("The stack a notification joins unless it names its own."),
                    prop("limit", "usize")
                        .default("5")
                        .doc("How many show at once per stack. The rest wait."),
                    prop("auto_close", "AutoClose")
                        .default("After(6000)")
                        .doc("When a notification closes, unless it says otherwise."),
                    prop("contained", "bool")
                        .default("false")
                        .doc("Draws the stacks in this host's own box and gives the handles below it their own queue. Read once, at mount."),
                    prop("hotkey", "Key")
                        .default("F8")
                        .doc("Focuses the newest notification from anywhere, pressed without Ctrl, Alt or Meta."),
                    prop("children", "Element")
                        .doc("Rendered inside a contained host."),
                ]),
                props("NotificationOptions", vec![
                    prop("placement", "Option<Placement>").doc("The stack it joins. `None` is the host's."),
                    prop("auto_close", "Option<AutoClose>").doc("When it closes. `None` is the host's."),
                    prop("closable", "bool").default("true").doc("Whether the template draws a close button."),
                    prop("live", "NotificationLive").default("Polite").doc("`Polite` or `Assertive`, how it is announced."),
                ]),
                props("NotificationHandle", vec![
                    prop("show(args)", "NotificationId").doc("Shows one with the host's options."),
                    prop("show_with(args, options)", "NotificationId").doc("Shows one with its own options."),
                    prop("update(id, args)", "()").doc("Redraws that one notification with new data."),
                    prop("hide(id)", "()").doc("Fades it out, then removes it."),
                    prop("clear()", "()").doc("Removes every notification, shown and queued."),
                ]),
            ],
            accessibility: a11y()
                .key(["F8"], "Focuses the newest notification from anywhere. The host's `hotkey` sets the key.")
                .handles([
                    "Showing one takes no focus. `live` picks a polite or an assertive announcement.",
                    "Without the hotkey, a close button comes after the rest of the page in `Tab` order.",
                    "A focused notification never closes on its own.",
                    "Closing the focused one moves focus to the next close button in its stack, and back to where `F8` was pressed once the stack is empty.",
                ])
                .must([
                    "Give one with an action, such as Undo, `AutoClose::Never`: it is safer.",
                    "In your own template, draw the close button yourself: read `s.closable()` and give the button an `aria_label`, as the Card option does.",
                ]),
            lead: rsx! {
                Text {
                    "A hook and a host. Render "
                    Code { source: "Notifications {{}}" }
                    " once near the root. "
                    Code { source: "use_notifications()" }
                    " then returns a handle that shows notifications from anywhere, and each "
                    "one outlives the component that raised it."
                }
                Text {
                    Code { source: "use_notifications_with" }
                    " takes your own data type and a template to draw it. The template is a "
                    Code { source: "fn" }
                    ", not a capturing closure, so everything it draws travels in the data. "
                    Code { source: "update(id, data)" }
                    " redraws one notification in place."
                }
                Text {
                    "Each stack shows up to "
                    Code { source: "limit" }
                    " at once, and the rest wait. Hovering or focusing one pauses every "
                    "timer, and each starts over when you leave. A "
                    Code { source: "contained" }
                    " host draws its stacks in its own box and keeps a queue for the handles "
                    "below it."
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
                    .labels([
                        "Top start",
                        "Top center",
                        "Top end",
                        "Center start",
                        "Center",
                        "Center end",
                        "Bottom start",
                        "Bottom center",
                        "Bottom end",
                    ])
                    .default("bottom-end"),
                    // A host prop, and the preview only has a host of its own
                    // while it is contained.
                    Control::slider("limit", ["1", "2", "3", "4", "5"])
                        .default("5")
                        .hidden_when(|values| values.str("contained") != "true"),
                    Control::toggle("auto_close", ["2000", "6000", "never"])
                        .labels(["2s", "6s", "Never"])
                        .default("6000")
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
                        .default(Theme::DEFAULT.alert.color)
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
        }
    }
}
