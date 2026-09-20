use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, UNSET, Wrap, indent, or_unset, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Header, Input, Text},
    sx::sx,
};

/// The demo frame is the scroll container, so `sticky` pins to its top and
/// `static` scrolls away with the content - the difference only shows once
/// there is more content than the frame is tall. A stacking context, so a
/// native window clips the z-indexed header to it too (881).
fn wrap_frame(_: &DemoValues, code: &str) -> String {
    format!(
        "Box {{\n    sx: sx().height(\"200px\").width(\"100%\").overflow_y(\"auto\")\n        .position(\"relative\").z_index(\"0\")\n        .border(\"1px solid var(--lsx-muted-3)\"),\n{}    Box {{\n        sx: sx().padding(\"md\"),\n        for i in 0..12 {{\n            Text {{ key: \"{{i}}\", \"Scroll me, line {{i}}\" }}\n        }}\n    }}\n}}",
        indent(code)
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
                    .doc("`sticky` pins to the top of the scrolling ancestor, `static` scrolls away. `fixed` pins to the viewport, so offset your content by `var(--lsx-header-height)`."),
                prop("publish_height", "bool")
                    .default("false")
                    .doc("Publishes the height as `--lsx-header-height` and `scroll-padding-top` on `:root`, so focus scrolls clear of a sticky or fixed banner. Set it on the page's own banner only."),
                prop("size", "ThemeAwareValue")
                    .default("md")
                    .doc("Minimum height. The header grows when its content wraps."),
                prop("color", "ThemeAwareValue")
                    .default("none, a neutral background")
                    .doc("Fills the header with shade 6 and a readable text color."),
                prop("glass", "bool")
                    .default("false")
                    .doc("Frosted glass, as on `Paper`: content scrolling under the bar shows through, blurred. It takes the paper surface, so it replaces `color`. Opaque when the user reduces transparency, in forced colours, and in native windows."),
                prop("gradient", "Gradient")
                    .doc("Fills the header with a gradient, as on `Paper`, `Gradient::default()` for the theme's. The text colour and focus rings are picked to read on both stops. It replaces `color`; with `glass`, the stops turn translucent."),
                prop("z_index", "ThemeAwareValue")
                    .default("100")
                    .doc("Stacking order."),
                prop("children", "Element").doc("Nav and actions."),
            ])],
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
                children_text: "Libero",
                controls: vec![
                    Control::toggle("position", ["sticky", "static"]).labels(["Sticky", "Static"]),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    // Opens on the tinted banner, since that is what
                    // `color` is for - but unset (the neutral white one)
                    // is the real default, so it prints nothing.
                    Control::color("color").with_unset()
                    .default("primary")
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
                            .overflow_y("auto")
                            .position("relative")
                            .z_index("0")
                            .border("1px solid var(--lsx-muted-3)"),
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
                },
                wrap: Wrap(wrap_frame),
            }
            DocSection { title: "Accessibility",
                Text {
                    "Keep the page's header outside "
                    Code { source: "main" }
                    ", "
                    Code { source: "nav" }
                    ", "
                    Code { source: "section" }
                    ", "
                    Code { source: "article" }
                    " and "
                    Code { source: "aside" }
                    ", or it is no banner. Give it "
                    Code { source: "publish_height" }
                    " so focus scrolls clear of it. A header stuck inside another scroller "
                    "needs "
                    Code { source: "scroll-padding-top" }
                    " on that scroller, from you."
                }
            }
        }
    }
}
