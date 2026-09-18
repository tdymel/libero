use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Text, Title},
    hooks::{DrawerOptions, ModalScope, use_drawer},
    sx::sx,
};

/// The panel's own content, a subtree, so it is printed verbatim rather than
/// as `children_text`.
// snippet: let s: ModalScope<()> = todo!();
const CONTENT: &str = r#"Flex {
    direction: "column",
    gap: "md",
    sx: sx().padding("16px"),
    Title { size: "lg", component: "h2", "Temporary drawer" }
    Text { "Closes on Escape or a backdrop click." }
    Button { variant: "outlined", onclick: move |_| s.close(), "Close" }
}"#;

/// A drawer is a hook, so the options and the trigger belong in the example.
/// The generated props are rebuilt here as the struct literal they are.
fn wrap_hook(values: &DemoValues, _: &str) -> String {
    format!(
        "let nav = use_drawer(\n    \
             DrawerOptions {{\n        \
                 anchor: {:?}.into(),\n        \
                 size: {:?}.into(),\n        \
                 aria_label: Some(\"Menu\".into()),\n        \
                 ..Default::default()\n    \
             }},\n    \
             |s: ModalScope<()>| rsx! {{\n{}    }},\n);\n\n\
         rsx! {{\n    \
             Button {{ variant: \"outlined\", onclick: move |_| {{ nav.open(); }}, \"Open drawer\" }}\n\
         }}",
        values.str("anchor"),
        values.str("size"),
        indent(&indent(CONTENT)),
    )
}

/// The hook needs a scope of its own: `Demo` calls its `render` closure from
/// its own, where a hook would be invisible to the next reader.
#[component]
fn DrawerDemo(anchor: String, size: String) -> Element {
    let nav = use_drawer(
        DrawerOptions {
            anchor: anchor.into(),
            size: size.into(),
            aria_label: Some("Menu".into()),
            ..Default::default()
        },
        |s: ModalScope<()>| {
            rsx! {
                Flex {
                    direction: "column",
                    gap: "md",
                    sx: sx().padding("16px"),
                    Title { size: "lg", component: "h2", "Temporary drawer" }
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
    rsx! {
        DocPage {
            title: "Drawer",
            source: "libero/src/components/overlay/use_drawer.rs",
            markdown: "/md/drawer.md",
            properties: vec![
                props("DrawerOptions", vec![
                    prop("anchor", "Input<DrawerAnchor>").default("start").doc("The edge the panel docks to. `start` is the right edge under `dir=\"rtl\"`."),
                    prop("size", "Input<Size>").default("md").doc("Width when docked start or end, height when docked top or bottom."),
                    prop("z_index", "Input<ThemeAwareValue>").doc("Stacking order of the panel."),
                    prop("aria_label", "Option<String>").doc("Names the panel, which is a dialog. Unset warns in a debug build."),
                ]).without_base_props(),
            ],
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
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                ],
                render: move |values: DemoValues| rsx! {
                    DrawerDemo { anchor: values.str("anchor"), size: values.str("size") }
                },
                wrap: Wrap(wrap_hook),
            }

            DocSection {
                title: "Accessibility",
                Text {
                    "Escape and a backdrop click close it. Set "
                    Code { source: "DrawerOptions::aria_label" }
                    ", since the panel is a dialog with no name of its own. It has no header "
                    "close button, so give its content a way to close it."
                }
            }
        }
    }
}
