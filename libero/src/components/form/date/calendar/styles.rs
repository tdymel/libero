use crate::{
    components::{
        common::{Part, disabled_look_sx, focus_ring_sx, inset_focus_ring_sx},
        form::ChronoPickerPart,
    },
    sx::{FORCED_COLORS, StaticSx, sx},
    theme::{CHRONO_DAY, CHRONO_FONT_SIZE, ChronoPickerDefaults, Size, SizeCss},
};

pub(super) static CALENDAR_SX: StaticSx = StaticSx::new(|| {
    let day = CHRONO_DAY.value();
    let button = sx()
        .display("flex")
        .align_items("center")
        .justify_content("center")
        .padding("0")
        .margin("0")
        .border_style("solid")
        .border_width("1px")
        .border_color("transparent")
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        .background("transparent")
        .color("inherit")
        .font_family("inherit")
        .letter_spacing("inherit")
        .font_size("inherit")
        .cursor("pointer")
        .hover(sx().background("muted.1"));
    ChronoPickerDefaults::theme_vars()
        .display("inline-flex")
        .flex_direction("column")
        .gap("4px")
        // A mini strip shrinks to its parent's width rather than overflow it.
        .max_width("100%")
        .font_size(CHRONO_FONT_SIZE.value())
        .selector(
            ChronoPickerPart::Header.selector(),
            sx().display("flex").align_items("center").gap("4px"),
        )
        .selector(
            ChronoPickerPart::Title.selector(),
            button
                .clone()
                .flex("1 1 0")
                .height(day.clone())
                .font_weight("600"),
        )
        .selector(
            "& div[data-slot='title']",
            sx().cursor("default").hover(sx().background("transparent")),
        )
        .selector(
            "& > [data-slot='months']",
            sx().display("flex").gap("16px").align_items("flex-start"),
        )
        .selector(
            ChronoPickerPart::Weekday.selector(),
            sx().height(day.clone())
                .display("flex")
                .align_items("center")
                .justify_content("center")
                .font_size("0.85em")
                .font_weight("500"),
        )
        .selector(
            "& [role='grid']",
            sx().display("grid")
                .grid_template_columns(format!("repeat(7, {day})")),
        )
        // Rows exist for assistive technology; the grid lays out the cells.
        .selector("& [role='row']", sx().display("contents"))
        .selector("& [role='gridcell']", sx().display("flex"))
        .selector(
            ChronoPickerPart::Blank.selector(),
            sx().width(day.clone()).height(day.clone()),
        )
        .selector(
            ChronoPickerPart::Cells.selector(),
            sx().display("grid")
                .grid_template_columns("repeat(3, 1fr)")
                .gap("4px")
                .width(format!("calc(7 * {day})")),
        )
        .selector(
            ChronoPickerPart::Day.selector(),
            button.clone().width(day.clone()).height(day.clone()),
        )
        // The mini variant: one row of taller days, a month label over the number.
        // Seven days are ~430px: on a narrower page they shrink instead (1.4.10).
        .selector(
            ChronoPickerPart::Strip.selector(),
            sx().display("flex").align_items("center").gap("4px"),
        )
        .selector(
            "& [data-slot='strip'] > [data-slot='months']",
            sx().min_width("0"),
        )
        .selector(
            "& [data-slot='strip'] [role='grid']",
            sx().display("flex").gap("2px"),
        )
        .selector(
            "& [data-slot='strip'] [role='gridcell']",
            sx().flex("0 1 auto").min_width("0"),
        )
        .selector(
            "& [data-slot='strip'] [data-slot='day']",
            sx().flex_direction("column")
                .gap("2px")
                .width(format!("calc(1.4 * {day})"))
                .max_width("100%")
                .height(format!("calc(1.6 * {day})")),
        )
        .selector(
            "& [data-slot='strip'] [data-slot='month']",
            sx().font_size("0.75em").opacity("0.7"),
        )
        // Faded on the selected fill it read 3.07:1 (todo 1998).
        .selector(
            "& [data-slot='strip'] [data-selected] > [data-slot='month']",
            sx().opacity("1"),
        )
        .selector(
            ChronoPickerPart::Cell.selector(),
            button.width("100%").height(format!("calc(1.25 * {day})")),
        )
        .selector("& [data-outside]", sx().color("text-dimmed"))
        .selector("& [data-today]", sx().border_color("primary.6"))
        .selector(
            "& [data-in-range]",
            sx().background("primary.1")
                .color("primary-contrast.1")
                .border_radius("0")
                .hover(sx().background("primary.2")),
        )
        // After the plain hover and the range tint, so a picked day keeps its
        // fill under the mouse: equal specificity, source order decides.
        .selector(
            "& [data-selected]",
            sx().background("primary.6")
                .color("primary-contrast.6")
                .hover(sx().background("primary.7")),
        )
        .selector(
            "& :is([data-slot='day'], [data-slot='cell']):disabled",
            disabled_look_sx("not-allowed").hover(sx().background("transparent")),
        )
        .selector("& button:focus-visible", focus_ring_sx())
        // A picked cell's fill sets a ring colour meant for its inside: outside, a light ring vanishes.
        // More specific than the ring above.
        .selector(
            "& [data-selected]:focus-visible",
            inset_focus_ring_sx("-4px"),
        )
        // Forced colours paint the picked day's fill `Canvas`, like every other,
        // drop the range tint, and draw every transparent border.
        .media(
            FORCED_COLORS,
            sx().selector(
                "& :is([data-slot='day'], [data-slot='cell'])",
                sx().border_color("Canvas"),
            )
            // The disabled look's `GrayText` border would box every closed day.
            .selector(
                "& :is([data-slot='day'], [data-slot='cell']):disabled:not([data-today], [data-in-range])",
                sx().border_color("Canvas"),
            )
            .selector("& [data-in-range]", sx().border_color("Highlight"))
            .selector("& [data-today]", sx().border_color("CanvasText"))
            .selector(
                "& [data-selected]",
                sx().background("Highlight")
                    .color("HighlightText")
                    .hover(sx().background("Highlight")),
            ),
        )
});

/// The header runs right to left under RTL, so the chevrons point the other way.
pub(super) static NAV_SX: StaticSx =
    StaticSx::new(|| sx().rtl(sx().selector("& svg", sx().transform("scaleX(-1)"))));
