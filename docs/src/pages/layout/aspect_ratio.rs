use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{AspectRatio, Code, Flex, Text},
    sx::sx,
};

/// The child is the fixture - `ratio` is the only prop - so the code block
/// prints it verbatim.
const CHILD: &str = r#"Flex {
    sx: sx().background("primary").color("primary-contrast"),
    align: "center",
    justify: "center",
    "The child fills the box"
}"#;

/// A ratio only shows against a known width; the box has no size of its own.
// snippet: in AspectRatio { .. }
const SX: &str = r#"sx: sx().width("240px")"#;

#[component]
pub fn AspectRatioPage() -> Element {
    rsx! {
        DocPage {
            title: "AspectRatio",
            source: "libero/src/components/layout/aspect_ratio.rs",
            markdown: "/md/aspect_ratio.md",
            properties: vec![props("AspectRatio", vec![
                prop("ratio", "f32")
                    .default("1.0")
                    .doc("Width-to-height ratio, e.g. `16.0 / 9.0`."),
                prop("children", "Element").doc("The child that fills the box; an image or a video is cropped to it. An `a`, `picture` or `span` child becomes a block; give any other inline child `display: block`, and a `picture`'s inner `img` the full size. The size and `object-fit` reach the direct child only: an image inside a wrapper needs `width: 100%`, `height: 100%` and `object-fit: cover` itself. A text child is never clipped: when it needs more room, at larger text or a narrow width, it grows the box taller than its ratio."),
            ])],
            accessibility: a11y()
                .handles([
                    "A focused child keeps its focus ring: it is drawn inside the child's edges.",
                    "A text child grows the box past its ratio at larger text, so nothing is cut off.",
                ])
                .must([
                "Keep nothing meaningful at an image's or a video's edges: they get cropped.",
                "Leave room below a text child: at larger text it grows the box taller than its ratio.",
                "Give an image `alt` text that describes what the reader can see.",
            ])
                .example("A video thumbnail in `AspectRatio { ratio: 16.0 / 9.0, .. }` with an image whose `alt` is \"Sunset over the harbour\": the picture is cropped at its edges, and the play button in its centre keeps its focus ring."),
            lead: rsx! {
                Text {
                    "Holds its child to a width-to-height ratio, filling the box. Write "
                    Code { source: "ratio" }
                    " as a division, like "
                    Code { source: "16.0 / 9.0" }
                    ". The box has no size of its own, so give it a width. An image or a "
                    "video is cropped with "
                    Code { source: "object-fit: cover" }
                    ", not stretched. A text child that needs more room grows the box past "
                    "its ratio instead of being cut off."
                }
            },
            Demo {
                component: "AspectRatio",
                children_text: "",
                children_code: CHILD.to_string(),
                fixed: vec![SX.to_string()],
                controls: vec![
                    // The options are the rsx expressions themselves, so
                    // the code block prints an unquoted `f32`.
                    Control::slider(
                        "ratio",
                        ["3.0 / 4.0", "1.0", "4.0 / 3.0", "16.0 / 9.0", "21.0 / 9.0"],
                    )
                    .labels(["3 / 4", "1 / 1", "4 / 3", "16 / 9", "21 / 9"])
                    .default("1.0")
                    .code(|control, values| {
                        let value = values.str("ratio");
                        match value == control.default {
                            true => vec![],
                            false => vec![format!("ratio: {value}")],
                        }
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    AspectRatio {
                        ratio: match values.str("ratio").as_str() {
                            "3.0 / 4.0" => 3.0 / 4.0,
                            "4.0 / 3.0" => 4.0 / 3.0,
                            "16.0 / 9.0" => 16.0 / 9.0,
                            "21.0 / 9.0" => 21.0 / 9.0,
                            _ => 1.0,
                        },
                        sx: sx().width("240px"),
                        Flex {
                            sx: sx().background("primary").color("primary-contrast"),
                            align: "center",
                            justify: "center",
                            "The child fills the box"
                        }
                    }
                },
            }
        }
    }
}
