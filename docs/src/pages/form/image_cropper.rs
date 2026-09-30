use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use crate::site::SAMPLE_IMAGE;
use dioxus::prelude::*;
use libero::components::{Code, CropRect, CropShape, Flex, ImageCropper, ImageCropperPart, Text};

/// `SAMPLE_IMAGE` is 16:9.
const SAMPLE_RATIO: f64 = 16.0 / 9.0;

fn aspect_of(values: &DemoValues) -> Option<f64> {
    match values.str("aspect").as_str() {
        "1" => Some(1.0),
        "4/3" => Some(4.0 / 3.0),
        _ => None,
    }
}

/// Where an unset cropper starts on the sample image: centred, 80% of the
/// largest box `aspect` allows, in whole percent.
fn starting(aspect: Option<f64>) -> CropRect {
    let (width, height) = match aspect.map(|aspect| aspect / SAMPLE_RATIO) {
        Some(ratio) if ratio <= 1.0 => (ratio, 1.0),
        Some(ratio) => (1.0, 1.0 / ratio),
        None => (1.0, 1.0),
    };
    let percent = |fraction: f64| (fraction * 100.0).round() / 100.0;
    let (width, height) = (percent(width * 0.8), percent(height * 0.8));
    CropRect {
        x: percent((1.0 - width) / 2.0),
        y: percent((1.0 - height) / 2.0),
        width,
        height,
    }
}

