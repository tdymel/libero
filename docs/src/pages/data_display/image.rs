use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, Image, Input, Text},
    sx::sx,
    theme::Size,
    use_theme,
};

/// `alt` is required, and the box has to be sized before `fit` means
/// anything. Neither is a control, so both print as `fixed`.
const FIXED: [&str; 2] = [
    r#"alt: "A stylised landscape""#,
    r#"sx: sx().width("160px").height("160px").background("muted.1")"#,
];

const MISSING_SRC: &str = "/does-not-exist.png";

#[component]
pub fn ImagePage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Image",
            source: "libero/src/components/data_display/image.rs",
            markdown: "/md/image.md",
            properties: vec![
                props("Image", vec![
                    prop("src", "String").default("required").doc("The image source."),
                    prop("fallback_src", "Option<String>")
                        .default("None")
                        .doc("Shown in place of `src` once it fails to load."),
                    prop("zoomed_src", "Option<String>")
                        .default("None")
                        .doc("A larger source for the zoom overlay. Falls back to `src`."),
                    prop("fit", "ImageFit").default("cover").doc("Maps onto `object-fit`."),
                    prop("radius", "Size")
                        .default("0")
                        .doc("Corner radius, a step on the radius scale. Any other value goes through `sx`."),
                    prop("alt", "Option<String>")
                        .default("None")
                        .doc("What the picture shows. A debug build warns when neither `alt` nor `decorative` is set."),
                    prop("decorative", "bool")
                        .default("false")
                        .doc("Marks the picture as decoration, hidden from screen readers. Wins over `alt`, with a warning in a debug build."),
                    prop("zoomable", "bool")
                        .default("false")
                        .doc("Opens the picture in a single-picture Lightbox on click. Ignored inside a linked `ImageItem`, with a warning."),
                    prop("loading", "ImageLoading")
                        .default("eager")
                        .doc("The `<img>`'s `loading`. `lazy` loads the picture only when it nears the viewport."),
                ]),
            ],
            accessibility: a11y()
                .key(["Enter", "Space"], "On a zoomable image: opens the overlay.")
                .key(["Escape"], "Closes the overlay, as do the backdrop and the Close button.")
                .handles([
                    "`decorative` renders `alt=\"\"` and `role=\"presentation\"`.",
                    "A zoomable image is a button named after its `alt`, \"Zoom in: <alt>\" (the localization's `image.zoom_named`).",
                    "An image with neither `alt` nor `decorative` warns in a debug build and renders no `alt`, so a checker still flags it.",
                ])
                .must(["Give every image an `alt`, or set `decorative` for one that carries nothing."]),
            lead: rsx! {
                Text {
                    "An "
                    Code { source: "<img>" }
                    " with a fallback source for when it fails to load, rounded corners and "
                    "an optional click-to-zoom overlay. It fills its box, so size the box. "
                    Code { source: "fit" }
                    " maps straight onto "
                    Code { source: "object-fit" }
                    ", and "
                    Code { source: "radius" }
                    " takes a step on the radius scale."
                }
            },
            Demo {
                component: "Image",
                children_text: "",
                fixed: FIXED.map(str::to_string).to_vec(),
                controls: vec![
                    Control::select(
                        "fit",
                        ["fill", "contain", "cover", "none", "scale-down"],
                    )
                    .labels(["Fill", "Contain", "Cover", "None", "Scale down"])
                    .default(theme.image.fit.as_str()),
                    // The theme's `0` is a CSS length, not a step: `none`
                    // leaves the prop unset.
                    Control::slider("radius", ["none", "xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("none")
                        .code(|_, values| match values.str("radius").as_str() {
                            "none" => vec![],
                            radius => vec![format!("radius: {radius:?}")],
                        }),
                    Control::switch("zoomable"),
                    // Drives `src` too: a fallback only shows once the
                    // real source fails, so the switch has to break it.
                    Control::switch("broken_src").code(|_, values| {
                        let src = match values.str("broken_src").as_str() {
                            "true" => format!("{MISSING_SRC:?}"),
                            _ => "SAMPLE_IMAGE".to_string(),
                        };
                        vec![
                            format!("src: {src}"),
                            "fallback_src: FALLBACK_IMAGE".to_string(),
                        ]
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    Image {
                        src: match values.str("broken_src").as_str() {
                            "true" => MISSING_SRC.to_string(),
                            _ => crate::SAMPLE_IMAGE.to_string(),
                        },
                        fallback_src: crate::FALLBACK_IMAGE.to_string(),
                        alt: "A stylised landscape",
                        fit: values.str("fit"),
                        radius: match values.str("radius").as_str() {
                            "none" => Input::None,
                            radius => Input::from(Size::from(radius)),
                        },
                        zoomable: values.str("zoomable") == "true",
                        sx: sx().width("160px").height("160px").background("muted.1"),
                    }
                },
            }
        }
    }
}
