use crate::{
    components::{
        common::{LogicalTextAlign, focus_ring_sx, ring_overlay_sx},
        form::{drag_over_sx, field_control_sx},
    },
    sx::{StaticSx, sx},
    theme::{
        FILE_FIELD_DROPZONE_HEIGHT, FILE_FIELD_PADDING, FILE_FIELD_RADIUS, FileFieldDefaults,
        SizeCss,
    },
};

/// The `Input` variant's group, the `Select` trigger's shape: the chips, then
/// the Browse button over the rest of the line.
pub(super) static FILE_CONTROL_SX: StaticSx = StaticSx::new(|| {
    field_control_sx()
        .display("flex")
        .align_items("center")
        // The frame's height, not its contents': an empty placeholder left a
        // 0px control that no click or drop could reach (todo 520).
        .align_self("stretch")
        // Lets the control shrink inside the frame, which is what makes the
        // value slot's own ellipsis take effect instead of the frame growing.
        .min_width("0")
        .gap("4px")
        .cursor("pointer")
        .user_select("none")
        .selector(
            "& > [data-slot='value']",
            sx().display("flex")
                .flex("0 1 auto")
                .min_width("0")
                .gap("4px")
                .margin("0")
                .padding("0")
                .list_style("none")
                .overflow("hidden"),
        )
        // A filename has no spaces to break on, so without this one long one
        // pushes the frame past whatever width its parent allows.
        .selector(
            "& [data-slot='name']",
            sx().display("block")
                .min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis")
                .white_space("nowrap"),
        )
        .selector("& [data-slot='chip'] > *", sx().max_width("100%"))
        // The ring goes on what was drawn, not on the item, so it follows that
        // element's own radius.
        .selector("& [data-slot='chip']:focus-visible", sx().outline("none"))
        .selector("& [data-slot='chip']:focus-visible > *", focus_ring_sx())
        .when(
            "multiple",
            sx().selector(
                "& > [data-slot='value']",
                // Chips wrap, and the single-line clip would cut the second
                // row off at the slot's edge.
                sx().flex_wrap("wrap").overflow("visible"),
            ),
        )
        .when("disabled", sx().cursor("not-allowed"))
});

/// The `Input` variant's Browse button: the rest of the line, showing the
/// placeholder while nothing is picked. The frame draws its ring.
pub(super) static FILE_BROWSE_SX: StaticSx = StaticSx::new(|| {
    field_control_sx()
        .display("flex")
        .align_items("center")
        .align_self("stretch")
        .flex("1 1 0")
        // Room to aim at beside a full row of chips.
        .min_width("2em")
        .text_align_start()
        .cursor("pointer")
        .selector(
            "& [data-placeholder]",
            sx().color("text-dimmed")
                .min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis")
                .white_space("nowrap"),
        )
        .selector("&:disabled", sx().cursor("not-allowed"))
});

/// The dropzone's Browse button: the whole surface inside its padding.
pub(super) static FILE_DROPZONE_BROWSE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .align_items("center")
        .justify_content("center")
        .gap("4px")
        .flex("1 1 auto")
        .align_self("stretch")
        .border("none")
        .outline("none")
        .background("transparent")
        .padding("0")
        .color("inherit")
        .font_family("inherit")
        .letter_spacing("inherit")
        .font_size("inherit")
        .text_align("center")
        .cursor("pointer")
        .selector("&:disabled", sx().cursor("not-allowed"))
});

/// The tall surface. Same value, same input, same picker - only the thing the
/// user aims at differs, which is why this is a variant and not a component.
pub(super) static FILE_DROPZONE_SX: StaticSx = StaticSx::new(|| {
    // The surface carries the size and radius states, so the per-size vars
    // resolve here and its children inherit them.
    FileFieldDefaults::theme_vars()
        // Past `per_radius`'s reset: the field's custom radius, set on its wrapper.
        .var(SizeCss::RADIUS.override_var(), "inherit")
        .display("flex")
        .flex_direction("column")
        .align_items("center")
        .justify_content("center")
        .gap("4px")
        .width("100%")
        // Height, padding, font and radius all come from the field's own
        // scale, so a dropzone and a text field at one `size` read as a family.
        .min_height(FILE_FIELD_DROPZONE_HEIGHT.value())
        .padding(FILE_FIELD_PADDING.value())
        .border("2px dashed")
        // 3:1 on the page, as a field frame (WCAG 1.4.11, todo 490).
        .border_color("muted.6")
        .border_radius(FILE_FIELD_RADIUS.value())
        .background("transparent")
        .cursor("pointer")
        .text_align("center")
        .transition("border-color 150ms, background 150ms")
        // The Browse button's ring is the surface's, out by the dashed border.
        .position("relative")
        .selector(
            "& > [data-ring]",
            ring_overlay_sx()
                .inset("-2px")
                .border_radius(FILE_FIELD_RADIUS.value()),
        )
        .selector("& :focus-visible ~ [data-ring]", focus_ring_sx())
        .selector("& [data-slot='hint']", sx().color("text-dimmed"))
        // Sized against the text, which is the one thing on the surface that
        // already scales.
        .selector(
            "& [data-slot='browse'] > svg",
            sx().width("2em").height("2em").color("muted.6"),
        )
        .when("dragging", drag_over_sx())
        .when(
            "disabled",
            sx().cursor("not-allowed").border_color("muted.3"),
        )
});

/// The picked files, under the surface rather than inside it: a dropzone that
/// grows with its own contents stops being a target to aim at.
pub(super) static FILE_CARD_LIST_SX: StaticSx = StaticSx::new(|| {
    // The list is a sibling of the surface, not a descendant, so it resolves
    // the same vars for its own cards.
    FileFieldDefaults::theme_vars()
        .var(SizeCss::RADIUS.override_var(), "inherit")
        .display("flex")
        .flex_direction("column")
        .gap("4px")
        .width("100%")
        .margin_top("8px")
        .list_style("none")
        .padding("0")
});

pub(super) static FILE_CARD_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .gap("8px")
        .width("100%")
        // The row must be allowed to shrink, or a long name pushes the x out
        // of the card instead of being clipped.
        .min_width("0")
        .padding(format!(
            "calc({} / 2) {}",
            FILE_FIELD_PADDING.value(),
            FILE_FIELD_PADDING.value()
        ))
        .border("1px solid")
        .border_color("muted.3")
        .border_radius(FILE_FIELD_RADIUS.value())
        .selector(
            "& [data-slot='name']",
            sx().flex("1 1 auto")
                .min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis")
                .white_space("nowrap"),
        )
        .selector(
            "& [data-slot='size']",
            sx().flex("0 0 auto")
                .color("text-dimmed")
                .font_size("0.85em"),
        )
        // `inline-flex`, or the button sits on the text's baseline.
        .selector(
            "& [data-slot='remove']",
            sx().flex("0 0 auto")
                .display("inline-flex")
                .align_items("center"),
        )
});

/// A flex line of its own, or the remove button hangs off the label's baseline.
pub(super) static FILE_CHIP_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex").max_width("100%").selector(
        "& [data-slot='remove']",
        sx().display("inline-flex").align_items("center"),
    )
});

/// The real input, and the only thing a form posts. Hidden rather than absent:
/// its `FileList` is what `set_files` keeps equal to `value`.
pub(super) static FILE_INPUT_SX: StaticSx = StaticSx::new(|| sx().display("none"));