#[component]
pub fn ImageCropperPage() -> Element {
    rsx! {
        DocPage {
            title: "ImageCropper",
            source: "libero/src/components/form/image_cropper",
            markdown: "/md/image_cropper.md",
            properties: vec![
                props("ImageCropper", vec![
                    prop("src", "String")
                        .doc("The image: any URL, a `data:` URL included."),
                    prop("alt", "String").doc("Describes the image."),
                    prop("value", "Option<CropRect>")
                        .doc("The box, in fractions of the image. Pair it with `onchange`. Unset starts centred at 80% of the largest box `aspect` allows, so it can move at once, and reports it once the image has loaded. A new `src` or `aspect` starts it over."),
                    prop("onchange", "EventHandler<CropRect>")
                        .doc("Fires on every move of the box, by a drag or a key. Without it the cropper only shows."),
                    prop("aspect", "f64")
                        .doc("Locks width over height, in image pixels: `1.0` is square, `16.0 / 9.0` wide. Unset is free."),
                    prop("shape", "CropShape")
                        .default("Rect")
                        .doc("`Circle` masks outside an ellipse, for an avatar. The rect is the same either way."),
                    prop("min_size", "f64")
                        .default("0.05")
                        .doc("The smallest side, a fraction of the image's."),
                    prop("pan", "bool")
                        .default("false")
                        .doc("Holds the box still, centred, and moves the image under it, as a phone's profile picture cropper does: a drag pans the image, a pinch, the wheel or the + and - keys zoom it. No resize handles; `value` stays the crop in fractions of the image."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Dims the cropper and takes no input."),
                    prop("aria_label", "String")
                        .doc("Names the box; the localization's `image_cropper.label` (\"Crop area\") when unset."),
                    prop("onerror", "EventHandler<()>")
                        .doc("Fires when `src` fails to load. The box is not drawn until `src` changes, so the alt text shows."),
                ])
                .parts("ImageCropperPart", vec![
                    (ImageCropperPart::Image, "The image."),
                    (ImageCropperPart::Mask, "The dimmed image outside the box."),
                    (ImageCropperPart::Box, "The crop box, a tab stop."),
                    (ImageCropperPart::Frame, "Over the box: takes its drags and holds the handles."),
                    (ImageCropperPart::Handle, "One of the eight resize handles."),
                    (ImageCropperPart::Zoom, "With `pan`: the bar under the image holding the zoom slider."),
                ]),
                props("CropRect", vec![
                    prop("x, y", "f64").doc("The top left corner, a fraction of the image's width and height."),
                    prop("width, height", "f64").doc("The size, a fraction of the image's. `CropRect::FULL` is the whole image."),
                    prop("to_pixels(width, height)", "PixelRect")
                        .doc("The box in pixels of an image that size, rounded."),
                ])
                .without_base_props(),
                props("CropOptions", vec![
                    prop("aspect", "Option<f64>").doc("As `aspect` above."),
                    prop("shape", "CropShape").default("Rect").doc("As `shape` above."),
                    prop("pan", "bool").default("false").doc("As `pan` above."),
                    prop("max_size", "Option<u32>")
                        .doc("Scales the crop down so its longer side is at most this many pixels."),
                ])
                .without_base_props(),
            ],
            accessibility: a11y()
                .key(["Arrow"], "On the box: moves it. On a corner: moves that corner, resizing the box.")
                .key(["Shift+Arrow"], "Ten times as far.")
                .key(["+", "-"], "With `pan`: zoom the image in or out around the box's centre.")
                .handles([
                    "The box is a `slider` tab stop named by `aria_label`, its value spoken as \"50% by 50%, at 25%, 25%\".",
                    "The four corners are tab stops of their own, sliders named \"Top left corner\" and so on.",
                    "Each handle is a 24px target around a 12px square.",
                    "A drag focuses the box or the corner it grabbed, so the keys carry on from there.",
                    "On a touch screen a finger on the box drags it without scrolling the page; a touch on the image outside it still scrolls. Two fingers on the box pinch it larger or smaller around its centre.",
                    "With `pan` the whole cropper takes touches: one finger pans the image, two zoom it around their midpoint. The mouse wheel and a trackpad pinch zoom it around the pointer. Its value adds the zoom: \"40% by 80%, at 30%, 10%, zoom 100%\".",
                    "With `pan` a slider named \"Zoom\" under the image zooms around the box's centre, by one pointer or its keys, so no pinch is needed (WCAG 2.5.1). Every step zooms by the same factor. Its value is the zoom, \"100%\".",
                    "With an `aspect`, a corner key resizes both sides together.",
                    "The box and the corners describe their keys.",
                ])
                .must([
                    "Describe the image with `alt`.",
                    "Translate the corner names and the spoken value with the localization.",
                ])
                .limits([
                    "The edge handles are pointer-only: the corners reach every size.",
                    "A screen reader hears where the box is, not what it shows.",
                    "Under Blitz a `FileField` with `crop` keeps an AVIF whole: the rect still reaches `oncrop`.",
                ]),
            lead: rsx! {
                Text {
                    "A box with handles over an image, picking the part to keep. Drag the box or "
                    "a handle, pinch the box with two fingers, or use the arrow keys on the box "
                    "and its corners. "
                    Code { source: "value" }
                    " is a "
                    Code { source: "CropRect" }
                    " in fractions of the image, so it fits any resolution; "
                    Code { source: "to_pixels" }
                    " turns it into pixels."
                }
                Text {
                    "It only picks the box. To cut the picture, give a "
                    Code { source: "FileField" }
                    " a "
                    Code { source: "crop" }
                    ": a picked image opens in a cropper first, and the field takes the cut file."
                }
            },
            // snippet: let mut crop = use_signal(|| None::<CropRect>);
            Demo {
                component: "ImageCropper",
                children_text: "",
                // Beside the controls the image drew at 318 px on a 1280 px screen (1343).
                wide_preview: true,
                controls: vec![
                    Control::toggle("aspect", ["free", "1", "4/3"])
                        .labels(["Free", "Square", "4:3"])
                        .default("free")
                        .code(|_, values| {
                            // `src` and the value pair print in every state.
                            let mut code = vec![
                                "src: SAMPLE_IMAGE".to_string(),
                                r#"alt: "A sample landscape""#.to_string(),
                                "value: crop()".to_string(),
                                "onchange: move |rect| crop.set(Some(rect))".to_string(),
                            ];
                            match values.str("aspect").as_str() {
                                "1" => code.push("aspect: 1.0".to_string()),
                                "4/3" => code.push("aspect: 4.0 / 3.0".to_string()),
                                _ => {}
                            }
                            code
                        }),
                    Control::toggle("shape", ["rect", "circle"])
                        .labels(["Rect", "Circle"])
                        .default("rect")
                        .code(|_, values| match values.str("shape").as_str() {
                            "circle" => vec!["shape: CropShape::Circle".to_string()],
                            _ => vec![],
                        }),
                    Control::switch("pan"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    ImageCropperDemo { values }
                },
            }
        }
    }
}

/// Its own component, so the box survives a control change.
#[component]
fn ImageCropperDemo(values: DemoValues) -> Element {
    // The box, with the aspect it was drawn for.
    let mut crop = use_signal(|| None::<(String, CropRect)>);

    let aspect = aspect_of(&values);
    let key = values.str("aspect");
    let rect = match crop() {
        Some((drawn, rect)) if drawn == key => rect,
        _ => starting(aspect),
    };
    let pixels = rect.to_pixels(1600, 900);

    rsx! {
        Flex { direction: "column", gap: "sm",
            ImageCropper {
                src: SAMPLE_IMAGE,
                alt: "A sample landscape",
                aspect,
                shape: match values.str("shape").as_str() {
                    "circle" => CropShape::Circle,
                    _ => CropShape::Rect,
                },
                pan: values.str("pan") == "true",
                disabled: values.str("disabled") == "true",
                value: rect,
                onchange: move |next: CropRect| crop.set(Some((key.clone(), next))),
            }
            Text { size: "sm",
                "{pixels.width} x {pixels.height} px at {pixels.x}, {pixels.y} of 1600 x 900"
            }
        }
    }
}
