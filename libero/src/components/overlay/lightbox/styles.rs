//! The viewer's static styles and its private CSS variables.

use crate::{
    components::common::{
        SVG_FIT, inset_focus_ring_sx, ring_overlay_sx, safe_area_padding, svg_fit_sx,
    },
    hooks::drag_handle_sx,
    sx::{FORCED_COLORS, REDUCED_MOTION, StaticSx, sx},
    theme::{
        CssVar, LIGHTBOX_STAGE_HEIGHT, LIGHTBOX_THUMBNAIL_SIZE, LIGHTBOX_THUMBNAILS_GAP,
        LIGHTBOX_WIDTH, Size, SizeCss,
    },
};

/// The current picture's transform, written per instance. Private: nothing
/// themes a zoom.
pub(super) const LIGHTBOX_TRANSFORM: CssVar = CssVar::new("--lsx-lightbox-transform");
/// Thumbnails actually shown at once, so a short strip is only as wide as its
/// thumbnails and centres.
pub(super) const LIGHTBOX_THUMBNAILS_SHOWN: CssVar = CssVar::new("--lsx-lightbox-thumbnails-shown");

/// A phone either way up: held sideways it is 640 to 932px wide but never
/// over 480px tall, so a width query alone misses it.
fn phone() -> String {
    format!(
        "(width < {}), (height < 30rem)",
        Size::Sm.breakpoint_value()
    )
}

/// The dialog's padding on one side of a phone: the usual `sm` plus the safe area.
fn safe_padding(side: &str) -> String {
    safe_area_padding(&SizeCss::SPACING.value(Size::Sm), side)
}

// On a phone the dialog is the whole screen, the stage taking the room left; it
// scrolls when a long caption outgrows it (todo 1307). A double-click zooms, so it selects nothing (todo 917).
pub(super) static LIGHTBOX_DIALOG_SX: StaticSx = StaticSx::new(|| {
    sx().width("100%")
        .user_select("none")
        .max_width(LIGHTBOX_WIDTH.value())
        .padding("sm")
        .media(
            phone(),
            sx().position("fixed")
                .inset("0")
                .max_width("none")
                .margin("0")
                .border_radius("0")
                .box_shadow("none")
                .display("flex")
                .flex_direction("column")
                .overflow_y("auto")
                .padding_top(safe_padding("top"))
                .padding_right(safe_padding("right"))
                .padding_bottom(safe_padding("bottom"))
                .padding_left(safe_padding("left")),
        )
});

// Everything under the dialog's header: the stage, the caption, the strip.
pub(super) static LIGHTBOX_BODY_SX: StaticSx = StaticSx::new(|| {
    sx().media(
        phone(),
        sx().flex("1 0 auto")
            .display("flex")
            .flex_direction("column"),
    )
});

// On a phone the pictures bleed to the safe area's edge, and the stage is a
// size container so a frame is `100cqh` tall. Never under 60dvh: a long caption scrolls the dialog instead.
pub(super) static LIGHTBOX_STAGE_SX: StaticSx = StaticSx::new(|| {
    let bleed = format!("calc(-1 * {})", SizeCss::SPACING.value(Size::Sm));
    sx().media(
        phone(),
        sx().margin_left(bleed.clone())
            .margin_right(bleed)
            .flex("1 0 60dvh")
            .container_type("size"),
    )
});

// A picture fits into this box rather than sizing it. The focus ring sits here:
// on the zoomed picture it would be scaled and clipped.
pub(super) static LIGHTBOX_FRAME_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative")
        .height(LIGHTBOX_STAGE_HEIGHT.value())
        .media(phone(), sx().height("100cqh"))
        .overflow("hidden")
        .selector("& > [data-ring]", ring_overlay_sx())
        .selector(
            "& > :focus-visible ~ [data-ring]",
            inset_focus_ring_sx("-2px"),
        )
        .when(
            SVG_FIT,
            sx().display("flex")
                .align_items("center")
                .justify_content("center"),
        )
});

pub(super) static LIGHTBOX_IMAGE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .height("100%")
        // Shrinks a large picture to the stage and leaves a small one at its
        // own size: an upscaled thumbnail only shows its pixels.
        .object_fit("scale-down")
        // Replaces `Box`'s own ring: the frame draws this one.
        .focus_visible(sx().outline("none"))
        .transform(LIGHTBOX_TRANSFORM.value_or("none"))
        .transition("transform 150ms ease")
        .media(REDUCED_MOTION, sx().transition("none"))
        .when("zoomable", sx().cursor("zoom-in"))
        // The carousel still scrolls sideways natively; a vertical move is
        // left to the swipe-down, two fingers to the pinch.
        .when("sideways", sx().touch_action("pan-x"))
        .when("zoomed", drag_handle_sx().cursor("grab"))
        // A drag is the pointer's own position: easing towards it lags.
        .when("dragging", sx().cursor("grabbing").transition("none"))
        // Natively the box is the picture itself, centred by the frame: Blitz's
        // SVG `contain` then leaves no letterbox to misplace when zoomed (todo 920).
        .when(
            SVG_FIT,
            sx().width("auto")
                .height("auto")
                .max_width("100%")
                .max_height("100%"),
        )
});

// Lifted: Blitz hit-tests a zoomed picture past its frame's clip, so a press
// here panned it (todo 916).
pub(super) static LIGHTBOX_TOOLBAR_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative")
        .z_index("1")
        .display("flex")
        .align_items("center")
        .justify_content("flex-end")
        .gap("sm")
        .margin_bottom("md")
});

// The one real text in the viewer, so it stays selectable.
pub(super) static LIGHTBOX_CAPTION_SX: StaticSx = StaticSx::new(|| {
    sx().margin("0")
        .margin_top("sm")
        .text_align("center")
        .user_select("text")
});

pub(super) static LIGHTBOX_THUMBNAILS_SX: StaticSx = StaticSx::new(|| {
    sx().margin_top("sm")
        .margin_left("auto")
        .margin_right("auto")
        .max_width(format!(
            "calc({shown} * {size} + ({shown} - 1) * {gap})",
            shown = LIGHTBOX_THUMBNAILS_SHOWN.value(),
            size = LIGHTBOX_THUMBNAIL_SIZE.value(),
            gap = LIGHTBOX_THUMBNAILS_GAP.value(),
        ))
});

// The current thumbnail's padding shows a frame. No opacity on the others:
// it would dim their focus ring.
pub(super) static LIGHTBOX_THUMBNAIL_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .aspect_ratio("1")
        .padding("2px")
        .border_width("0")
        .background("transparent")
        .cursor("pointer")
        .when("current", sx().background("primary.6"))
        // Forced colours paint the frame `Canvas`, like the rest.
        .media(
            FORCED_COLORS,
            sx().when("current", sx().background("Highlight")),
        )
        // Inset: the carousel slide around it clips.
        .focus_visible(inset_focus_ring_sx("-2px"))
});

pub(super) static LIGHTBOX_THUMBNAIL_IMAGE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .height("100%")
        .object_fit("cover")
        .when(SVG_FIT, svg_fit_sx())
});
