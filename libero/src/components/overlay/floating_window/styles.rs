use super::options::FloatingWindowPart;
use crate::{
    components::{
        common::{Part, focus_ring_sx, inset_focus_ring_sx},
        layout::paper_sx,
    },
    hooks::drag_handle_sx,
    sx::{StaticSx, sx},
    theme::{PAPER_BORDER_COLOR, Size, SizeCss},
};

// `paper_sx()`, with the parts keyed by `data-slot` under one class.
pub(super) static WINDOW_SX: StaticSx = StaticSx::new(|| {
    paper_sx()
        // The resize handle's containing block.
        .position("relative")
        .display("flex")
        .flex_direction("column")
        // A caller's max replaces these; the `Float` wrapper caps it again.
        .max_width("100dvw")
        .max_height("100dvh")
        .overflow("hidden")
        .selector("&:focus-visible", focus_ring_sx())
        .selector(
            FloatingWindowPart::TitleBar.selector(),
            sx().display("flex")
                .align_items("center")
                .gap("sm")
                .padding(format!(
                    "{} {}",
                    SizeCss::SPACING.value(Size::Xs),
                    SizeCss::SPACING.value(Size::Sm)
                ))
                .border_bottom(format!("1px solid {}", PAPER_BORDER_COLOR.value())),
        )
        .selector(
            FloatingWindowPart::Handle.selector(),
            sx().flex("1")
                .min_width("0")
                .min_height("1.5em")
                // A long word wraps instead of being clipped by the window (1.4.10).
                .selector("& h2", sx().margin("0").with("overflow-wrap", "anywhere"))
                .selector("&:focus-visible", inset_focus_ring_sx("-2px")),
        )
        .selector(
            format!("{}[tabindex]", FloatingWindowPart::Handle.selector()),
            drag_handle_sx().cursor("move").user_select("none"),
        )
        .selector(
            FloatingWindowPart::Steps.selector(),
            sx().display("flex")
                .flex_wrap("wrap")
                .align_items("center")
                .gap("xs")
                .padding(format!(
                    "{} {}",
                    SizeCss::SPACING.value(Size::Xs),
                    SizeCss::SPACING.value(Size::Sm)
                ))
                .border_bottom(format!("1px solid {}", PAPER_BORDER_COLOR.value())),
        )
        .selector(
            FloatingWindowPart::Body.selector(),
            sx().flex("1 1 auto")
                .min_height("0")
                .overflow("auto")
                .padding("sm"),
        )
        .selector(
            FloatingWindowPart::Resize.selector(),
            drag_handle_sx()
                .position("absolute")
                .right("0")
                .bottom("0")
                .width("14px")
                .height("14px")
                .cursor("nwse-resize")
                // Two short strokes in the corner, the usual grip.
                .background(
                    "linear-gradient(135deg, transparent 55%, currentColor 55%, currentColor 62%, \
                     transparent 62%, transparent 75%, currentColor 75%, currentColor 82%, transparent 82%)",
                )
                .opacity("0.6")
                .selector("&:focus-visible", inset_focus_ring_sx("-2px"))
                // The end corner under RTL is the bottom-left (todo 796).
                .rtl(
                    sx().right("auto").left("0").cursor("nesw-resize").background(
                        "linear-gradient(225deg, transparent 55%, currentColor 55%, currentColor 62%, \
                         transparent 62%, transparent 75%, currentColor 75%, currentColor 82%, transparent 82%)",
                    ),
                ),
        )
});

pub(super) static FLOAT_SX: StaticSx = StaticSx::new(|| {
    sx().width("max-content")
        .max_width("100dvw")
        .max_height("100dvh")
        .display("flex")
        .flex_direction("column")
});
