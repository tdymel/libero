use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use crate::samples::SAMPLE_IMAGE;
use dioxus::prelude::*;
use libero::{
    components::{
        Backdrop, Box, Code, CropRect, CropShape, Flex, ImageCropPreview, ImageCropper,
        ImageCropperControls, ImageCropperPart, Text,
    },
    platform::image::{
        CutOptions, ImageType, crop_image, file_from_bytes, has_transparency, image_size,
    },
    sx::sx,
    utils::data_url,
};

/// The sample's bytes, so the demo can cut it as a picked file.
const SAMPLE_SVG: &[u8] = include_bytes!("../../../assets/sample.svg");

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
/// largest box `aspect` allows, its corner in whole percent.
fn starting(aspect: Option<f64>) -> CropRect {
    let (width, height) = match aspect.map(|aspect| aspect / SAMPLE_RATIO) {
        Some(ratio) if ratio <= 1.0 => (ratio, 1.0),
        Some(ratio) => (1.0, 1.0 / ratio),
        None => (1.0, 1.0),
    };
    let percent = |fraction: f64| (fraction * 100.0).round() / 100.0;
    let (width, height) = (width * 0.8, height * 0.8);
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
                    prop("src", "String").default("required")
                        .doc("The image: any URL, a `data:` URL included. The browser fetches it with a plain GET and the page's cookies, so a URL built from user data can act as a tracking beacon. A `javascript:` or `vbscript:` URL loads nothing and warns in debug builds."),
                    prop("alt", "String").default("required").doc("Describes the image."),
                    prop("value", "Option<CropRect>")
                        .doc("The box, in fractions of the image. Pair it with `onchange`. Unset starts centred at 80% of the largest box `aspect` allows, so it can move at once, and reports it once the image has loaded. A new `src` or `aspect` starts an unset box over. A set `value` stays as it is, so reset it yourself."),
                    prop("onchange", "EventHandler<CropRect>")
                        .doc("Fires on every move of the box, by a drag, a key or a button under the image. Without it the cropper only shows, with no buttons."),
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
                        .doc("Holds the box still, centred, and moves the image under it, as a phone's profile picture cropper does: a drag pans the image, a pinch, the wheel or the + and - keys zoom it. No resize handles; `value` stays the crop in fractions of the image. With an `aspect` on a picture of another shape the stage takes the box's shape, the picture centred on it as on a square, and `overhang` counts from that stage, so a wide picture zooms out until its width fits the box."),
                    prop("controls", "ImageCropperControls")
                        .default("Shown")
                        .doc("The buttons under the image, and with `pan` the zoom bar. `HiddenUntilFocus` (what `false` converts to) hides them from sight only: they show again while focus is inside them, so a keyboard and a screen reader keep them. `None` removes them; the box's keys stay."),
                    prop("overhang", "f64")
                        .default("0.0")
                        .doc("How far the box may reach past each side of the picture, a fraction of its size, 0 to 1: room to centre a logo or zoom out on a square. The cropper reserves that room round the picture and draws `backdrop` in it; `value` leaves 0 to 1 by as much."),
                    prop("backdrop", "Backdrop")
                        .default("Checkerboard")
                        .doc("What shows past the picture's edge, and behind its see-through pixels: a `Checkerboard` or a flat `Color` (`Backdrop::rgb(255, 255, 255)`). A flat colour is also what the cut paints there; a checkerboard leaves it transparent."),
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
                    (ImageCropperPart::Backdrop, "The checkerboard or colour behind the picture and its overhang; drawn with an `overhang` or a flat colour."),
                    (ImageCropperPart::Mask, "The dimmed image outside the box; a drag on it moves the box."),
                    (ImageCropperPart::Box, "The crop box, a tab stop."),
                    (ImageCropperPart::Frame, "Over the box: takes its drags and holds the handles."),
                    (ImageCropperPart::Handle, "One of the eight resize handles."),
                    (ImageCropperPart::Zoom, "With `pan`: the bar under the image holding the zoom slider."),
                    (ImageCropperPart::Controls, "The buttons under the image that move the box and, without `pan`, make it smaller or larger."),
                ]),
                props("ImageCropPreview", vec![
                    prop("src", "String").default("required").doc("The picture the cropper shows, as its `src`."),
                    prop("alt", "String").default("required").doc("Describes the crop."),
                    prop("value", "CropRect").default("required").doc("The crop, as the cropper's `value`; live when both read one signal. Past the picture's edge shows `backdrop`."),
                    prop("shape", "CropShape").default("Rect").doc("`Circle` dims the corners past the circle, so what sticks out of an avatar shows."),
                    prop("aspect", "f64").default("1.0").doc("The crop's width over its height in pixels, as the cropper's `aspect`."),
                    prop("size", "f64").default("96.0").doc("The longer side in pixels."),
                    prop("backdrop", "Backdrop").default("Checkerboard").doc("What shows past the picture's edge."),
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
                    prop("overhang", "f64").default("0.0").doc("As `overhang` above."),
                    prop("backdrop", "Backdrop").default("Checkerboard").doc("As `backdrop` above; also what fills the cut past the picture."),
                ])
                .without_base_props(),
                props("CropEvent", vec![
                    prop("rect", "CropRect").doc("The box the `FileField` crop dialog applied."),
                    prop("original", "FileData").doc("The file as picked, before the cut."),
                ])
                .without_base_props(),
                props("libero::platform::image", vec![
                    prop("file_from_bytes(name, content_type, bytes)", "Option<FileData>")
                        .doc("The bytes as a file a form can post. `None` where no page holds files: a server render, Blitz."),
                    prop("crop_image(file, rect, options).await", "Option<FileData>")
                        .doc("The file cut to a `CropRect`, past the picture's edge too, as `CutOptions` say. `None` where no page can cut it (a server render, so call it from an event) or the picture would not decode."),
                    prop("on_square(file, margin, options).await", "Option<FileData>")
                        .doc("The picture centred on a square canvas, `margin` of its longer side free on every side, cut as `crop_image` cuts."),
                    prop("has_transparency(source).await", "Option<bool>")
                        .doc("Whether any pixel is not fully opaque, of a `&FileData` or a URL (`ImageSource`). A JPEG, or a PNG or lossy WebP without alpha, answers from its header; otherwise it reads the pixels and stops at the first see-through one. `None` when the URL would not load or the picture would not decode."),
                    prop("image_size(source).await", "Option<(u32, u32)>")
                        .doc("The width and height in pixels of a `&FileData` or a URL, turned as its EXIF says."),
                ])
                .without_base_props(),
                props("CutOptions", vec![
                    prop("max_size", "Option<u32>").doc("Scales the cut down so its longer side is at most this many pixels."),
                    prop("backdrop", "Backdrop").default("Checkerboard").doc("What fills past the picture's edge and, with a `Circle`, past the circle. A checkerboard leaves it transparent; a JPEG gets white."),
                    prop("shape", "CropShape").default("Rect").doc("`Circle` fills everything outside the ellipse inscribed in the cut with `backdrop`: a round avatar."),
                    prop("output", "ImageType").default("Keep").doc("The type written: `Keep` keeps a PNG, JPEG or WebP's and makes anything else a PNG; `Png`, `Jpeg` or `Webp` picks one, so a JPEG cut past its edge can stay see-through as a PNG."),
                ])
                .without_base_props(),
                props("ImageSource", vec![
                    prop("&FileData", "ImageSource::File").doc("A picked or built file."),
                    prop("&str, &String", "ImageSource::Url").doc("Loaded with the page's `fetch` on the web and in a WebView, so a cross-origin URL needs CORS; Blitz decodes `data:` itself and loads `http(s)` through its network provider. `javascript:`, `file:` and the like load nothing."),
                ])
                .without_base_props(),
            ],
            accessibility: a11y()
                .key(["Arrow"], "On the box: moves it, with `pan` the image, as a drag does. On a corner: moves that corner, resizing the box.")
                .key(["Shift+Arrow"], "Ten times as far.")
                .key(["Home", "End"], "On the box: moves it to the left or right edge, with `pan` the image. On a corner: moves its left or right side as far left or right as it goes.")
                .key(["+", "=", "-"], "With `pan`: + or = zooms the image in, - zooms it out, around the box's centre.")
                .handles([
                    "The box is a `slider` tab stop named by `aria_label`, its value spoken as \"50% by 50%, at 25%, 25%\".",
                    "The four corners are tab stops of their own, sliders named \"Top left corner\" and so on.",
                    "Each slider's number rises with ArrowRight: the box's left edge, with `pan` the image's; a left corner's left edge, a right corner's width.",
                    "Each handle is a 24px target around a 12px square, 44px on a touch screen.",
                    "A drag focuses the box or the corner it grabbed, so the keys carry on from there.",
                    "On a touch screen a finger on the image, on the box or beside it, drags the box without scrolling the page; a disabled cropper lets the page scroll. Two fingers pinch the box larger or smaller around its centre; so do Ctrl or Cmd with the mouse wheel and a trackpad pinch over the image. A plain wheel scrolls the page.",
                    "With `pan` the whole cropper takes touches: one finger pans the image, two zoom it around their midpoint. The mouse wheel and a trackpad pinch zoom it around the pointer. Its value adds the zoom: \"40% by 80%, at 30%, 10%, zoom 100%\".",
                    "With `pan` a slider named \"Zoom\" under the image zooms around the box's centre, by one pointer or its keys, so no pinch is needed (WCAG 2.5.1). Every step zooms by the same factor. Its value is the zoom, \"100%\", the same number the box says. Minus and plus glyphs, hidden from assistive tech, mark its ends; a press on one zooms a step, as - and + do, and leaves focus where it is. One that would change nothing dims.",
                    "Under the image a group named \"Crop area controls\" holds buttons for every drag (WCAG 2.5.7): \"Move left\", \"Move up\", \"Move down\" and \"Move right\" move the box 5% of the image, with `pan` the image under it; \"Smaller\" and \"Larger\" scale the box around its centre, without `pan`. A button whose press would leave the box as it is, such as \"Move left\" at the left edge, is `aria-disabled` and stays focusable. With `controls: false` the buttons and the zoom bar stay out of sight until focus reaches them; with `ImageCropperControls::None` they are gone and only the keys remain.",
                    "With an `aspect`, a corner key resizes both sides together.",
                    "The box and the corners describe their keys.",
                ])
                .must([
                    "Describe the image with `alt`.",
                    "Translate the corner names and the spoken value with the localization.",
                ])
                .example("An avatar crop, `ImageCropper { alt: \"Your photo\", aria_label: \"Crop area\", aspect: 1.0, .. }`: Tab reaches the crop box, the arrows move it, and a screen reader hears where it sits, such as \"50% by 50%, at 25%, 25%\".")
                .limits([
                    "The edge handles are pointer-only: the corners reach every size.",
                    "A screen reader hears where the box is, not what it shows.",
                    "Under Blitz a `FileField` with `crop` passes an AVIF through uncropped, with no dialog.",
                    "Under Blitz a cut WebP is lossless, so it can be larger than the browser's lossy one.",
                    "The overhang is room round the picture, so it takes layout space the picture's own size does not: a picture fills the container less by that much.",
                    "Under Blitz the `platform::image` helpers read PNG, JPEG, WebP and BMP only: an SVG gives `None`.",
                    "Under Blitz an image takes no `aspect-ratio`, so `pan` with an `aspect` keeps the picture's own stage there.",
                ]),
            lead: rsx! {
                Text {
                    "A box with handles over an image, picking the part to keep. Drag the box, "
                    "a handle or the image around the box, pinch with two fingers, or use the "
                    "arrow keys on the box and its corners. "
                    Code { source: "value" }
                    " is a "
                    Code { source: "CropRect" }
                    " in fractions of the image, so it fits any resolution; "
                    Code { source: "to_pixels" }
                    " turns it into pixels. It only picks the box. To cut the picture, give a "
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
                    Control::toggle("overhang", ["none", "25", "50"])
                        .labels(["None", "25%", "50%"])
                        .default("25")
                        .code(|_, values| match values.str("overhang").as_str() {
                            "25" => vec!["overhang: 0.25".to_string()],
                            "50" => vec!["overhang: 0.5".to_string()],
                            _ => vec![],
                        }),
                    Control::toggle("backdrop", ["checkerboard", "white", "dark"])
                        .labels(["Checkerboard", "White", "Dark"])
                        .default("checkerboard")
                        .code(|_, values| match values.str("backdrop").as_str() {
                            "white" => vec!["backdrop: Backdrop::rgb(255, 255, 255)".to_string()],
                            "dark" => vec!["backdrop: Backdrop::rgb(34, 36, 40)".to_string()],
                            _ => vec![],
                        }),
                    Control::toggle("controls", ["shown", "hidden", "none"])
                        .labels(["Shown", "Hidden until focus", "None"])
                        .default("shown")
                        .code(|_, values| match values.str("controls").as_str() {
                            "hidden" => vec!["controls: false".to_string()],
                            "none" => vec!["controls: ImageCropperControls::None".to_string()],
                            _ => vec![],
                        }),
                    Control::switch("pan"),
                    Control::switch("disabled"),
                    // Only the cut under the cropper: no prop of its own.
                    Control::toggle("cut", ["png", "jpeg"])
                        .labels(["PNG", "JPEG"])
                        .default("png")
                        .code(|_, _| vec![]),
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
    let shape = match values.str("shape").as_str() {
        "circle" => CropShape::Circle,
        _ => CropShape::Rect,
    };
    let backdrop = match values.str("backdrop").as_str() {
        "white" => Backdrop::rgb(255, 255, 255),
        "dark" => Backdrop::rgb(34, 36, 40),
        _ => Backdrop::Checkerboard,
    };
    let options = CutOptions {
        max_size: Some(96),
        backdrop,
        shape,
        output: match values.str("cut").as_str() {
            "jpeg" => ImageType::Jpeg,
            _ => ImageType::Png,
        },
    };
    // `crop_image` on the box as it moves, then `image_size` and `has_transparency` on the cut.
    let cut = use_resource(use_reactive!(|(rect, options)| async move {
        let file = file_from_bytes("sample.svg", "image/svg+xml", SAMPLE_SVG.to_vec())?;
        let cut = crop_image(&file, rect, options).await?;
        let (width, height) = image_size(&cut).await?;
        let clear = match has_transparency(&cut).await? {
            true => "see-through",
            false => "opaque",
        };
        let kind = cut.content_type().unwrap_or_default();
        Some((
            data_url(&cut).await?,
            format!("{width} x {height} px {kind}, {clear}"),
        ))
    }));

    rsx! {
        Flex { direction: "column", gap: "sm",
            // The live preview beside the cropper, under it on a narrow screen.
            Flex { direction: "row", gap: "md", align: "flex-start", wrap: "wrap",
            // Holds the 16:9 picture and the 38px controls row until it loads, so the page below does not jump.
            Box { sx: sx().flex("1").min_width("0").min_height("calc(min(360px, (100vw - 50px) * 9 / 16) + 38px)"),
            ImageCropper {
                src: SAMPLE_IMAGE,
                alt: "A sample landscape",
                aspect,
                shape,
                pan: values.str("pan") == "true",
                controls: match values.str("controls").as_str() {
                    "hidden" => ImageCropperControls::HiddenUntilFocus,
                    "none" => ImageCropperControls::None,
                    _ => ImageCropperControls::Shown,
                },
                overhang: match values.str("overhang").as_str() {
                    "25" => 0.25,
                    "50" => 0.5,
                    _ => 0.0,
                },
                backdrop,
                disabled: values.str("disabled") == "true",
                value: rect,
                onchange: move |next: CropRect| crop.set(Some((key.clone(), next))),
            }
            }
            ImageCropPreview {
                src: SAMPLE_IMAGE,
                alt: "The crop",
                value: rect,
                shape,
                // Free: the box's own shape in pixels.
                aspect: aspect.unwrap_or(rect.width * SAMPLE_RATIO / rect.height),
                backdrop,
            }
            }
            Text { size: "sm",
                bdi { dir: "ltr",
                    "{pixels.width} x {pixels.height} px at {pixels.x}, {pixels.y} of 1600 x 900"
                }
            }
            // Its height held, so the page does not jump when the first cut lands.
            Flex { direction: "row", gap: "sm", align: "center", sx: sx().min_height("96px"),
                if let Some(Some((src, about))) = cut() {
                    img { src, alt: "The cut", style: "max-width: 96px; max-height: 96px" }
                    Text { size: "sm", "Cut: {about}" }
                }
            }
        }
    }
}
