use crate::components::{
    Control, Demo, DemoValues, DocPage, UNSET, Wrap, indent, or_unset, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Header, Input, Text},
    sx::sx,
};

/// The demo frame is the scroll container, so `sticky` pins to its top and
/// `static` scrolls away with the content - the difference only shows once
/// there is more content than the frame is tall.
fn wrap_frame(_: &DemoValues, code: &str) -> String {
    format!(
        "Box {{\n    sx: sx().height(\"200px\").width(\"100%\").overflow_y(\"auto\")\n        .border(\"1px solid var(--lsx-grey-3)\"),\n{}    Box {{\n        sx: sx().padding(\"md\"),\n        for i in 0..12 {{\n            Text {{ key: \"{{i}}\", \"Scroll me - line {{i}}\" }}\n        }}\n    }}\n}}",
        indent(code)
    )
}

#[component]
pub fn HeaderPage() -> Element {
    rsx! {
        DocPage {
            title: "Header",
            properties: vec![props("Header", vec![
                prop("position", "HeaderPosition")
                    .default("sticky")
                    .doc("Sticky needs no offset; fixed is viewport-relative and you offset your own content, as with Drawer's anchor."),
                prop("size", "ThemeAwareValue")
                    .default("md")
                    .doc("Header height."),
                prop("color", "ThemeAwareValue")
                    .default("unset - neutral background, inherited text")
                    .doc("A set color takes shade 6 and picks its own contrast text."),
                prop("z_index", "ThemeAwareValue")
                    .default("100")
                    .doc("Stacking order."),
                prop("children", "Element").doc("Nav and actions, hosted rather than scoped."),
            ])],
            lead: rsx! {
                Text {
                    "The page's banner landmark - always renders header. This page's own "
                    "header uses one. A set "
                    Code { source: "color" }
                    " takes shade 6 and picks its own contrast text. Scroll the demo frame to "
                    "tell the two positions apart: "
                    Code { source: "sticky" }
                    " pins to the top of the scrolling ancestor, "
                    Code { source: "static" }
                    " scrolls away with the content. "
                    Code { source: "position" }
                    " also accepts "
                    Code { source: "fixed" }
                    ", which is viewport-relative - you offset your own content for it, so "
                    "it is left out of the demo below."
                }
            },
            Demo {
                component: "Header",
                children_text: "Libero",
                controls: vec![
                    Control::toggle("position", ["sticky", "static"]),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    // Opens on the tinted banner, since that is what
                    // `color` is for - but unset (the neutral white one)
                    // is the real default, so it prints nothing.
                    Control::color(
                        "color",
                        [UNSET, "primary", "secondary", "success", "error", "warning"],
                    )
                    .default("primary")
                    .code(|_, values| match values.str("color").as_str() {
                        UNSET => vec![],
                        color => vec![format!("color: {color:?}")],
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    Box {
                        sx: sx()
                            .height("200px")
                            .width("100%")
                            .overflow_y("auto")
                            .border("1px solid var(--lsx-grey-3)"),
                        Header {
                            position: values.str("position"),
                            size: or_unset(values.str("size")),
                            color: match values.str("color").as_str() {
                                UNSET => Input::None,
                                color => Input::from(color),
                            },
                            "Libero"
                        }
                        Box {
                            sx: sx().padding("md"),
                            for i in 0..12 {
                                Text { key: "{i}", "Scroll me - line {i}" }
                            }
                        }
                    }
                },
                wrap: Wrap(wrap_frame),
            }
        }
    }
}
