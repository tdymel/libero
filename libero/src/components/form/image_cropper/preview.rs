use dioxus::prelude::*;

use super::crop::{Backdrop, CropRect, CropShape};
use super::image_cropper::backdrop_fill;
use crate::{
    components::{
        common::{HtmlTag, Input, States, Variables, base_props, media_url, variables},
        layout::use_box,
    },
    sx::{StaticSx, sx},
    theme::CssVar,
};

/// The preview's width and height in px.
const WIDTH: CssVar = CssVar::new("--lsx-image-crop-preview-width");
const HEIGHT: CssVar = CssVar::new("--lsx-image-crop-preview-height");
/// The picture's box, in percent of the preview: the crop fills it.
const LEFT: CssVar = CssVar::new("--lsx-image-crop-preview-left");
const TOP: CssVar = CssVar::new("--lsx-image-crop-preview-top");
const SCALE_X: CssVar = CssVar::new("--lsx-image-crop-preview-scale-x");
const SCALE_Y: CssVar = CssVar::new("--lsx-image-crop-preview-scale-y");
const FILL: CssVar = CssVar::new("--lsx-image-crop-preview-backdrop");

static IMAGE_CROP_PREVIEW_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative")
        .display("inline-block")
        .flex("none")
        .width(WIDTH.value())
        .height(HEIGHT.value())
        .overflow("hidden")
        .line_height("0")
        .background(FILL.value())
        .forced_color_adjust("none")
        .selector(
            "& > img",
            sx().position("absolute")
                .left(LEFT.value())
                .top(TOP.value())
                .width(SCALE_X.value())
                .height(SCALE_Y.value())
                .max_width("none")
                .pointer_events("none"),
        )
        // The square past the circle stays in sight, dimmed: what will stick out of it.
        .selector(
            "& > [data-slot='ring']",
            sx().position("absolute")
                .top("0")
                .left("0")
                .right("0")
                .bottom("0")
                .border_radius("50%")
                .box_shadow("0 0 0 9999px rgba(0, 0, 0, 0.55)")
                .pointer_events("none"),
        )
});

base_props! {
    pub struct ImageCropPreviewProps {
        /// The picture the cropper shows, as its `src`.
        #[props(into)]
        src: String,
        #[props(into)]
        alt: String,
        /// The crop, as the cropper's `value`: past the picture's edge shows `backdrop`.
        value: CropRect,
        /// `Circle` dims the corners past the circle, so what sticks out of it shows.
        #[props(default)]
        shape: CropShape,
        /// The crop's width over its height in px, as the cropper's `aspect`. Default square.
        #[props(default)]
        aspect: Option<f64>,
        /// The longer side in px. Default 96.
        #[props(default)]
        size: Option<f64>,
        /// What shows past the picture's edge.
        #[props(default)]
        backdrop: Backdrop,
    }
}

/// The crop an [`ImageCropper`](super::ImageCropper) picks, live: give it the same `src`,
/// `value` and `aspect`, and put it anywhere.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{CropRect, CropShape, ImageCropPreview, ImageCropper};
/// # fn app() -> Element {
/// let mut crop = use_signal(|| CropRect::FULL);
/// rsx! {
///     ImageCropper {
///         src: "/photo.jpg",
///         alt: "Your photo",
///         aspect: 1.0,
///         shape: CropShape::Circle,
///         value: crop(),
///         onchange: move |rect| crop.set(rect),
///     }
///     ImageCropPreview { src: "/photo.jpg", alt: "Your avatar", value: crop(), shape: CropShape::Circle }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/image-cropper>
#[component]
pub fn ImageCropPreview(props: ImageCropPreviewProps) -> Element {
    let own_src = media_url(&props.src, "ImageCropPreview `src`").unwrap_or_default();
    let valid = |value: &f64| value.is_finite() && *value > 0.0;
    let aspect = props.aspect.filter(valid).unwrap_or(1.0);
    let size = props.size.filter(valid).unwrap_or(96.0);
    let (width, height) = match aspect >= 1.0 {
        true => (size, size / aspect),
        false => (size * aspect, size),
    };
    let rect = props.value;
    let shown = rect.width > 0.0 && rect.height > 0.0 && rect.is_finite();
    let percent = |value: f64| format!("{}%", value * 100.0);
    let mut look = variables()
        .with(WIDTH, format!("{width}px"))
        .with(HEIGHT, format!("{height}px"))
        .with(FILL, backdrop_fill(props.backdrop));
    if shown {
        look = look
            .with(LEFT, percent(-rect.x / rect.width))
            .with(TOP, percent(-rect.y / rect.height))
            .with(SCALE_X, percent(1.0 / rect.width))
            .with(SCALE_Y, percent(1.0 / rect.height));
    }
    let look: Input<Variables> = look.into();
    let states: Input<States> = props.states.clone().unwrap_or_default().into();

    use_box()
        .framework_sx(&IMAGE_CROP_PREVIEW_SX)
        .class(&props.class)
        .visibility(&props.hidden_from, &props.visible_from)
        .sx(&props.sx)
        .states(&states)
        .variables(&look)
        .prepare()
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                if shown {
                    img { src: own_src, alt: props.alt, draggable: "false" }
                }
                if props.shape == CropShape::Circle {
                    div { "data-slot": "ring", "aria-hidden": "true" }
                }
            },
        )
}
