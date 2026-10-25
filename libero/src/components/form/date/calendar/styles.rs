use crate::{
    components::{
        common::{COARSE_POINTER, Part, disabled_look_sx, focus_ring_sx, inset_focus_ring_sx},
        form::ChronoPickerPart,
    },
    sx::{FORCED_COLORS, StaticSx, sx},
    theme::{CHRONO_DAY, CHRONO_FONT_SIZE, ChronoPickerDefaults, CssVar, Size, SizeCss},
};

/// Between months side by side, in px.
pub(super) const MONTHS_GAP: f64 = 16.0;

/// The size's day, at least 44px to a finger (todo 3901), unless seven would overflow the screen.
const CALENDAR_DAY: CssVar = CssVar::new("--lsx-calendar-day");

pub(super) static CALENDAR_SX: StaticSx = StaticSx::new(|| {
    let day = CALENDAR_DAY.value();
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
        .var(CALENDAR_DAY, CHRONO_DAY.value())
        .media(
            COARSE_POINTER,
            sx().var(
                CALENDAR_DAY,
                format!(
                    "min(max(44px, {}), calc((100vw - 24px) / 7))",
                    CHRONO_DAY.value()
                ),
            ),
        )
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
            "& > [data-slot='months'], & > [data-slot='panel'] > [data-slot='months']",
            sx().display("flex")
                .gap(format!("{MONTHS_GAP}px"))
                .align_items("flex-start"),
        )
        // Presets above the views on a phone, beside them from `sm` up.
        .selector(
            "& > [data-slot='panel']",
            sx().display("flex")
                .flex_direction("column")
                .gap("4px")
                .min_width("0")
                .min_height("0"),
        )
        .selector(
            "&[data-state~='presets']",
            sx().gap("8px").media(
                format!("(min-width: {})", Size::Sm.breakpoint_value()),
                sx().flex_direction("row").align_items("flex-start").gap("12px"),
            ),
        )
        .selector(
            "&[data-state~='scroll'] > [data-slot='panel']",
            sx().flex("1 1 auto"),
        )
        .selector(
            ChronoPickerPart::Presets.selector(),
            sx().display("flex")
                .flex_wrap("wrap")
                .gap("4px")
                .media(
                    format!("(min-width: {})", Size::Sm.breakpoint_value()),
                    sx().flex_direction("column").flex_wrap("nowrap"),
                ),
        )
        .selector(
            ChronoPickerPart::Preset.selector(),
            button
                .clone()
                .justify_content("flex-start")
                .min_height(day.clone())
                .padding("0 10px")
                .white_space("nowrap"),
        )
        .selector(
            "& [data-slot='preset'][aria-pressed='true']",
            sx().background("primary.1")
                .color("primary-contrast.1")
                .hover(sx().background("primary.2")),
        )
        // Drawn only to measure how many months fit (todo 3958); opacity keeps focus.
        .selector("&[data-state~='measuring']", sx().opacity("0"))
        // Below `sm` a range scrolls its months under the readout (todo 3903).
        // A sheet's flex column shrinks it; inline, the months stop at 60% of the screen.
        .selector(
            "&[data-state~='scroll']",
            sx().display("flex").width("100%").min_height("0"),
        )
        .selector(
            "& > [data-slot='scroller'], & > [data-slot='panel'] > [data-slot='scroller']",
            sx().flex("1 1 auto")
                .min_height("0")
                .max_height("60vh")
                .overflow_y("auto")
                .supports("(height: 1dvh)", sx().max_height("60dvh")),
        )
        .selector(
            "& > [data-slot='readout'], & > [data-slot='panel'] > [data-slot='readout']",
            sx().display("flex")
                .align_items("center")
                .justify_content("center")
                .gap("8px")
                .min_height(day.clone())
                .font_weight("600"),
        )
        .selector(
            "& [data-slot='readout'] > [data-unset]",
            sx().color("text-dimmed").font_weight("400"),
        )
        .selector(
            "& [data-slot='months'][data-scroll]",
            sx().display("flex")
                .flex_direction("column")
                .align_items("center")
                .gap("16px"),
        )
        .selector(
            "& [data-slot='month-title']",
            sx().display("flex")
                .align_items("center")
                .height(day.clone())
                .font_weight("600"),
        )
        .selector(
            "& [data-slot='more']",
            sx().display("flex").justify_content("center"),
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
        .selector(
            "& [role='grid'][data-week-numbers]",
            sx().grid_template_columns(format!("calc(0.75 * {day}) repeat(7, {day})")),
        )
        .selector(
            ChronoPickerPart::WeekNumber.selector(),
            sx().display("flex")
                .align_items("center")
                .justify_content("center")
                .height(day.clone())
                .font_size("0.75em")
                .color("text-dimmed"),
        )
        // `hide_weekdays`: out of sight, still the grid's column headers.
        .selector(
            "& [role='row'][data-hidden] > *",
            sx().position("absolute")
                .width("1px")
                .height("1px")
                .overflow("hidden")
                .clip_path("inset(50%)")
                .white_space("nowrap"),
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
            sx().font_size("0.75em").color("text-dimmed"),
        )
        // Dimmed it reads 3.07:1 on the selected fill (todo 1998), 4.08:1 on the hover fill.
        .selector(
            "& [data-slot='strip'] :is([data-selected], [data-slot='day']:hover:not(:disabled)) > [data-slot='month']",
            sx().color("inherit"),
        )
        .selector(
            ChronoPickerPart::Cell.selector(),
            button.width("100%").height(format!("calc(1.25 * {day})")),
        )
        .selector("& [data-outside]", sx().color("text-dimmed"))
        .selector(
            "& [data-weekend]:not([data-outside], [data-selected], [data-in-range], :disabled, [aria-disabled='true'])",
            sx().color("error.7"),
        )
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
        // A disabled picker (todo 3881): soft, so a focused day keeps its focus.
        .selector(
            "& :is([data-slot='day'], [data-slot='cell'], [data-slot='title'])[aria-disabled='true']",
            disabled_look_sx("not-allowed"),
        )
        .selector(
            "& [aria-disabled='true']:not([data-selected], [data-in-range]):hover",
            sx().background("transparent"),
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
                // Else Chromium paints a Canvas backplate behind the digit.
                sx().forced_color_adjust("none")
                    .background("Highlight")
                    .color("HighlightText")
                    .hover(sx().background("Highlight")),
            ),
        )
});

/// The header runs right to left under RTL, so the chevrons point the other way.
pub(super) static NAV_SX: StaticSx =
    StaticSx::new(|| sx().rtl(sx().selector("& svg", sx().transform("scaleX(-1)"))));
