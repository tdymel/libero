use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, or_unset};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Drawer, Text, Title},
    sx::sx,
};

/// The panel's own content - a subtree, not one string, so it is printed
/// verbatim rather than as `children_text`.
const CONTENT: &str = r#"Title { size: "lg", "Temporary drawer" }
Text { "Closes on Escape or backdrop click." }
Button { variant: "outlined", onclick: move |_| open.set(false), "Close" }"#;

/// A drawer only exists while it is open, so the trigger and the page state
/// that opens it are the example as much as the component is.
fn wrap_trigger(_: &DemoValues, code: &str) -> String {
    format!(
        "Button {{ variant: \"outlined\", onclick: move |_| open.set(true), \"Open drawer\" }}\nif open() {{\n{}}}",
        indent(code)
    )
}

#[component]
pub fn DrawerPage() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        DocPage {
            title: "Drawer",
            lead: rsx! {
                Text {
                    "A portaled, dimmed, focus-trapped panel docked to one edge, closing on "
                    "Escape or a backdrop click. It is mounted only while open - there is no "
                    Code { source: "opened" }
                    " prop, the caller's own state is the switch. For an in-flow panel, see "
                    "Sidebar."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Drawer",
                    children_text: "",
                    children_code: CONTENT.to_string(),
                    // The close wiring and the panel's padding are the demo's
                    // fixture, not props a control varies.
                    fixed: vec![
                        "onclose: move |_| open.set(false)".to_string(),
                        r#"sx: sx().padding("16px")"#.to_string(),
                    ],
                    controls: vec![
                        Control::toggle("anchor", ["left", "right", "top", "bottom"]),
                        Control::slider("size", ["auto", "xs", "sm", "md", "lg", "xl", "xxl"]),
                    ],
                    render: move |values: DemoValues| rsx! {
                        Button {
                            variant: "outlined",
                            onclick: move |_| open.set(true),
                            "Open drawer"
                        }
                        if open() {
                            Drawer {
                                anchor: values.str("anchor"),
                                size: or_unset(values.str("size")),
                                onclose: move |_| open.set(false),
                                sx: sx().padding("16px"),
                                Title { size: "lg", "Temporary drawer" }
                                Text { "Closes on Escape or backdrop click." }
                                Button {
                                    variant: "outlined",
                                    onclick: move |_| open.set(false),
                                    "Close"
                                }
                            }
                        }
                    },
                    wrap: Wrap(wrap_trigger),
                }
            }
        }
    }
}
