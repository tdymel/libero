use crate::{
    components::common::input_from_str,
    sx::{StaticSx, sx},
    theme::{
        ChoiceVariant, FIELD_CAPTIONS, FIELD_CARD_PADDING, FIELD_FRAME_GAP, FieldDefaults,
        PAPER_BACKGROUND, PAPER_BORDER_COLOR, PAPER_RADIUS, SizeCss,
    },
};

input_from_str!(ChoiceVariant);

/// The wrapper styles all four text slots by `data-slot`, saving four
/// stylesheet registrations per field.
pub(in crate::components::form) static FIELD_SX: StaticSx = StaticSx::new(|| {
    FieldDefaults::theme_vars()
        // A field inside a `Paper { radius: "0" }` must not inherit the paper's radius.
        .var(SizeCss::RADIUS.override_var(), "initial")
        .display("flex")
        .flex_direction("column")
        // Shrinks below its input's intrinsic width in a flex row (WCAG 1.4.10).
        .min_width("0")
        // A long unbroken word in a label or message wraps instead of scrolling the page (WCAG 1.4.10).
        .with("overflow-wrap", "anywhere")
        .selector(FIELD_CAPTIONS, sx().color("muted.7"))
        // The severity icon sits in the text line, one em tall.
        .selector(
            "& > [data-slot='status'] > svg",
            sx().width("1em")
                .height("1em")
                .vertical_align("-0.15em")
                .with("margin-inline-end", "0.25em"),
        )
        .selector(
            "& label > [data-slot='required']",
            sx().color("error.7")
                .margin_left("2px")
                .rtl(sx().margin_left("0").margin_right("2px")),
        )
        // A full-width control fills only the wrapper, so the wrapper fills its parent.
        .when("full-width", sx().width("100%"))
        // A frameless control sits beside its label; captions line up under the label.
        .when(
            "inline",
            sx().display("grid")
                .grid_template_columns("auto 1fr")
                .align_items("center")
                .column_gap(FIELD_FRAME_GAP.value())
                .selector(
                    format!("& > label, {FIELD_CAPTIONS}"),
                    sx().grid_column("2"),
                ),
        )
        // The inline layout as a `Paper` surface that is all hit area. No
        // `--lsx-focus-contrast`: the ring sits outside the card.
        .when(
            "card",
            sx().background(PAPER_BACKGROUND.value())
                .border(format!("1px solid {}", PAPER_BORDER_COLOR.value()))
                .border_radius(PAPER_RADIUS.value())
                // Under `:hover`: WebView spatial navigation makes a `pointer` element a Tab stop (3156).
                .hover(sx().cursor("pointer"))
                // A stretched card keeps its content at the top.
                .align_content("start")
                .per_size(|size| sx().padding(FIELD_CARD_PADDING.value(size)))
                // The control turns static under `card`, so its ring overlay
                // covers the card instead.
                .position("relative")
                .selector(
                    "& [data-state~=\"card\"] > [data-ring]",
                    sx().inset("-1px").border_radius(PAPER_RADIUS.value()),
                )
                .when("disabled", sx().cursor("not-allowed")),
        )
        // Not `opacity`: the control dims itself, and two stacked opacities
        // multiply.
        .when(
            "disabled",
            sx().color("muted.6")
                .selector(FIELD_CAPTIONS, sx().color("muted.6")),
        )
        .when(
            "warning",
            sx().selector("& > [data-slot='status']", sx().color("warning.7")),
        )
        .when(
            "error",
            sx().selector("& > [data-slot='status']", sx().color("error.7")),
        )
});
