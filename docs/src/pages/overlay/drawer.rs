use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Text},
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

/// A drawer is a hook, so the options and the trigger belong in the example.
/// The generated props are rebuilt here as the struct literal they are.
fn wrap_hook(values: &DemoValues, _: &str) -> String {
    let handle = match values.str("handle").as_str() {
        "true" => "\n        handle: true,",
        _ => "",
    };
    format!(
        "let nav = use_drawer(\n    \
             DrawerOptions {{\n        \
                 anchor: {:?}.into(),\n        \
                 size: {:?}.into(),\n        \
                 title: Some(\"Temporary drawer\".into()),{}\n        \
                 ..Default::default()\n    \
             }},\n    \
             |s: ModalScope<()>| rsx! {{\n{}    }},\n);\n\n\
         rsx! {{\n    \
             Button {{ variant: \"outlined\", onclick: move |_| {{ nav.open(); }}, \"Open drawer\" }}\n\
         }}",
        values.str("anchor"),
        values.str("size"),
        handle,
        indent(&indent(CONTENT)),
    )
}

/// The hook needs a scope of its own: `Demo` calls its `render` closure from
/// its own, where a hook would be invisible to the next reader.
#[component]
fn DrawerDemo(anchor: String, size: String, handle: bool) -> Element {
    let nav = use_drawer(
        DrawerOptions {
            anchor: anchor.into(),
            size: size.into(),
            title: Some("Temporary drawer".into()),
            handle,
            ..Default::default()
        },
        |s: ModalScope<()>| {
            rsx! {
                Flex {
                    direction: "column",
                    gap: "md",
                    sx: sx().padding("16px"),
                    Text { "Closes on Escape or a backdrop click." }
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
                    prop("anchor", "Input<DrawerAnchor>").default("start").doc("The edge the panel docks to. `start` is the right edge under `dir=\"rtl\"`."),
                    prop("size", "Input<ThemeAwareValue>").default(theme.drawer.size.as_str()).doc("Width when docked start or end, height when docked top or bottom: a size word or any CSS, e.g. `size: \"24rem\".into()`."),
                    prop("z_index", "Input<ThemeAwareValue>").doc("Stacking order of the panel."),
                    prop("aria_label", "Option<String>").doc("Names the panel, which is a dialog. With neither this nor `title`, it warns in a debug build."),
                    prop("title", "Option<String>").doc("A visible heading at the top of the panel that names it."),
                    prop("sx", "Input<Sx>").doc("Styles the panel. The content is yours to style; a bottom drawer's swipe strip is `& > [data-slot='handle']`."),
                    prop("ondismiss", "Option<Callback<Dismiss, bool>>").doc("Whether Escape, the backdrop, Back or a swipe may close it; `false` keeps it open, e.g. over an unsaved draft. Unset, all close."),
                    prop("handle", "bool").default("false").doc("Docked bottom, a touch swipe down from the panel's top padding closes it; `true` draws a grab bar there. Other anchors ignore it."),
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
                    "Give its content a way to close it: it has no header close button.",
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
                    // Drawn only docked bottom.
                    Control::switch("handle"),
                ],
                render: move |values: DemoValues| rsx! {
                    DrawerDemo {
                        anchor: values.str("anchor"),
                        size: values.str("size"),
                        handle: values.str("handle") == "true",
                    }
                },
                wrap: Wrap(wrap_hook),
            }
        }
    }
}
