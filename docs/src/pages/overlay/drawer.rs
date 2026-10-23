use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Options, Tabs, Text},
    hooks::{DrawerOptions, ModalScope, use_drawer},
    sx::sx,
    use_theme,
};

/// The panel's own content, a subtree, so it is printed verbatim rather than
/// as `children_text`.
// snippet: let s: ModalScope<()> = todo!();
const CONTENT: &str = r#"Flex {
    direction: "column",
    gap: "md",
    sx: sx().padding("16px"),
    Text { "Closes on Escape or a backdrop click." }
    Button { variant: "outlined", onclick: move |_| s.close(), "Close" }
}"#;

/// The enum is the tab strip, so the tabs variant has to print it. `static`: the
/// snippet scan compiles these in the Demo's output, not alone.
static SETTING_ENUM: &str = r#"#[derive(Clone, PartialEq, Options)]
enum Setting {
    General,
    Account,
    Billing,
    Privacy,
    Security,
    Alerts,
    Language,
    Advanced,
}

"#;

/// A vertical strip taller than its box, opened on its last tab so the strip
/// scrolls that tab into view.
static TABS_CONTENT: &str = r#"Flex {
    direction: "column",
    gap: "md",
    sx: sx().padding("16px"),
    Tabs {
        aria_label: "Settings",
        placement: "start",
        sx: sx().selector("& > [role=\"tablist\"]", sx().max_height("8rem")),
        value: setting(),
        onchange: move |next| setting.set(next),
        panel: |setting: Setting| rsx! { Text { "{setting.label()}" } },
    }
    Button { variant: "outlined", onclick: move |_| s.close(), "Close" }
}"#;

#[derive(Clone, PartialEq, Options)]
enum Setting {
    General,
    Account,
    Billing,
    Privacy,
    Security,
    Alerts,
    Language,
    Advanced,
}

/// A drawer is a hook, so the options and the trigger belong in the example.
/// The generated props are rebuilt here as the struct literal they are.
fn wrap_hook(values: &DemoValues, _: &str) -> String {
    let handle = match values.str("handle") == "true" && values.str("anchor") == "bottom" {
        true => "\n        handle: true,",
        false => "",
    };
    let close_button = match values.str("close_button").as_str() {
        "true" => "\n        close_button: true,",
        _ => "",
    };
    let tabs = values.str("tabs") == "true";
    let (items, state, content) = match tabs {
        true => (
            SETTING_ENUM,
            "let mut setting = use_signal(|| Setting::Advanced);\n\n",
            TABS_CONTENT,
        ),
        false => ("", "", CONTENT),
    };
    format!(
        "{items}{state}let nav = use_drawer(\n    \
             DrawerOptions {{\n        \
                 anchor: {:?}.into(),\n        \
                 size: {:?}.into(),\n        \
                 title: Some(\"Temporary drawer\".into()),{}{}\n        \
                 ..Default::default()\n    \
             }},\n    \
             move |s: ModalScope<()>| rsx! {{\n{}    }},\n);\n\n\
         rsx! {{\n    \
             Button {{ variant: \"outlined\", onclick: move |_| {{ nav.open(); }}, \"Open drawer\" }}\n\
         }}",
        values.str("anchor"),
        values.str("size"),
        close_button,
        handle,
        indent(&indent(content)),
    )
}

/// The hook needs a scope of its own: `Demo` calls its `render` closure from
/// its own, where a hook would be invisible to the next reader.
#[component]
fn DrawerDemo(
    anchor: String,
    size: String,
    handle: bool,
    close_button: bool,
    tabs: bool,
) -> Element {
    let mut setting = use_signal(|| Setting::Advanced);
    let nav = use_drawer(
        DrawerOptions {
            anchor: anchor.into(),
            size: size.into(),
            title: Some("Temporary drawer".into()),
            close_button,
            handle,
            ..Default::default()
        },
        move |s: ModalScope<()>| {
            rsx! {
                Flex {
                    direction: "column",
                    gap: "md",
                    sx: sx().padding("16px"),
                    if tabs {
                        Tabs {
                            aria_label: "Settings",
                            placement: "start",
                            sx: sx().selector("& > [role=\"tablist\"]", sx().max_height("8rem")),
                            value: setting(),
                            onchange: move |next| setting.set(next),
                            panel: |setting: Setting| rsx! { Text { "{setting.label()}" } },
                        }
                    } else {
                        Text { "Closes on Escape or a backdrop click." }
                    }
                    Button { variant: "outlined", onclick: move |_| s.close(), "Close" }
                }
            }
        },
    );

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| {
                nav.open();
            },
            "Open drawer"
        }
    }
}

