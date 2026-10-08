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
    let side = values.str("side");
    let panel = indent(code);
    let rest = indent(REST);
    let body = match side.as_str() {
        "end" | "bottom" => format!("{rest}{panel}"),
        _ => format!("{panel}{rest}"),
    };
    let (direction, height) = layout_axis(&side);
    format!(
        "Flex {{\n    direction: \"{direction}\",\n    align: \"stretch\",\n    wrap: false,\n    sx: sx().height(\"{height}\").width(\"100%\").border(\"1px solid\").border_color(\"muted.3\"),\n{body}}}"
    )
}

/// A `top` / `bottom` panel's `size` is a height, so it needs a column taller than the panel.
fn layout_axis(side: &str) -> (&'static str, &'static str) {
    match side {
        "top" | "bottom" => ("column", "400px"),
        _ => ("row", "120px"),
    }
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
                prop("size", "Size").default("md").doc("The panel's width, or its height on a `top` or `bottom` side. A start or end panel takes half its parent's width at most."),
                prop("component", "HtmlTag").default("aside").doc("The element to render, such as `nav` for a navigation panel."),
                prop("children", "Element").doc("The panel's content, scrolled by an inner `ScrollArea`."),
                prop("parts", "Parts<SidebarPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`."),
            ])
            .parts("SidebarPart", vec![
                (SidebarPart::Scroll, "The `ScrollArea` holding the content. It carries the panel's padding."),
            ])],
            accessibility: a11y()
                .handles([
                    "The root is an `aside`, the `complementary` landmark.",
                    "Content that overflows with nothing focusable in it makes the inner scroll area a tab stop, a `region` that takes the panel's `aria_label` or `aria_labelledby`.",
                    "Content wider than the panel scrolls sideways, and a start or end panel takes half its parent's width at most, so the content keeps room at 320 px.",
                ])
                .must([
                    "Pass `component: \"nav\"` for the site navigation.",
                    "Give it an `aria_label` when the page has more than one landmark of that kind.",
                    "Give it an `aria_label` or `aria_labelledby` when its content can overflow with nothing focusable in it, or the tab stop is an unnamed region.",
                ])
                .example("The site navigation, `Sidebar { component: \"nav\", aria_label: \"Main\" }`: a screen reader lists it as the \"Main\" navigation landmark."),
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
                    Control::toggle("side", ["start", "end", "top", "bottom"])
                        .labels(["Start", "End", "Top", "Bottom"]),
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
                    let side = values.str("side");
                    let (first, second) = match side.as_str() {
                        "end" | "bottom" => (rest, panel),
                        _ => (panel, rest),
                    };
                    let (direction, height) = layout_axis(&side);
                    rsx! {
                        Flex {
                            direction,
                            // A row centres its items; the panel spans the row's height.
                            align: "stretch",
                            wrap: false,
                            sx: sx().height(height).width("100%").border("1px solid").border_color("muted.3"),
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
