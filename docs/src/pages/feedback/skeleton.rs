use crate::components::{Child, Control, Demo, DemoValues, DocPage, Wrap, indent, prop, props};
use dioxus::prelude::*;
use libero::components::{Box, Button, Code, Flex, Input, Skeleton, Text};
use libero::sx::sx;

/// What the wrapper covers. A button in it, so the preview shows that covered
/// content is out of the tab order, not only out of sight.
const CONTENT: &str = r#"Flex { direction: "row", gap: "sm", align: "center",
    Text { "Ada Lovelace" }
    Button { size: "xs", "Follow" }
}"#;

fn busy_region(values: &DemoValues) -> bool {
    values.str("busy_region") == "true"
}

/// The region is the caller's: it carries `aria-busy` while the skeleton
/// covers, the same rule as the `Loader` page.
fn wrap_region(values: &DemoValues, code: &str) -> String {
    match busy_region(values) {
        true => format!(
            "Box {{ \"aria-busy\": \"{}\",\n{}}}",
            values.str("visible"),
            indent(code)
        ),
        false => code.to_string(),
    }
}

fn with_content(values: &DemoValues) -> bool {
    values.str("children") == "true"
}

fn content_code(values: &DemoValues) -> String {
    match with_content(values) {
        true => CONTENT.to_string(),
        false => String::new(),
    }
}

/// The wrapper's `height`: unset at `auto`.
fn height_code(control: &Control, values: &DemoValues) -> Vec<String> {
    let value = values.str(control.name);
    match value == control.default {
        true => vec![],
        false => vec![format!("height: {value:?}")],
    }
}

/// A shape's `height` is never unset, so it always prints.
fn shape_height_code(_: &Control, values: &DemoValues) -> Vec<String> {
    vec![format!("height: {:?}", values.str("shape_height"))]
}

/// `"auto"` is the prop left unset.
fn length(value: String, unset: &str) -> Input<libero::sx::ThemeAwareValue> {
    match value == unset {
        true => Input::None,
        false => Input::from(value),
    }
}

#[component]
pub fn SkeletonPage() -> Element {
    rsx! {
        DocPage {
            title: "Skeleton",
            source: "libero/src/components/feedback/skeleton.rs",
            markdown: "/md/skeleton.md",
            properties: vec![props("Skeleton", vec![
                prop("visible", "bool")
                    .default("true")
                    .doc("Cover the children, or draw the standalone shape. `false` shows the children as they are."),
                prop("height", "ThemeAwareValue")
                    .doc("A CSS length. Unset, the height of the children."),
                prop("width", "ThemeAwareValue")
                    .default("100%")
                    .doc("A CSS length. Ignored when `circle`."),
                prop("circle", "bool")
                    .default("false")
                    .doc("Width equals `height`, corners fully round. Without `height`, as wide as the children."),
                prop("radius", "Size")
                    .default("sm")
                    .doc("Corner. Ignored when `circle`."),
                prop("animate", "bool")
                    .default("true")
                    .doc("Run the pulse. Under `prefers-reduced-motion: reduce` it stops half-way."),
                prop("children", "Element")
                    .doc("The real content, when the skeleton wraps it."),
            ])],
            lead: rsx! {
                Text {
                    "A placeholder for content that is still loading, used two ways. Without "
                    "children it is a grey shape, and a few of them stand in for a layout. "
                    "Wrapped around the real content, it covers that content while "
                    Code { source: "visible" }
                    " and steps aside when it turns "
                    Code { source: "false" }
                    ": the layout is written once, and the placeholder is exactly its size. "
                    "Switch the children on and off, then flip "
                    Code { source: "visible" }
                    ". While it covers, the content is "
                    Code { source: "aria-hidden" }
                    " and "
                    Code { source: "inert" }
                    " - not announced, and not reachable with Tab. It is hidden with "
                    Code { source: "visibility: hidden" }
                    " rather than covered, so the grey is right on any surface; a "
                    "descendant that sets "
                    Code { source: "visibility: visible" }
                    " on itself would show through."
                }
                Text {
                    "A skeleton says nothing to a screen reader, on purpose: "
                    Code { source: "aria-busy" }
                    " on hidden content would reach nobody. Mark the region you are filling "
                    Code { source: "aria-busy" }
                    " while it waits - switch on "
                    Code { source: "Busy region" }
                    ", the same rule as "
                    Code { source: "Loader" }
                    "."
                }
            },
            Demo {
                component: "Skeleton",
                children_text: "",
                code_child: Child(content_code),
                controls: vec![
                    // Not a prop: it picks between the two usages, and the
                    // code block prints the children through `code_child`.
                    Control::switch("children").default("true").code(|_, _| vec![]),
                    Control::switch("visible").default("true"),
                    Control::switch("animate").default("true"),
                    Control::switch("circle"),
                    // Not a prop: the caller's region around the skeleton.
                    Control::switch("busy_region").code(|_, _| vec![]),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm")
                        .hidden_when(|values| values.str("circle") == "true"),
                    // Two controls for one prop: a wrapper sizes itself from
                    // its content, but a shape with no content and no height
                    // is zero pixels tall.
                    Control::toggle("height", ["auto", "12px", "40px"])
                        .code(height_code)
                        .hidden_when(|values| !with_content(values)),
                    Control::toggle("shape_height", ["12px", "40px", "80px"])
                        .code(shape_height_code)
                        .hidden_when(with_content),
                    Control::toggle("width", ["100%", "180px", "60%"])
                        .hidden_when(|values| values.str("circle") == "true"),
                ],
                wrap: Wrap(wrap_region),
                render: move |values: DemoValues| {
                    let content = with_content(&values);
                    let height = match content {
                        true => length(values.str("height"), "auto"),
                        false => length(values.str("shape_height"), ""),
                    };
                    let circle = values.str("circle") == "true";
                    let visible = values.str("visible") == "true";
                    let skeleton = rsx! {
                        Skeleton {
                            visible,
                            animate: values.str("animate") == "true",
                            circle,
                            radius: values.str("radius"),
                            height,
                            width: length(values.str("width"), "100%"),
                            if content {
                                Flex { direction: "row", gap: "sm", align: "center",
                                    Text { "Ada Lovelace" }
                                    Button { size: "xs", "Follow" }
                                }
                            }
                        }
                    };
                    match busy_region(&values) {
                        true => rsx! {
                            // Full width in the preview's flex row, as it
                            // is in a block flow; a shrunk region would
                            // shrink the `100%` skeleton with it.
                            Box {
                                "aria-busy": if visible { "true" } else { "false" },
                                sx: sx().width("100%"),
                                {skeleton}
                            }
                        },
                        false => skeleton,
                    }
                },
            }
        }
    }
}