#[component]
pub fn DrawerPage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "Drawer",
            source: "libero/src/components/overlay/use_drawer.rs",
            markdown: "/md/drawer.md",
            properties: vec![
                props("DrawerOptions", vec![
                    prop("anchor", "Input<DrawerAnchor>").default("start").doc("The edge the panel docks to. `start` is the right edge under `dir=\"rtl\"`. It is read on every render, so an anchor in a signal moves an open drawer."),
                    prop("size", "Input<ThemeAwareValue>").default(theme.drawer.size.as_str()).doc("Width when docked start or end, height when docked top or bottom: a size word or any CSS, e.g. `size: \"24rem\".into()`."),
                    prop("z_index", "Input<ThemeAwareValue>").doc("Stacking order of the panel."),
                    prop("aria_label", "Option<String>").doc("Names the panel, which is a dialog. With neither this nor `title`, it warns in a debug build."),
                    prop("title", "Option<String>").doc("A visible heading at the top of the panel that names it."),
                    prop("sx", "Input<Sx>").doc("Styles the panel. The content is yours to style; a bottom drawer's swipe strip is `& > [data-slot='handle']`."),
                    prop("ondismiss", "Option<Callback<Dismiss, bool>>").doc("Whether Escape, the backdrop, Back, a swipe or the header close button may close it; `false` keeps it open, e.g. over an unsaved draft. Unset, all close."),
                    prop("handle", "bool").default("false").doc("Docked bottom, a touch swipe down from the panel's top padding closes it; `true` draws a grab bar there. Other anchors ignore it. Let go far enough and, once `ondismiss` allows `Dismiss::Swipe`, it slides off and closes; a short swipe springs back. Under reduced motion it closes at once."),
                    prop("close_button", "bool").default("false").doc("Draws a close button in the header, beside `title`. As a `Dialog`'s, it asks `ondismiss` with `Dismiss::CloseButton`."),
                ]).without_base_props(),
            ],
            accessibility: a11y()
                .key(["Escape"], "Closes the drawer, as a backdrop click does, unless `ondismiss` returns `false`.")
                .key(["Tab", "Shift+Tab"], "Moves the focus within the panel. It does not leave while the drawer shows.")
                .handles([
                    "Focus moves into the panel, and back to the trigger once it closes, as with `use_modal`.",
                    "A dismissal settles the `Opening` with `None`, so a result handler runs with `None`: match `Some(answer)` to act on an answer only.",
                    "A `title` is a heading that names the panel; without it or `aria_label` the panel warns in a debug build.",
                    "Android's Back button closes the drawer, as Escape does, rather than the app.",
                    "Docked bottom, a touch swipe down from the panel's top edge closes it. It is an extra route: Escape, Back, the backdrop and the content's own close button still work.",
                ])
                .must([
                    "Set `DrawerOptions::title` or `aria_label`, since the panel is a dialog with no name of its own.",
                    "Give it a visible way to close: `close_button: true`, or a button in the content.",
                ])
                .example("A filters drawer, `DrawerOptions { aria_label: Some(\"Filters\".into()), .. }`: focus moves into the panel, Tab stays inside it, and Escape or the panel's own Done button closes it and returns focus to the trigger.")
                .limits([
                    "On Android, a drawer opened without a tap (on mount or from a timer) may let Back close the app.",
                    "In a desktop WebView or on Android, Tab and Shift+Tab move between the controls but can leave the drawer at its ends.",
                ]),
            lead: rsx! {
                Text {
                    "A dimmed, focus-trapped panel docked to one edge. "
                    Code { source: "use_drawer" }
                    " is "
                    Code { source: "use_modal" }
                    " with the docking around it, so it has the same handle, arguments and "
                    "results. For a panel in the page flow, use "
                    Code { source: "Sidebar" }
                    "."
                }
            },
            Demo {
                component: "DrawerOptions",
                children_text: "",
                children_code: CONTENT.to_string(),
                controls: vec![
                    Control::toggle("anchor", ["start", "end", "top", "bottom"])
                        .labels(["Start", "End", "Top", "Bottom"]),
                    // `24rem` is plain CSS, next to the size scale.
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl", "24rem"]).default(theme.drawer.size.as_str()),
                    // On by default, so the header button shows on the first open; printed while on.
                    Control::switch("close_button").default("true").code(|_, _| vec![]),
                    // Drawn only docked bottom, where a touch swipe down from it closes the panel.
                    Control::switch("handle").hidden_when(|values| values.str("anchor") != "bottom"),
                    // A short vertical Tabs strip that scrolls its last tab into view on open.
                    Control::switch("tabs").default("true").code(|_, _| vec![]),
                ],
                render: move |values: DemoValues| rsx! {
                    DrawerDemo {
                        anchor: values.str("anchor"),
                        size: values.str("size"),
                        handle: values.str("handle") == "true",
                        close_button: values.str("close_button") == "true",
                        tabs: values.str("tabs") == "true",
                    }
                },
                wrap: Wrap(wrap_hook),
            }
        }
    }
}
