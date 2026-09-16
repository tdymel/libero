use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Flex, Sidebar, Text},
    sx::sx,
};

const CONTENT: &str = r#"Text { "Navigation" }"#;
const REST: &str = r#"Flex {
    direction: "column",
    sx: sx().flex("1").padding("12px"),
    Text { "Rest of the layout" }
}"#;

/// `side` picks the border and the size axis, not the position - an in-flow
/// panel sits where its parent's layout puts it, so the wrapper puts it at
/// the matching end of the DOM, and the code block shows that.
fn wrap_layout(values: &DemoValues, code: &str) -> String {
    let panel = indent(code);
    let rest = indent(REST);
    let body = match values.str("side").as_str() {
        "right" => format!("{rest}{panel}"),
        _ => format!("{panel}{rest}"),
    };
    format!(
        "Flex {{\n    direction: \"row\",\n    align: \"stretch\",\n    sx: sx().height(\"120px\").width(\"100%\").border(\"1px solid\").border_color(\"muted.3\"),\n{body}}}"
    )
}

#[component]
pub fn SidebarPage() -> Element {
    rsx! {
        DocPage {
            title: "Sidebar",
            source: "libero/src/components/layout/sidebar.rs",
            markdown: "/md/sidebar.md",
            properties: vec![props("Sidebar", vec![
                prop("side", "SidebarSide")
                    .default("left")
                    .doc("Which edge this panel borders and which axis `size` applies to. It does not place the panel - an in-flow item is positioned by its parent's layout, so put it at the matching end of the DOM yourself."),
                prop("size", "Size").default("md").doc("The panel's width (or height, on a top/bottom side)."),
                prop("component", "HtmlTag").default("aside").doc("Element to render as, e.g. nav for a navigation panel."),
                prop("children", "Element").doc("The panel's content, scrolled by an inner `ScrollArea`."),
            ])],
            lead: rsx! {
                Text {
                    "An in-flow panel bordering one edge of its parent, scrolling its own "
                    "content. side picks the border and the size axis - the panel's actual "
                    "position is your layout's, so place it at the matching end of the DOM."
                }
            },
            Demo {
                component: "Sidebar",
                children_text: "",
                children_code: CONTENT.to_string(),
                controls: vec![
                    Control::toggle("side", ["left", "right"]),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                ],
                render: move |values: DemoValues| {
                    let panel = rsx! {
                        Sidebar {
                            side: values.str("side"),
                            size: values.str("size"),
                            Text { "Navigation" }
                        }
                    };
                    let rest = rsx! {
                        Flex {
                            direction: "column",
                            sx: sx().flex("1").padding("12px"),
                            Text { "Rest of the layout" }
                        }
                    };
                    let (first, second) = match values.str("side").as_str() {
                        "right" => (rest, panel),
                        _ => (panel, rest),
                    };
                    rsx! {
                        Flex {
                            direction: "row",
                            // A row centres its items; the panel spans the row's height.
                            align: "stretch",
                            sx: sx().height("120px").width("100%").border("1px solid").border_color("muted.3"),
                            {first}
                            {second}
                        }
                    }
                },
                wrap: Wrap(wrap_layout),
            }
        }
    }
}
