use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, CodeBlock, Flex, Text, Title},
    hooks::{DrawerOptions, ModalScope, use_drawer},
    sx::sx,
};

/// The panel's own content - a subtree, so it is printed verbatim rather than
/// as `children_text`.
const CONTENT: &str = r#"Flex {
    direction: "column",
    gap: "md",
    sx: sx().padding("16px"),
    Title { size: "lg", component: "h2", "Temporary drawer" }
    Text { "Closes on Escape or a backdrop click." }
    Button { variant: "outlined", onclick: move |_| s.close(), "Close" }
}"#;

/// A drawer is a hook, so the options and the trigger are the example as much
/// as the panel is - the generated props are rebuilt here as the struct
/// literal they actually are.
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

const ARGS_EXAMPLE: &str = r#"// The argument type is the modal's, so a drawer takes per-opening data and
// answers its caller exactly like any other dialog.
let details = use_drawer(
    DrawerOptions {
        anchor: "right".into(),
        aria_label: Some("Order details".into()),
        ..Default::default()
    },
    |s: ModalScope<Order, bool>| {
        let order = s.args();

        rsx! {
            Title { size: "lg", component: "h2", "Order {order.id}" }
            Button { onclick: move |_| s.resolve(true), "Mark shipped" }
        }
    },
);

if details.open_with(order).await == Some(true) {
    refresh().await;
}"#;

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
            source: "libero/src/hooks/drawer.rs",
            markdown: "/md/drawer.md",
            properties: vec![
                props("DrawerOptions", vec![
                    prop("anchor", "Input<DrawerAnchor>").default("left").doc("The edge the panel docks to."),
                    prop("size", "Input<Size>").default("md").doc("Width along the docked edge, height for top/bottom."),
                    prop("z_index", "Input<ThemeAwareValue>").doc("Stacking order for the docked panel."),
                    prop("aria_label", "Option<String>").doc("Names the panel, which is a dialog. Unset warns in a debug build."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A dimmed, focus-trapped panel docked to one edge. It is "
                    Code { source: "use_modal" }
                    " with the docking around it, so it has the same handle, the same "
                    "per-opening arguments and the same results. For an in-flow panel, see "
                    "Sidebar."
                }
            },
            Demo {
                component: "DrawerOptions",
                children_text: "",
                children_code: CONTENT.to_string(),
                controls: vec![
                    Control::toggle("anchor", ["left", "right", "top", "bottom"]),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                ],
                render: move |values: DemoValues| rsx! {
                    DrawerDemo { anchor: values.str("anchor"), size: values.str("size") }
                },
                wrap: Wrap(wrap_hook),
            }

            DocSection {
                title: "Arguments and answers",
                Text {
                    "Everything on the Modal page applies here unchanged: "
                    Code { source: "open_with" }
                    " carries this opening's data, the returned "
                    Code { source: "Opening" }
                    " takes a handler or is awaited, and Escape or a backdrop click settles "
                    "it with "
                    Code { source: "None" }
                    "."
                }
                CodeBlock { source: ARGS_EXAMPLE, language: "rust" }
            }

            DocSection {
                title: "Accessibility",
                Text {
                    "Escape and a backdrop click close it. Set "
                    Code { source: "DrawerOptions::aria_label" }
                    ": the panel is a dialog and has no name of its own. Unlike a plain "
                    Code { source: "Dialog" }
                    ", the panel renders no header close button - a drawer's content usually "
                    "owns its own dismissal."
                }
            }
        }
    }
}
