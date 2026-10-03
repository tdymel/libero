use crate::components::{
    Control, Demo, DemoValues, DocPage, UNSET, Wrap, a11y, indent, or_unset, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Header, Input, ScrollArea, Text},
    sx::sx,
};

/// The frame scrolls, with more content than height, so `sticky` and `static` differ.
/// A stacking context, so a native window clips the z-indexed header to it too (881).
fn wrap_frame(_: &DemoValues, code: &str) -> String {
    format!(
        "Box {{\n    sx: sx().height(\"200px\").width(\"100%\")\n        .position(\"relative\").z_index(\"0\")\n        .border(\"1px solid var(--lsx-muted-3)\"),\n    ScrollArea {{\n        aria_label: \"Header demo\",\n{}        Box {{\n            sx: sx().padding(\"md\"),\n            for i in 0..12 {{\n                Text {{ key: \"{{i}}\", \"Scroll me, line {{i}}\" }}\n            }}\n        }}\n    }}\n}}",
        indent(&indent(code))
    )
}

#[component]
pub fn HeaderPage() -> Element {
    rsx! {
        DocPage {
            title: "Header",
            source: "libero/src/components/layout/header.rs",
            markdown: "/md/header.md",
            properties: vec![props("Header", vec![
                prop("position", "HeaderPosition")
                    .default("sticky")
                    .doc("`sticky` pins to the top of the scrolling ancestor, `static` scrolls away. `fixed` pins to the viewport, so offset your content by `var(--lsx-header-height)`. In a native app `fixed` scrolls with the page for now; use `sticky`."),
                prop("publish_height", "bool")
                    .default("false")
                    .doc("Publishes the height as `--lsx-header-height` and `scroll-padding-top` on `:root`, so focus scrolls clear of a sticky or fixed banner. Set it on the page's own banner only."),
                prop("size", "ThemeAwareValue")
                    .default("md")
                    .doc("Minimum height. The header grows when its content wraps."),
                prop("color", "ThemeAwareValue")
                    .default("none, a neutral background")
                    .doc("Fills the header with shade 6 and a readable text color. With `glass`, a translucent tint of it, as on `Paper`. Under a gradient, its first stop."),
                prop("glass", "bool")
                    .default("false")
                    .doc("Frosted glass, as on `Paper`: content scrolling under the bar shows through, blurred. A `color` tints it, as on `Paper`. Opaque when the user reduces transparency, in forced colours, and in native windows."),
                prop("gradient", "Gradient")
                    .doc("Fills the header with a gradient from `color`, as on `Paper`: `(\"info\", 90)` or `Gradient::default().to(\"info\").deg(90)`; `Gradient::default()` is the theme's. The text colour and focus rings are picked to read on both stops. With `glass`, the stops turn translucent."),
                prop("z_index", "ThemeAwareValue")
                    .default("100")
                    .doc("Stacking order."),
                prop("children", "Element").doc("Nav and actions."),
            ])],
            accessibility: a11y()
                .handles([
                    "Draws a `header` element, which screen readers list as the page's banner when it sits at the top level.",
                    "With `publish_height`, an element that gets focus under a `sticky` or `fixed` header scrolls clear of it (WCAG 2.4.11). This covers the page's own scroller only.",
                ])
                .must([
                    "Put the page's header at the top level, not inside `main`, `nav`, `section`, `article` or `aside`. Use one per page.",
                    "Put a `nav` inside it for the site's links.",
                    "Set `publish_height` on a `sticky` or `fixed` header.",
                    "For a header stuck inside another scroller, set `scroll-padding-top: var(--lsx-header-height)` on that scroller yourself.",
                ])
                .example("A page banner: `Header { publish_height: true, nav { .. } }` as the app's first child, outside `main`. A screen reader lists a banner with the navigation in it, and a link Tab focuses near the top scrolls into view instead of under the bar."),
            lead: rsx! {
                Text {
                    "The page's banner landmark, always a "
                    Code { source: "header" }
                    " element. This site's own header is one. Scroll the frame to compare "
                    "the positions. "
                    Code { source: "sticky" }
                    " pins to the top of the frame and "
                    Code { source: "static" }
                    " scrolls away. "
                    Code { source: "fixed" }
                    " pins to the viewport, so the demo leaves it out."
                }
            },
            Demo {
                component: "Header",
                wide_preview: true,
                children_text: "Libero",
                controls: vec![
                    Control::toggle("position", ["sticky", "static"]).labels(["Sticky", "Static"]),
                    Control::sizes("size")
                        .default("md"),
                    // Unset is the paper surface, the real default, so that prints nothing.
                    Control::color("color").with_unset()
                    .code(|_, values| match values.str("color").as_str() {
                        UNSET => vec![],
                        color => vec![format!("color: {color:?}")],
                    }),
                    Control::switch("glass"),
                ],
                render: move |values: DemoValues| rsx! {
                    Box {
                        sx: sx()
                            .height("200px")
                            .width("100%")
                            .position("relative")
                            .z_index("0")
                            .border("1px solid var(--lsx-muted-3)"),
                        ScrollArea {
                            aria_label: "Header demo",
                            Header {
                                position: values.str("position"),
                                size: or_unset(values.str("size")),
                                color: match values.str("color").as_str() {
                                    UNSET => Input::None,
                                    color => Input::from(color),
                                },
                                glass: values.str("glass") == "true",
                                "Libero"
                            }
                            Box {
                                sx: sx().padding("md"),
                                for i in 0..12 {
                                    Text { key: "{i}", "Scroll me, line {i}" }
                                }
                            }
                        }
                    }
                },
                wrap: Wrap(wrap_frame),
            }
        }
    }
}
