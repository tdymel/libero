use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, Flex, Sidebar, SidebarPart, Text},
    sx::sx,
};

const CONTENT: &str = r#"Text { "Navigation" }"#;
const REST: &str = r#"Flex {
    direction: "column",
    sx: sx().flex("1").padding("12px"),
    Text { "Rest of the layout" }
}"#;

/// `side` picks the border and size axis, not the position: the printed wrapper puts the
/// panel at the matching end of the DOM.
fn wrap_layout(values: &DemoValues, code: &str) -> String {
    let panel = indent(code);
    let rest = indent(REST);
    let body = match values.str("side").as_str() {
        "end" => format!("{rest}{panel}"),
        _ => format!("{panel}{rest}"),
    };
    format!(
        "Flex {{\n    direction: \"row\",\n    align: \"stretch\",\n    wrap: false,\n    sx: sx().height(\"120px\").width(\"100%\").border(\"1px solid\").border_color(\"muted.3\"),\n{body}}}"
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
                    .default("start")
                    .doc("The edge that gets the border, and whether `size` is a width or a height. `start` is the right edge under `dir=\"rtl\"`. It does not move the panel, so put it at the matching end of the DOM."),
                prop("size", "Size").default("md").doc("The panel's width, or its height on a `top` or `bottom` side."),
                prop("component", "HtmlTag").default("aside").doc("The element to render, such as `nav` for a navigation panel."),
                prop("children", "Element").doc("The panel's content, scrolled by an inner `ScrollArea`."),
                prop("parts", "Parts<SidebarPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`."),
            ])
            .parts("SidebarPart", vec![
                (SidebarPart::Scroll, "The `ScrollArea` holding the content. It carries the panel's padding."),
            ])],
            accessibility: a11y()
                .handles(["The root is an `aside`, the `complementary` landmark."])
                .must([
                    "Pass `component: \"nav\"` for the site navigation.",
                    "Give it an `aria_label` when the page has more than one landmark of that kind.",
                ]),
            lead: rsx! {
                Text {
                    "An in-flow panel on one edge of its parent that scrolls its own "
                    "content. "
                    Code { source: "side" }
                    " picks the border, not the position, so an end sidebar comes after "
                    "its sibling in the DOM. For a panel that slides in over the page, see "
                    Code { source: "use_drawer" }
                    "."
                }
            },
            Demo {
                component: "Sidebar",
                children_text: "",
                children_code: CONTENT.to_string(),
                // A `md` panel is 240px, wider than the preview beside the controls.
                wide_preview: true,
                controls: vec![
                    Control::toggle("side", ["start", "end"]).labels(["Start", "End"]),
                    Control::sizes("size")
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
                        "end" => (rest, panel),
                        _ => (panel, rest),
                    };
                    rsx! {
                        Flex {
                            direction: "row",
                            // A row centres its items; the panel spans the row's height.
                            align: "stretch",
                            wrap: false,
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
