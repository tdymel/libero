use crate::{
    components::{
        common::{
            Part, borderless_on_state_sx, disabled_look_sx, focus_ring_sx, inset_focus_ring_sx,
            on_tint_color,
        },
        form::ChronoPickerPart,
    },
    sx::{FORCED_COLORS, StaticSx, ThemeAwareValue, sx},
    theme::{
        CHRONO_DAY, CHRONO_FONT_SIZE, ChronoPickerDefaults, Color, ColorShade, ColorValue, Size,
        SizeCss,
    },
};

pub(in super::super) static TIME_PICKER_SX: StaticSx = StaticSx::new(|| {
    let day = CHRONO_DAY.value();
    let button = sx()
        .display("flex")
        .align_items("center")
        .justify_content("center")
        .padding("0 4px")
        .margin("0")
        .border_style("none")
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        .background("transparent")
        .color("inherit")
        .font_family("inherit")
        .font_size("inherit")
        .cursor("pointer")
        .hover(sx().background("muted.1"));
    ChronoPickerDefaults::theme_vars()
        .display("inline-flex")
        .flex_direction("column")
        .align_items("center")
        .gap("8px")
        .font_size(CHRONO_FONT_SIZE.value())
        // A digital clock: `HH:MM`, each number between its faded neighbours.
        .selector(
            "& > [data-slot='columns']",
            sx().display("flex")
                .align_items("center")
                .justify_content("center")
                // Room for the focus ring, clear of the separators and an edge.
                .gap("6px")
                .padding("4px")
                .with("font-variant-numeric", "tabular-nums")
                .white_space("nowrap"),
        )
        .selector(
            ChronoPickerPart::Spin.selector(),
            sx().display("flex")
                .flex_direction("column")
                .align_items("center")
                .min_width(format!("calc(1.25 * {day})"))
                .padding("2px 6px")
                .border_radius(SizeCss::RADIUS.value(Size::Sm))
                .line_height("1.25")
                .cursor("ns-resize")
                // A touch drags the column rather than scrolling the page.
                .touch_action("none")
                .user_select("none")
                .hover(sx().background("muted.1")),
        )
        .selector(
            ChronoPickerPart::Value.selector(),
            sx().font_size("1.75em").font_weight("500"),
        )
        .selector(
            ChronoPickerPart::Neighbour.selector(),
            // The press is the single-pointer alternative to the drag: 24px target (WCAG 2.5.8).
            sx().display("flex")
                .align_items("center")
                .min_height("max(1.25em, 24px)")
                .color("text-dimmed")
                .cursor("pointer"),
        )
        .selector("& [data-slot='spin']:focus-visible", focus_ring_sx())
        .selector(
            "& [data-slot='spin']:focus-visible > [data-slot='value']",
            sx().color("primary.6"),
        )
        .selector(
            ChronoPickerPart::Separator.selector(),
            sx().font_size("1.75em").font_weight("500").color("text-dimmed"),
        )
        // A duration's unit after each column: `h`, `min`.
        .selector(
            ChronoPickerPart::Unit.selector(),
            sx().color("text-dimmed"),
        )
        .selector(
            "& > [data-slot='readout']",
            sx().display("flex")
                .align_items("center")
                .gap("4px")
                .font_size("1.5em"),
        )
        .selector(
            "& [data-slot='readout'] button",
            button.clone().height(format!("calc(1.25 * {day})")),
        )
        // The house on-state ring marks the hand being set (todo 744).
        .selector(
            "& [data-slot='readout'] [data-active]",
            borderless_on_state_sx().focus_visible(focus_ring_sx()),
        )
        .selector(
            ChronoPickerPart::Face.selector(),
            sx().position("relative")
                .width(format!("calc(7 * {day})"))
                .height(format!("calc(7 * {day})"))
                .border_radius("50%")
                .background("muted.1")
                .cursor("pointer")
                // A touch drags the hand rather than scrolling the page.
                .touch_action("none")
                .user_select("none"),
        )
        .selector(
            ChronoPickerPart::Mark.selector(),
            button
                .clone()
                .position("absolute")
                .width(day.clone())
                .height(day)
                .border_radius("50%")
                .padding("0")
                // The face's fill: the hand passes under an inner mark, not behind its digits.
                .background("muted.1")
                // A step above the face: the button's own `muted.1` hover vanished on it.
                .hover(sx().background("muted.2")),
        )
        // A quiet grey made for the face's tint, not half opacity: that read 3.87:1 (todo 1999).
        .selector(
            "& [data-slot='mark'][data-disabled]",
            sx().color(
                on_tint_color(&ThemeAwareValue::ColorValue(ColorValue::Shade(
                    Color::Muted,
                    ColorShade::S6,
                )))
                .unwrap_or_default(),
            )
                .cursor("not-allowed")
                .media(FORCED_COLORS, sx().color("GrayText"))
                .hover(sx().background("muted.1")),
        )
        // One tick per step where the marks are coarser than the step.
        .selector(
            ChronoPickerPart::Ticks.selector(),
            sx().position("absolute")
                .inset("4%")
                .border_radius("50%")
                .background(
                    "repeating-conic-gradient(from -0.5deg, color-mix(in srgb, currentColor 35%, transparent) 0 1deg, transparent 1deg var(--libero-clock-tick))",
                ),
        )
        // The face's own fill over the middle leaves the ticks a thin ring.
        .selector(
            "& [data-slot='ticks'] > div",
            sx().position("absolute")
                .inset("4px")
                .border_radius("50%")
                .background("muted.1"),
        )
        .selector(
            "& [data-slot='hand'], & [data-slot='pivot']",
            sx().position("absolute").background("primary.6"),
        )
        .selector("& [data-selected]", {
            sx().background("primary.6")
                .color("primary-contrast.6")
                .hover(sx().background("primary.7"))
        })
        .selector(
            "& button:disabled",
            disabled_look_sx("not-allowed").hover(sx().background("transparent")),
        )
        .selector("& button:focus-visible", focus_ring_sx())
        .selector("& [data-slot='face']:focus-visible", focus_ring_sx())
        // As in `Calendar`: a picked fill takes the ring inside. More specific than the ring above.
        .selector(
            "& [data-selected]:focus-visible",
            inset_focus_ring_sx("-4px"),
        )
        // Forced colours paint every fill `Canvas`: the picks and the hand would vanish.
        .media(
            FORCED_COLORS,
            sx().selector(
                "& [data-selected]",
                sx().background("Highlight")
                    .color("HighlightText")
                    .hover(sx().background("Highlight")),
            )
            .selector(
                "& [data-slot='hand'], & [data-slot='pivot']",
                sx().background("CanvasText"),
            ),
        )
});
