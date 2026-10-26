use std::{future::Future, pin::Pin};

use dioxus::prelude::*;

use super::slider_value::{SliderChangeEvent, SliderMark, SliderSegment};
use super::value::{SliderCoreValue, fraction, grid_bounds, on_track};
use crate::{
    CssLayer,
    components::{
        common::{
            COARSE_POINTER, ClassList, HtmlTag, Input, Part, States, Variables, base_color,
            disabled_look_within_sx, has_shortcut_modifier, shadow_sx, text_color, variables,
        },
        form::field_parts_enum,
        layout::use_box,
        overlay::{PressFocus, Tooltip, TooltipPinned},
    },
    hooks::{
        DragMove, DragOptions, DragPoint, DragStart, ElementHandle, Rect, sideways_drag_sx,
        use_css, use_element, use_escape_dismiss, use_formats, use_id, use_localization,
        use_sideways_drag_if, use_theme,
    },
    localization::{Formats, fill},
    platform::{Dimensions, ElementApi, logical_key, next_task},
    sx::{FORCED_COLORS, StaticSx, Sx, ThemeAwareValue, sx},
    theme::NamedColorCss,
    theme::{
        Color, ColorShade, ColorValue, CssVar, OWN_SHADOW, SLIDER_THUMB, SLIDER_TRACK, Side, Size,
        SliderDefaults,
    },
    utils::warn,
};

/// A unitless 0-1 fraction, not a percentage: everything positioned along the
/// track multiplies it by the travel, which is a `calc` of two lengths.
const SLIDER_FILLED: CssVar = CssVar::new("--lsx-slider-filled");
/// Where the bar starts, and how wide it is: `0` and the fill for one thumb,
/// the two thumbs' own fractions for a range.
const SLIDER_FILLED_FROM: CssVar = CssVar::new("--lsx-slider-filled-from");
const SLIDER_FILLED_SPAN: CssVar = CssVar::new("--lsx-slider-filled-span");
/// One thumb's own position, set on its anchor - the thumbs of a range sit at
/// two different fractions, so the fill's cannot serve them both.
const SLIDER_THUMB_AT: CssVar = CssVar::new("--lsx-slider-thumb-at");
const SLIDER_COLOR: CssVar = CssVar::new("--lsx-slider-color");
const SLIDER_MARK_AT: CssVar = CssVar::new("--lsx-slider-mark-at");
/// A segment's start and length as 0-1 fractions, and how much of it is filled.
const SLIDER_SEGMENT_AT: CssVar = CssVar::new("--lsx-slider-segment-at");
const SLIDER_SEGMENT_SPAN: CssVar = CssVar::new("--lsx-slider-segment-span");
const SLIDER_SEGMENT_FILLED: CssVar = CssVar::new("--lsx-slider-segment-filled");
/// The first and last segment reach out to the track's ends, past the thumb's inset.
const SLIDER_SEGMENT_REACH_START: CssVar = CssVar::new("--lsx-slider-segment-reach-start");
const SLIDER_SEGMENT_REACH_END: CssVar = CssVar::new("--lsx-slider-segment-reach-end");
/// Between two segments, as YouTube's chapters. A literal: a theme field breaks `SliderDefaults` literals.
const SEGMENT_GAP: &str = "2px";
/// A plain slider's thumb face - the color the track is pointing at.
const SLIDER_THUMB_FILL: CssVar = CssVar::new("--lsx-slider-thumb-fill");

/// Where a 0-1 `fraction` sits along the track, inset by half a thumb so it
/// never overhangs.
fn along_track(fraction: CssVar) -> String {
    let thumb = SLIDER_THUMB.value();
    format!(
        "calc({thumb} / 2 + {} * (100% - {thumb}))",
        fraction.value_or("0")
    )
}

field_parts_enum! {
    /// [`Slider`](super::Slider)'s and [`RangeSlider`](super::RangeSlider)'s
    /// inner parts, for their `parts` prop: a field's, and the slider's own.
    pub enum SliderPart {
        /// The slider under the label: the track and the room around it.
        Control = "control" => "& > [data-slot='control']",
        /// The rail the thumbs run along.
        Track = "track" => "& > [data-slot='control'] > [data-slot='track']",
        /// The filled stretch of the track.
        Bar = "bar" => "& > [data-slot='control'] > [data-slot='track'] > [data-slot='bar']",
        /// The row of bars a `SliderTrack::Bars` track draws.
        Bars = "bars" => "& > [data-slot='control'] > [data-slot='track'] > [data-slot='bars']",
        /// The row of segments a `segments` track draws.
        Segments = "segments" => "& > [data-slot='control'] > [data-slot='track'] > [data-slot='segments']",
        /// One stretch of a segmented track.
        Segment = "segment" => "& > [data-slot='control'] > [data-slot='track'] > [data-slot='segments'] > [data-slot='segment']",
        /// The filled part of a segment.
        SegmentFill = "segment-fill" => "& > [data-slot='control'] > [data-slot='track'] > [data-slot='segments'] > [data-slot='segment'] > [data-slot='segment-fill']",
        /// One tick on the track.
        Mark = "mark" => "& > [data-slot='control'] > [data-slot='track'] > [data-slot='mark']",
        /// A tick's caption.
        MarkLabel = "mark-label" => "& > [data-slot='control'] > [data-slot='track'] > [data-slot='mark-label']",
        /// The handle; a range has two.
        Thumb = "thumb" => "& > [data-slot='control'] > [data-slot='track'] [data-slot='thumb']",
    }
}

const PLAIN_THUMB_SHADOW: &str = "0 0 0 1px rgba(0, 0, 0, 0.2), inset 0 0 0 1px rgba(0, 0, 0, 0.2)";

static SLIDER_ROOT_SX: StaticSx = StaticSx::new(|| {
    SliderDefaults::theme_vars()
        // A vertical swipe scrolls the page, as over a native Android slider.
        .and(sideways_drag_sx())
        .display("flex")
        .align_items("center")
        .position("relative")
        // Or a flex/grid parent collapses the empty track to nothing.
        .width("100%")
        .min_height(SLIDER_THUMB.value())
        .user_select("none")
        // Room for the captions. Padding, not margin, which would collapse.
        .when("marks-labeled", sx().padding_bottom("1.5em"))
        // No grab cursor for a drag `onstart` refuses.
        .when("readonly", sx().selector("& *", sx().cursor("default")))
        // No `pointer-events: none`, so `not-allowed` shows (todo 596); `onstart` refuses the
        // drag. After `readonly`; focus inside lifts the fade the thumb's ring would take.
        .when(
            "disabled",
            disabled_look_within_sx("not-allowed").selector("& *", sx().cursor("not-allowed")),
        )
        // As the `Audio` seek track: the bars draw it, and a press anywhere on them moves.
        .when(
            "bars",
            sx().selector(
                format!("& > [data-slot='{}']", SliderPart::Track.slot()),
                sx().background("transparent").height("1.5rem"),
            ),
        )
        // Gaps between the segments show through, so the rail itself goes.
        .when(
            "segments",
            sx().selector(
                format!("& > [data-slot='{}']", SliderPart::Track.slot()),
                sx().background("transparent"),
            ),
        )
        // A column, as tall as the sx says; captions sit beside the track, not under it.
        .when(
            "vertical",
            sx().flex_direction("column")
                .align_items("flex-start")
                .height("12rem")
                .padding_bottom("0")
                // A vertical swipe is the drag, not a scroll.
                .touch_action("none")
                .selector(
                    format!("& > [data-slot='{}']", SliderPart::Track.slot()),
                    sx().width(SLIDER_TRACK.value())
                        .height("auto")
                        // Centred under the thumb, which is wider than the track.
                        .margin_inline(format!(
                            "calc(({} - {}) / 2)",
                            SLIDER_THUMB.value(),
                            SLIDER_TRACK.value()
                        )),
                ),
        )
        // White ring and dark halo read on any track color. The shadow only
        // unfocused, or it outranks the thumb's `:focus-visible` ring.
        .when(
            "plain",
            sx().selector(
                "& [role='slider']",
                sx().border_color("surface")
                    .var(OWN_SHADOW, PLAIN_THUMB_SHADOW),
            )
            .selector(
                "& [role='slider']:not(:focus-visible)",
                shadow_sx(PLAIN_THUMB_SHADOW.to_string()),
            ),
        )
});

static SLIDER_TRACK_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative")
        .flex("1 1 auto")
        .height(SLIDER_TRACK.value())
        // 3:1 in both schemes (WCAG 1.4.11, todo 512). Via its var: a named
        // shade would publish a `--lsx-focus-contrast` the thumb's ring takes.
        .background(ColorValue::Shade(Color::Muted, ColorShade::S6).value())
        // Always a pill: at 2-10px tall every radius step clamps the same.
        .border_radius("999px")
        // Under `:hover`: WebView spatial navigation makes a `pointer` element a Tab stop (3156).
        .hover(sx().cursor("pointer"))
        // Forced colours drop the background but paint a transparent outline
        // (todo 506).
        .outline("1px solid transparent")
});

static SLIDER_BAR_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .top("0")
        .bottom("0")
        .left("0")
        .width(along_track(SLIDER_FILLED))
        .background(SLIDER_COLOR.value())
        .border_radius("inherit")
        .media(FORCED_COLORS, sx().background("Highlight"))
        // Under RTL the track runs right to left, as a native range does.
        .rtl(sx().left("auto").right("0"))
});

/// Inset by half a thumb, so the first and last bar sit under the thumb's ends of travel.
static SLIDER_BARS_SX: StaticSx = StaticSx::new(|| {
    let inset = format!("calc({} / 2)", SLIDER_THUMB.value());
    sx().position("absolute")
        .top("0")
        .bottom("0")
        .left(inset.clone())
        .right(inset)
        .display("flex")
        .align_items("center")
        .justify_content("space-between")
        .pointer_events("none")
        .selector(
            "& > span",
            sx().flex("0 1 2px")
                .min_width("1px")
                .min_height("2px")
                .border_radius("999px")
                // The line track's 3:1 (WCAG 1.4.11).
                .background(ColorValue::Shade(Color::Muted, ColorShade::S6).value())
                .media(FORCED_COLORS, sx().background("CanvasText")),
        )
        .selector(
            "& > span[data-state='filled']",
            sx().background(SLIDER_COLOR.value())
                .media(FORCED_COLORS, sx().background("Highlight")),
        )
});

/// The `Bars` row's inset, so a segment's fractions line up with the thumb's centre.
static SLIDER_SEGMENTS_SX: StaticSx = StaticSx::new(|| {
    let inset = format!("calc({} / 2)", SLIDER_THUMB.value());
    sx().position("absolute")
        .top("0")
        .bottom("0")
        .left(inset.clone())
        .right(inset)
        .pointer_events("none")
});

/// Placed by exact fractions, not flex weights, which would drift the fill off the thumb.
static SLIDER_SEGMENT_SX: StaticSx = StaticSx::new(|| {
    let start = SLIDER_SEGMENT_REACH_START.value_or("0px");
    let end = SLIDER_SEGMENT_REACH_END.value_or("0px");
    let at = format!(
        "calc({} * 100% + {SEGMENT_GAP} / 2 - {start})",
        SLIDER_SEGMENT_AT.value_or("0")
    );
    let reach = format!("calc({SEGMENT_GAP} / 2 + {} / 2)", SLIDER_THUMB.value());
    sx().position("absolute")
        .top("0")
        .bottom("0")
        .left(at.clone())
        .width(format!(
            "calc({} * 100% - {SEGMENT_GAP} + {start} + {end})",
            SLIDER_SEGMENT_SPAN.value_or("0")
        ))
        .background(ColorValue::Shade(Color::Muted, ColorShade::S6).value())
        .border_radius("999px")
        .overflow("hidden")
        .outline("1px solid transparent")
        .selector(
            "&:first-child",
            sx().var(SLIDER_SEGMENT_REACH_START, reach.clone()),
        )
        .selector("&:last-child", sx().var(SLIDER_SEGMENT_REACH_END, reach))
        .rtl(sx().left("auto").right(at))
});

static SLIDER_SEGMENT_FILL_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .top("0")
        .bottom("0")
        .left("0")
        .width(format!(
            "calc({} * 100%)",
            SLIDER_SEGMENT_FILLED.value_or("0")
        ))
        .background(SLIDER_COLOR.value())
        .media(FORCED_COLORS, sx().background("Highlight"))
        .rtl(sx().left("auto").right("0"))
});

/// A segment on the track: its value range and label.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TrackSegment {
    pub(crate) start: f64,
    pub(crate) end: f64,
    pub(crate) label: Option<String>,
}

/// Sorted and bounded: a start outside `min..max` or a repeat is dropped (the
/// `bool`), and an unlabelled segment fills from `min` to the first start.
pub(crate) fn track_segments(
    segments: &[SliderSegment],
    min: f64,
    max: f64,
) -> (Vec<TrackSegment>, bool) {
    let mut starts: Vec<&SliderSegment> = segments
        .iter()
        .filter(|segment| segment.start >= min && segment.start < max)
        .collect();
    starts.sort_by(|a, b| a.start.total_cmp(&b.start));
    starts.dedup_by(|later, earlier| later.start == earlier.start);
    let dropped = starts.len() < segments.len();
    if starts.is_empty() {
        return (Vec::new(), dropped);
    }
    let lead = (starts[0].start > min).then(|| TrackSegment {
        start: min,
        end: starts[0].start,
        label: None,
    });
    let ends = starts.iter().skip(1).map(|next| next.start).chain([max]);
    let rest = starts.iter().zip(ends).map(|(segment, end)| TrackSegment {
        start: segment.start,
        end,
        label: segment.label.clone(),
    });
    (lead.into_iter().chain(rest).collect(), dropped)
}

/// Where two segments meet: the gap is the tick there, so a mark draws no dot.
fn on_boundary(segments: &[TrackSegment], value: f64) -> bool {
    segments
        .iter()
        .skip(1)
        .any(|segment| segment.start == value)
}

/// The label of the segment holding `value`.
fn segment_label(segments: &[TrackSegment], value: f64) -> Option<&str> {
    segments
        .iter()
        .rev()
        .find(|segment| segment.start <= value)
        .and_then(|segment| segment.label.as_deref())
}

/// A value's `aria-valuetext`, `None` for the bare value already in `aria-valuenow`, and
/// its bubble's text. A labelled segment names itself after the value, so it is heard too.
fn value_text(
    label: Option<Callback<f64, String>>,
    segments: &[TrackSegment],
    segment_text: &str,
    value: f64,
    formats: &Formats,
    group_digits: bool,
) -> (Option<String>, String) {
    let text = label.map(|label| label.call(value));
    let bare = || match group_digits {
        true => formats.write_number(&value.to_string()),
        false => value
            .to_string()
            .replacen('.', formats.decimal_separator, 1),
    };
    let text = match segment_label(segments, value) {
        Some(segment) => Some(fill(
            segment_text,
            &[("value", &text.unwrap_or_else(bare)), ("segment", &segment)],
        )),
        None => text,
    };
    let bubble = text.clone().unwrap_or_else(bare);
    (text, bubble)
}

/// The bubble over the value a hovering mouse or pen points at, while no drag runs.
#[component]
fn SliderHoverPreview(
    mut hovered: Signal<Option<(f64, ThumbTrack)>>,
    mut muted: CopyValue<bool>,
    dragging: Signal<bool>,
    min: f64,
    max: f64,
    size: Size,
    label: Option<Callback<f64, String>>,
    segments: Vec<TrackSegment>,
    segment_text: &'static str,
    group_digits: bool,
) -> Element {
    let formats = use_formats();
    let shown = hovered().filter(|_| !dragging());
    let hide = use_callback(move |()| {
        muted.set(true);
        hovered.set(None);
    });
    let _ = use_escape_dismiss(shown.is_some(), true, hide);
    let Some((value, track)) = shown else {
        return rsx! {};
    };
    let (_, text) = value_text(label, &segments, segment_text, value, formats, group_digits);
    rsx! {
        TooltipPinned {
            label: rsx! { {text} },
            anchor: track.at(fraction(value, min, max)),
            size,
            gap: Size::Sm,
            side: track.vertical.then_some(Side::End),
            id: None,
            rtl: track.rtl,
            shown: true,
        }
    }
}

/// How much of a segment is filled when the bar reaches `filled` (0-1 of the track).
pub(crate) fn segment_filled(start: f64, end: f64, filled: f64) -> f64 {
    if end <= start {
        return 0.0;
    }
    ((filled - start) / (end - start)).clamp(0.0, 1.0)
}

/// How many of `count` bars are filled at `fraction`: the `Audio` track's rule.
fn filled_bars(fraction: f64, count: usize) -> usize {
    (fraction.clamp(0.0, 1.0) * count as f64).round() as usize
}

/// The fill and thumb ring in the text role, darker on a light page: success, warning and
/// info reach 3:1 there, as ProgressBar's fill (todos 1577, 3736).
fn slider_variables(color: &ThemeAwareValue) -> Variables {
    variables().with(SLIDER_COLOR, text_color(color))
}

/// A range fills *between* its thumbs, so the bar starts at the lower one's
/// centre rather than at the track's edge, and spans the difference.
static SLIDER_RANGE_BAR_SX: StaticSx = StaticSx::new(|| {
    let thumb = SLIDER_THUMB.value();
    sx().position("absolute")
        .top("0")
        .bottom("0")
        .left(along_track(SLIDER_FILLED_FROM))
        .width(format!(
            "calc({} * (100% - {thumb}))",
            SLIDER_FILLED_SPAN.value_or("0")
        ))
        .background(SLIDER_COLOR.value())
        .border_radius("inherit")
        .media(FORCED_COLORS, sx().background("Highlight"))
        .rtl(sx().left("auto").right(along_track(SLIDER_FILLED_FROM)))
});

/// Carries the thumb's position, because the `Tooltip` between them styles
/// only its own bubble - its wrapper cannot be positioned from outside.
static SLIDER_THUMB_ANCHOR_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .top("50%")
        .left(along_track(SLIDER_THUMB_AT))
        // Centred by margins, not a `transform`: Blitz's client rect ignores
        // that, so the thumb measured half a thumb off where it is drawn.
        .margin(format!("calc({} / -2)", SLIDER_THUMB.value()))
        // Not inline: the tooltip's inline-block wrapper would sit on a
        // baseline and pull the thumb off the track's centre.
        .display("flex")
        .rtl(sx().left("auto").right(along_track(SLIDER_THUMB_AT)))
});

static SLIDER_THUMB_SX: StaticSx = StaticSx::new(|| {
    // A `span`, because the tooltip's wrapper is one - so it needs a box.
    sx().display("block")
        .width(SLIDER_THUMB.value())
        .height(SLIDER_THUMB.value())
        .border_radius("50%")
        // `value_or`'s fallback is raw CSS, so a var, not `"surface"`, which
        // computes to transparent (todo 385).
        .background(SLIDER_THUMB_FILL.value_or(NamedColorCss::SURFACE.value()))
        .border_style("solid")
        .border_width("2px")
        .border_color(SLIDER_COLOR.value())
        .cursor("grab")
        // An invisible 24px square for WCAG 2.5.8. A press picks its thumb by
        // value, not hit-test, so overlapping squares cannot swap them.
        .position("relative")
        .selector(
            "&::before",
            sx().content("\"\"")
                .position("absolute")
                .top("50%")
                .left("50%")
                .width("24px")
                .height("24px")
                .transform("translate(-50%, -50%)"),
        )
        // 44px under a finger (todo 3589).
        .media(
            COARSE_POINTER,
            sx().selector("&::before", sx().width("44px").height("44px")),
        )
});

/// A mark's dot, placed by the orientation's own sx.
fn mark_dot_sx() -> Sx {
    // Sized against the thumb, not the track: a 2px track leaves a dot too
    // small to see, and it has to read against the thumb beside it.
    let dot = format!("calc({} / 3)", SLIDER_THUMB.value());
    sx().position("absolute")
        .width(dot.clone())
        .height(dot)
        .border_radius("50%")
        // An open ring in the track's own colour: the hole reads at the
        // track's 3:1 on either side of it (WCAG 1.4.11, todo 512).
        .background("surface")
        .border(format!(
            "1px solid {}",
            ColorValue::Shade(Color::Muted, ColorShade::S6).value()
        ))
        // Forced colours would paint it the track's `Canvas`; a filled one
        // stays `Canvas`, on the `Highlight` bar.
        .media(FORCED_COLORS, sx().background("CanvasText"))
        // White on the filled bar, the way the thumb is.
        .when(
            "filled",
            sx().background("surface").border_color("transparent"),
        )
}

static SLIDER_MARK_SX: StaticSx = StaticSx::new(|| {
    mark_dot_sx()
        .top("50%")
        .left(along_track(SLIDER_MARK_AT))
        .transform("translate(-50%, -50%)")
        .rtl(
            sx().left("auto")
                .right(along_track(SLIDER_MARK_AT))
                .transform("translate(50%, -50%)"),
        )
});

static SLIDER_MARK_LABEL_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        // In thumbs, not `%`, which counts the `marks-labeled` padding. Keep
        // that reserve equal to this offset plus a line, or captions overflow.
        .top(SLIDER_THUMB.value())
        .left(along_track(SLIDER_MARK_AT))
        // Centred in the middle, edge-aligned at the ends: a `-50%` caption
        // on the first or last mark hangs outside the slider's own box.
        .transform(format!(
            "translateX(calc({} * -100%))",
            SLIDER_MARK_AT.value_or("0")
        ))
        .color("muted.7")
        .white_space("nowrap")
        .rtl(
            sx().left("auto")
                .right(along_track(SLIDER_MARK_AT))
                .transform(format!(
                    "translateX(calc({} * 100%))",
                    SLIDER_MARK_AT.value_or("0")
                )),
        )
});

/// The vertical track's bar, filled from the bottom; no RTL flip.
static SLIDER_BAR_VERTICAL_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .left("0")
        .right("0")
        .bottom("0")
        .height(along_track(SLIDER_FILLED))
        .background(SLIDER_COLOR.value())
        .border_radius("inherit")
        .media(FORCED_COLORS, sx().background("Highlight"))
});

static SLIDER_RANGE_BAR_VERTICAL_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .left("0")
        .right("0")
        .bottom(along_track(SLIDER_FILLED_FROM))
        .height(format!(
            "calc({} * (100% - {}))",
            SLIDER_FILLED_SPAN.value_or("0"),
            SLIDER_THUMB.value()
        ))
        .background(SLIDER_COLOR.value())
        .border_radius("inherit")
        .media(FORCED_COLORS, sx().background("Highlight"))
});

static SLIDER_THUMB_ANCHOR_VERTICAL_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .left("50%")
        .bottom(along_track(SLIDER_THUMB_AT))
        .margin(format!("calc({} / -2)", SLIDER_THUMB.value()))
        .display("flex")
});

static SLIDER_MARK_VERTICAL_SX: StaticSx = StaticSx::new(|| {
    mark_dot_sx()
        .bottom(along_track(SLIDER_MARK_AT))
        .left("50%")
        .transform("translate(-50%, 50%)")
});

/// Beside the track at the inline end, edge-aligned at the ends as the horizontal captions.
static SLIDER_MARK_LABEL_VERTICAL_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .left(SLIDER_THUMB.value())
        .bottom(along_track(SLIDER_MARK_AT))
        .transform(format!(
            "translateY(calc({} * 100%))",
            SLIDER_MARK_AT.value_or("0")
        ))
        .color("muted.7")
        .white_space("nowrap")
        .rtl(sx().left("auto").right(SLIDER_THUMB.value()))
});

/// On the bar, not the root: the root's scope skips a value move.
fn bar_variables(bar: (f64, f64)) -> String {
    let (from, to) = bar;
    variables()
        .with(SLIDER_FILLED, Some(to.to_string()))
        .with(SLIDER_FILLED_FROM, Some(from.to_string()))
        .with(SLIDER_FILLED_SPAN, Some((to - from).to_string()))
        .render()
}

/// The `f64` engine every `Slider<V>`, `HueSlider` and `AlphaSlider` renders.
/// Non-generic, so it compiles once.
#[derive(Props, Clone, PartialEq)]
pub(in crate::components::form) struct SliderCoreProps {
    /// Already resolved by the skin - the scale, not the caller's type. One
    /// thumb or two; a range's own clamping lives in `SliderCoreValue`.
    value: SliderCoreValue,
    min: f64,
    max: f64,
    step: f64,
    /// The smallest gap a range's two thumbs keep. Ignored by a single thumb.
    #[props(default)]
    min_range: f64,
    attributes: Vec<Attribute>,
    /// The field owns the wrapper's styling, so the core's own is empty
    /// unless something inside the library styles the track directly.
    #[props(default)]
    class: Input<ClassList>,
    #[props(default)]
    sx: Input<Sx>,
    #[props(default)]
    states: Input<States>,
    size: Input<Size>,
    color: Input<ThemeAwareValue>,
    disabled: Option<bool>,
    /// Focusable and posted, but no key or drag moves a thumb.
    #[props(default)]
    readonly: bool,
    /// `None` leaves the bubble showing the bare value and sets no
    /// `aria-valuetext`.
    label: Option<Callback<f64, String>>,
    marks: Vec<SliderMark>,
    /// `aria-hidden` on the mark captions, for a discrete scale whose
    /// `aria-valuetext` already names each value (todo 513).
    #[props(default)]
    captions_hidden: bool,
    aria_label: Option<String>,
    /// Names the second thumb of a range; `aria_label` names the first.
    aria_label_to: Option<String>,
    /// The field's label id, when a `<label for>` cannot name the thumb.
    labelledby: Option<String>,
    /// Names a range's group when no `labelledby` does.
    #[props(default)]
    group_label: Option<String>,
    /// The field's filled caption slots, joined.
    describedby: Option<String>,
    /// The field's status is an error.
    invalid: bool,
    name: Option<String>,
    oninput: Option<EventHandler<SliderChangeEvent<SliderCoreValue>>>,
    /// A CSS `background` for the track in place of its grey - a hue or an
    /// alpha gradient.
    #[props(default)]
    track: Option<String>,
    /// No filled bar and no value bubble: the track itself shows the value.
    #[props(default)]
    plain: bool,
    /// The thumb's face, for a `plain` slider. White otherwise.
    #[props(default)]
    thumb_fill: Option<String>,
    /// `false` keeps the thumbs out of the tab order and a drag from focusing
    /// them - for a slider inside a dropdown whose trigger must keep focus.
    #[props(default = true)]
    focusable: bool,
    /// The root's `data-slot`: `control` in a field, `hue`/`alpha` for a colour scale.
    #[props(default = SliderPart::Control.slot())]
    slot: &'static str,
    /// The bar heights of a `SliderTrack::Bars` track, drawn in place of the filled line.
    #[props(default)]
    bars: Option<Vec<f64>>,
    /// Splits the line track into segments with gaps; a labelled one joins the value's text.
    #[props(default)]
    segments: Vec<SliderSegment>,
    /// A bare value's text, with no `label`, is grouped in thousands in the formats'.
    #[props(default)]
    group_digits: bool,
    /// A bubble over the value a hovering mouse or pen points at.
    #[props(default)]
    preview_on_hover: bool,
    /// The track runs bottom to top; bars and segments draw a line instead.
    #[props(default)]
    vertical: bool,
}

/// What a value move changes. Only the thumbs' scope reads it, so the rest of
/// the slider skips a drag frame.
#[derive(Clone, PartialEq)]
struct Live {
    value: SliderCoreValue,
    thumb_fill: Option<String>,
}

#[component]
pub(in crate::components::form) fn SliderCore(props: SliderCoreProps) -> Element {
    // `f64::clamp` panics on a broken range; `step.max(0.0)` maps NaN to 0.
    let (min, max) = grid_bounds(props.min, props.max, props.step);
    let live_now = Live {
        value: props.value.snapped(min, max, props.step.max(0.0)),
        thumb_fill: props.thumb_fill.clone(),
    };
    let mut live = use_signal(|| live_now.clone());
    if *live.peek() != live_now {
        live.set(live_now);
    }
    // The body keeps the value's shape only, so its props compare equal.
    let shape = match props.value {
        SliderCoreValue::Single(_) => SliderCoreValue::Single(0.0),
        SliderCoreValue::Range { .. } => SliderCoreValue::Range { from: 0.0, to: 0.0 },
    };
    rsx! {
        SliderBody {
            live,
            core: SliderCoreProps {
                value: shape,
                thumb_fill: None,
                ..props
            },
        }
    }
}

/// A drag whose geometry is still being measured, and what arrived meanwhile.
#[derive(Clone, Copy, Default)]
struct Starting {
    measuring: bool,
    /// The last move's client coordinate along the track.
    moved_to: Option<f64>,
    released: bool,
}

/// Where a thumb sits at any value, measured at drag start: its bubble is
/// placed from the value, not from a measuring round trip per move (1065).
#[derive(Clone, Copy, PartialEq)]
struct ThumbTrack {
    /// The start edge of a thumb at the track's left or top end.
    left: f64,
    travel: f64,
    /// The thumb's edge across the track: its top, or its left on a vertical one.
    top: f64,
    thumb: Dimensions,
    rtl: bool,
    vertical: bool,
    /// From the press till the new measure: the bubble stays mounted, hidden (2188).
    stale: bool,
}

impl ThumbTrack {
    /// A vertical track's minimum is at the bottom, whatever the direction.
    fn flipped(&self) -> bool {
        self.rtl || self.vertical
    }

    /// The thumb's length along the track.
    fn thumb_along(&self) -> f64 {
        match self.vertical {
            true => self.thumb.height,
            false => self.thumb.width,
        }
    }

    fn at(&self, fraction: f64) -> Rect {
        let along = if self.flipped() {
            1.0 - fraction
        } else {
            fraction
        };
        let (x, y) = match self.vertical {
            false => (self.left + along * self.travel, self.top),
            true => (self.top, self.left + along * self.travel),
        };
        Rect {
            x,
            y,
            width: self.thumb.width,
            height: self.thumb.height,
        }
    }

    /// Whether a pointer at `client` along the track is over the thumb at `fraction`.
    fn covers(&self, fraction: f64, client: f64) -> bool {
        let at = self.at(fraction);
        let start = if self.vertical { at.y } else { at.x };
        start <= client && client <= start + self.thumb_along()
    }

    /// The fraction of the range under a pointer at `client` along the track, as the drag maps it.
    fn fraction_at(&self, client: f64) -> Option<f64> {
        Some(self.along_at(client)?.clamp(0.0, 1.0))
    }

    /// [`fraction_at`](Self::fraction_at) unclamped. The thumb's centre travels
    /// between two half-thumb insets, so the pointer maps over that span.
    fn along_at(&self, client: f64) -> Option<f64> {
        if self.travel <= 0.0 {
            return None;
        }
        let along = (client - self.left - self.thumb_along() / 2.0) / self.travel;
        Some(if self.flipped() { 1.0 - along } else { along })
    }
}

/// A pointer's client coordinate along the track.
fn along(point: DragPoint, vertical: bool) -> f64 {
    match vertical {
        true => point.y,
        false => point.x,
    }
}

/// The geometry a bubble is placed from, off the track's and a thumb's size and offset.
fn track_geometry(
    track: Dimensions,
    (track_left, track_top): (f64, f64),
    thumb: Dimensions,
    (thumb_left, thumb_top): (f64, f64),
    rtl: bool,
    vertical: bool,
) -> ThumbTrack {
    let (left, travel, top) = match vertical {
        true => (track_top, track.height - thumb.height, thumb_left),
        false => (track_left, track.width - thumb.width, thumb_top),
    };
    ThumbTrack {
        left,
        travel,
        top,
        thumb,
        rtl,
        vertical,
        stale: false,
    }
}

/// Provided by the video seek: its segmented slider previews the value under a
/// hovering mouse or pen (todo 2166).
#[derive(Clone, Copy)]
pub(crate) struct HoverPreview;

/// The value grid a drag or a key moves on.
#[derive(Clone, Copy)]
struct Grid {
    min: f64,
    max: f64,
    step: f64,
    min_range: f64,
}

impl Grid {
    fn moved(&self, from: SliderCoreValue, index: usize, raw: f64) -> SliderCoreValue {
        from.moved(index, raw, self.min, self.max, self.step, self.min_range)
    }

    /// The value under a pointer at `client`, unsnapped and unclamped: past the end it
    /// still tells two thumbs on one spot apart (1060).
    fn value_at(&self, track: &ThumbTrack, client: f64) -> Option<f64> {
        Some(self.min + track.along_at(client)? * (self.max - self.min))
    }
}

#[derive(Clone, Copy)]
struct Elements {
    root: ElementHandle,
    track: ElementHandle,
    /// Two handles, always: a hook cannot be conditional, and a single-thumb
    /// slider simply never mounts the second.
    thumbs: [ElementHandle; 2],
}

/// The geometry a drag and the hover preview share. The reads start now, awaited later:
/// Blitz locks the document while tasks drain and fails a read made inside one.
fn measure_track(
    elements: Elements,
    vertical: bool,
) -> impl Future<Output = Option<ThumbTrack>> + 'static {
    let (track_size, track_offset) = (elements.track.dimensions(), elements.track.client_offset());
    let (thumb_size, thumb_offset) = (
        elements.thumbs[0].dimensions(),
        elements.thumbs[0].client_offset(),
    );
    let rtl = elements.root.is_rtl();
    async move {
        let (Ok(track), Ok(track_at), Ok(thumb), Ok(thumb_at)) = (
            track_size.await,
            track_offset.await,
            thumb_size.await,
            thumb_offset.await,
        ) else {
            return None;
        };
        Some(track_geometry(
            track, track_at, thumb, thumb_at, rtl, vertical,
        ))
    }
}

/// What a drag calls, all `Copy` so a handler can hold them.
#[derive(Clone, Copy)]
struct Steps {
    emit: Callback<SliderChangeEvent<SliderCoreValue>>,
    /// The value under a client coordinate along the track, unsnapped; `moved` applies the grid.
    position_at: Callback<f64, Option<f64>>,
    grab: Callback<(f64, f64), usize>,
    slide: Callback<f64>,
    /// The drag's own last value, which the parent may not have echoed yet.
    /// Never rendered, so a write redraws nothing.
    latest: CopyValue<SliderCoreValue>,
    /// Which thumb the pointer grabbed. Always 0 for a single thumb. A signal:
    /// the thumbs' scope reads it for the open bubble.
    active: Signal<usize>,
}

/// The drag's steps, all through `use_callback` so they see this render's
/// bounds - the handlers `use_drag` holds do not.
fn use_steps(
    live: Signal<Live>,
    grid: Grid,
    thumb_track: Signal<Option<ThumbTrack>>,
    oninput: Option<EventHandler<SliderChangeEvent<SliderCoreValue>>>,
    thumbs: [ElementHandle; 2],
    focusable: bool,
    press_focus: &PressFocus,
) -> Steps {
    let mut latest = use_hook(|| CopyValue::new(live.peek().value));
    let mut active = use_signal(|| 0_usize);
    let emit = use_callback(move |event: SliderChangeEvent<SliderCoreValue>| {
        latest.set(event.value());
        if let Some(oninput) = &oninput {
            oninput.call(event);
        }
    });
    let position_at = use_callback(move |client: f64| {
        let track = (*thumb_track.peek())?;
        grid.value_at(&track, client)
    });

    // `heading`: where the pointer already went, which picks between two
    // thumbs on one spot. Unmoved, the press's side holds until `slide`.
    let mut undecided = use_hook(|| CopyValue::new(None::<f64>));
    let grab = use_callback(move |(raw, heading): (f64, f64)| {
        let value = live.peek().value;
        let choice = value.nearest(raw, heading);
        undecided.set(choice.is_none().then_some(raw));
        let index = choice.unwrap_or_else(|| usize::from(raw > value.thumb(1)));
        active.set(index);
        emit.call(SliderChangeEvent::Start(grid.moved(value, index, raw)));
        index
    });
    let press_focus = press_focus.clone();
    let slide = use_callback(move |raw: f64| {
        let pressed = *undecided.peek();
        if let Some(pressed) = pressed.filter(|pressed| *pressed != raw) {
            undecided.set(None);
            let index = usize::from(raw > pressed);
            active.set(index);
            let thumb = thumbs[index];
            if focusable && !thumb.is_focused() {
                let press_focus = press_focus.clone();
                press_focus.mark();
                let _ = thumb.focus();
                spawn(async move {
                    next_task().await;
                    press_focus.clear();
                });
            }
        }
        // Copied out first: `emit` writes `latest`.
        let (from, index) = (*latest.peek(), *active.peek());
        emit.call(SliderChangeEvent::Change(grid.moved(from, index, raw)));
    });
    Steps {
        emit,
        position_at,
        grab,
        slide,
        latest,
        active,
    }
}

type TrackMeasure = Pin<Box<dyn Future<Output = Option<ThumbTrack>>>>;

/// The value and geometry under a hovering mouse or pen.
#[derive(Clone, Copy)]
struct Hover {
    hovered: Signal<Option<(f64, ThumbTrack)>>,
    /// Escape hid the bubble: it stays hidden till the pointer leaves (WCAG 1.4.13).
    muted: CopyValue<bool>,
    track: CopyValue<Option<ThumbTrack>>,
    x: CopyValue<Option<f64>>,
    measuring: CopyValue<Option<TrackMeasure>>,
    /// Counts resets, so a measure that lands after one is dropped.
    resets: CopyValue<u32>,
    refresh: Callback<()>,
    vertical: bool,
}

fn use_hover(live: Signal<Live>, grid: Grid, elements: Elements, vertical: bool) -> Hover {
    let Grid { min, max, step, .. } = grid;
    let mut hovered = use_signal(|| None::<(f64, ThumbTrack)>);
    let muted = use_hook(|| CopyValue::new(false));
    let mut track = use_hook(|| CopyValue::new(None::<ThumbTrack>));
    let x = use_hook(|| CopyValue::new(None::<f64>));
    let mut busy = use_hook(|| CopyValue::new(false));
    let mut dirty = use_hook(|| CopyValue::new(false));
    let mut measuring = use_hook(|| CopyValue::new(None::<TrackMeasure>));
    let resets = use_hook(|| CopyValue::new(0u32));

    // Requests coalesce into one update per task: a WebView sends each move over IPC.
    let refresh = use_callback(move |()| {
        dirty.set(true);
        // The reads start here: Blitz fails one made inside a task (the document is borrowed).
        if track.peek().is_none() && measuring.peek().is_none() {
            measuring.set(Some(Box::pin(measure_track(elements, vertical))));
        }
        if std::mem::replace(&mut *busy.write(), true) {
            return;
        }
        spawn(async move {
            while std::mem::replace(&mut *dirty.write(), false) {
                let pending = measuring.write().take();
                if let Some(pending) = pending
                    && track.peek().is_none()
                {
                    let reset = *resets.peek();
                    let measured = pending.await;
                    // A resize meanwhile queued a fresh measure for the next pass.
                    if *resets.peek() == reset {
                        track.set(measured);
                    }
                }
                next_task().await;
                let geometry = track.peek().filter(|_| !*muted.peek());
                let shown = geometry.zip(*x.peek()).and_then(|(track, x)| {
                    // Over a thumb its own bubble shows the value.
                    let on_thumb = live
                        .peek()
                        .value
                        .thumbs()
                        .any(|thumb| track.covers(fraction(thumb, min, max), x));
                    let value = SliderCoreValue::Single(min + track.fraction_at(x)? * (max - min))
                        .snapped(min, max, step)
                        .thumb(0);
                    (!on_thumb).then_some((value, track))
                });
                if *hovered.peek() != shown {
                    hovered.set(shown);
                }
            }
            busy.set(false);
        });
    });
    Hover {
        hovered,
        muted,
        track,
        x,
        measuring,
        resets,
        refresh,
        vertical,
    }
}

impl Hover {
    fn clear(&self) {
        let mut hovered = self.hovered;
        if hovered.peek().is_some() {
            hovered.set(None);
        }
    }

    fn leave(&self) {
        let (mut x, mut track, mut muted) = (self.x, self.track, self.muted);
        x.set(None);
        track.set(None);
        muted.set(false);
        self.clear();
    }

    /// A resize or fullscreen moves the track: measured again for the pointer still on it.
    fn resize(&self) {
        let (mut track, mut measuring, mut resets) = (self.track, self.measuring, self.resets);
        track.set(None);
        measuring.set(None);
        *resets.write() += 1;
        if self.x.peek().is_some() {
            self.refresh.call(());
        }
    }

    fn pointer_moved(&self, event: &Event<PointerData>) {
        let pointer = event.data().pointer_type();
        if *self.muted.peek() || !matches!(pointer.as_str(), "mouse" | "pen") {
            return;
        }
        let mut x = self.x;
        let client = event.data().client_coordinates();
        x.set(Some(if self.vertical { client.y } else { client.x }));
        self.refresh.call(());
    }
}

/// Everything a pointer drag reads and writes.
#[derive(Clone)]
struct DragContext {
    editable: bool,
    focusable: bool,
    elements: Elements,
    steps: Steps,
    thumb_track: Signal<Option<ThumbTrack>>,
    starting: CopyValue<Starting>,
    press_focus: PressFocus,
    vertical: bool,
}

impl DragContext {
    /// Replays what waited, or drops it on a cancel.
    fn settle(&self, grabbed: bool) {
        let mut starting = self.starting;
        let waited = *starting.peek();
        starting.set(Starting::default());
        if !grabbed {
            return;
        }
        let steps = self.steps;
        if let Some(raw) = waited.moved_to.and_then(|x| steps.position_at.call(x)) {
            steps.slide.call(raw);
        }
        if waited.released {
            // Copied out first: `emit` writes `latest`.
            let last = *steps.latest.peek();
            steps.emit.call(SliderChangeEvent::End(last));
        }
    }

    fn start(&self, event: DragStart) {
        if !self.editable {
            event.cancel.call(());
            return;
        }

        let mut starting = self.starting;
        starting.set(Starting {
            measuring: true,
            ..Starting::default()
        });
        let (elements, vertical) = (self.elements, self.vertical);
        let mut thumb_track = self.thumb_track;
        let track = *thumb_track.peek();
        if let Some(track) = track.filter(|track| !track.stale) {
            thumb_track.set(Some(ThumbTrack {
                stale: true,
                ..track
            }));
        }
        // Any focus till the task ends is the pointer's, the web's focus of
        // a pressed thumb too: a touch matches `:focus-visible` (1058).
        self.press_focus.mark();
        let measure = measure_track(elements, vertical);
        let ctx = self.clone();
        spawn(async move {
            let Some(measured) = measure.await else {
                ctx.abandon(event);
                return;
            };
            // `position_at` maps the drag through this same geometry.
            thumb_track.set(Some(measured));

            let steps = ctx.steps;
            match steps.position_at.call(along(event.client, vertical)) {
                Some(raw) => {
                    let heading = ctx
                        .starting
                        .peek()
                        .moved_to
                        .and_then(|x| steps.position_at.call(x))
                        .unwrap_or(raw);
                    // `use_drag` cancels the pointerdown and so its focus;
                    // the grabbed thumb is focused here instead.
                    let index = steps.grab.call((raw, heading));
                    ctx.settle(true);
                    let thumb = elements.thumbs[index];
                    if ctx.focusable && !thumb.is_focused() {
                        // A track press is outside the thumb's `Tooltip`:
                        // tell it this focus is the pointer's (todo 476).
                        ctx.press_focus.mark();
                        let _ = thumb.focus();
                    }
                    next_task().await;
                    ctx.press_focus.clear();
                }
                None => ctx.abandon(event),
            }
        });
    }

    /// Ends a press whose geometry could not be measured.
    fn abandon(&self, event: DragStart) {
        self.press_focus.clear();
        self.settle(false);
        event.cancel.call(());
    }

    fn drag_move(&self, event: DragMove) {
        let waiting = *self.starting.peek();
        if waiting.measuring {
            let mut starting = self.starting;
            starting.set(Starting {
                moved_to: Some(along(event.client, self.vertical)),
                ..waiting
            });
            return;
        }
        if let Some(raw) = self
            .steps
            .position_at
            .call(along(event.client, self.vertical))
        {
            self.steps.slide.call(raw);
        }
    }

    fn drag_end(&self) {
        let waiting = *self.starting.peek();
        if waiting.measuring {
            let mut starting = self.starting;
            starting.set(Starting {
                released: true,
                ..waiting
            });
            return;
        }
        let last = *self.steps.latest.peek();
        self.steps.emit.call(SliderChangeEvent::End(last));
    }
}

/// The value a navigation key asks for, `None` for any other key.
fn key_target(
    key: Key,
    thumb: f64,
    grid: Grid,
    shift: bool,
    slider: &SliderDefaults,
) -> Option<f64> {
    // `step: 0` has no grid, so a key moves 1% of the range, as a native
    // `step="any"` range input does.
    let unit = if grid.step > 0.0 {
        grid.step
    } else {
        (grid.max - grid.min) / 100.0
    };
    let distance = if shift { slider.big_step } else { slider.step } * unit;
    // Under RTL ArrowLeft raises the value, as on a native range.
    Some(match key {
        Key::Named(NamedKey::ArrowRight) | Key::Named(NamedKey::ArrowUp) => thumb + distance,
        Key::Named(NamedKey::ArrowLeft) | Key::Named(NamedKey::ArrowDown) => thumb - distance,
        Key::Named(NamedKey::PageUp) => thumb + slider.big_step * unit,
        Key::Named(NamedKey::PageDown) => thumb - slider.big_step * unit,
        Key::Named(NamedKey::Home) => grid.min,
        Key::Named(NamedKey::End) => grid.max,
        _ => return None,
    })
}

fn visible_marks(marks: &[SliderMark], min: f64, max: f64) -> Vec<SliderMark> {
    let kept: Vec<SliderMark> = marks
        .iter()
        .filter(|mark| on_track(mark.value, min, max))
        .cloned()
        .collect();
    if kept.len() < marks.len() {
        warn(
            "Slider: a mark outside `min..=max` (or past an off-grid `max`'s last step) is dropped.",
        );
    }
    kept
}

/// Two thumbs are one control: a labelled group names them together. A single
/// thumb already carries the label itself.
fn group_naming(props: &SliderCoreProps) -> (Option<&'static str>, Option<String>, Option<String>) {
    let range = matches!(props.value, SliderCoreValue::Range { .. });
    let grouped = range.then(|| props.labelledby.clone()).flatten();
    let group_label = (range && grouped.is_none())
        .then(|| props.group_label.clone())
        .flatten();
    let role = (grouped.is_some() || group_label.is_some()).then_some("group");
    (role, grouped, group_label)
}

#[component]
fn SliderBody(live: Signal<Live>, core: SliderCoreProps) -> Element {
    let props = core;
    let theme = use_theme();
    let elements = Elements {
        root: use_element(),
        track: use_element(),
        thumbs: [use_element(), use_element()],
    };
    // Names each thumb's value bubble, so the thumb can point at it.
    let bubble_id = use_id();
    let press_focus = use_context_provider(PressFocus::default);

    let (min, max) = grid_bounds(props.min, props.max, props.step);
    let grid = Grid {
        min,
        max,
        step: props.step.max(0.0),
        min_range: props.min_range.max(0.0),
    };
    let size = props.size.copied_or(theme.slider.size);
    let color = props
        .color
        .as_ref()
        .cloned()
        .unwrap_or_else(|| ThemeAwareValue::from(theme.slider.color));
    let color = base_color(Some(&color));
    let disabled = props.disabled.unwrap_or(false);

    let interactive = props.oninput.is_some() && !disabled;
    // `editable` moves the value; any enabled thumb is a tab stop, so a
    // read-only one is still reached, read and posted.
    let editable = interactive && !props.readonly;
    let focusable = props.focusable;
    let vertical = props.vertical;
    if vertical && (props.bars.is_some() || !props.segments.is_empty()) {
        warn("Slider: a vertical track draws a line; `bars` and `segments` are dropped.");
    }
    let bars = props.bars.clone().filter(|_| !vertical);

    if props.oninput.is_none() && !disabled {
        warn("Slider: `value` without `oninput` can never change.");
    }

    let thumb_track = use_signal(|| None::<ThumbTrack>);
    let steps = use_steps(
        live,
        grid,
        thumb_track,
        props.oninput,
        elements.thumbs,
        focusable,
        &press_focus,
    );
    // The hover preview's value and geometry, measured at the first move over the slider.
    // The hook runs on every render, so toggling `preview_on_hover` keeps the hook order (4260).
    let in_preview = try_use_context::<HoverPreview>().is_some();
    let previews = props.preview_on_hover || in_preview && !props.segments.is_empty();
    let hover = use_hover(live, grid, elements, vertical);

    // Moves and a release that arrive before the measuring task has grabbed
    // (a sideways touch starts mid-move) wait for the `Start`.
    let starting = use_hook(|| CopyValue::new(Starting::default()));
    // Set by a thumb's `pointerdown`, which runs before the root's: only a
    // touch that began on a thumb drags (1059).
    let mut on_thumb = use_hook(|| CopyValue::new(false));

    let ctx = DragContext {
        editable,
        focusable,
        elements,
        steps,
        thumb_track,
        starting,
        press_focus,
        vertical,
    };
    let (on_start, on_move, on_end) = (ctx.clone(), ctx.clone(), ctx);
    let options = DragOptions {
        capture: elements.root,
        onstart: Callback::new(move |event: DragStart| on_start.start(event)),
        onmove: Callback::new(move |event: DragMove| on_move.drag_move(event)),
        onend: Callback::new(move |_| on_end.drag_end()),
    };
    let drag = use_sideways_drag_if(
        options,
        Callback::new(move |()| *on_thumb.peek()),
        !vertical,
    );
    let onpointerdown = use_callback(move |event: Event<PointerData>| {
        drag.onpointerdown.call(event);
        on_thumb.set(false);
        hover.clear();
    });
    let onthumbdown = use_callback(move |_: Event<PointerData>| on_thumb.set(true));
    let onpointermove = use_callback(move |event: Event<PointerData>| {
        drag.onpointermove.call(event.clone());
        if previews && interactive && !*drag.dragging.peek() {
            hover.pointer_moved(&event);
        }
    });
    let onpointerleave = move |_: Event<PointerData>| hover.leave();
    let onresize = move |_: Event<ResizeData>| hover.resize();

    // One handler for both thumbs: the focused thumb is the one the keys
    // move, so the index comes from whichever element fired.
    let onkeydown = use_callback(move |(index, event): (usize, Event<KeyboardData>)| {
        // Alt+ArrowLeft is Back, Ctrl+PageUp switches tabs: a chord is the browser's.
        if !editable || has_shortcut_modifier(&event) {
            return;
        }
        let value = live.peek().value;
        let shift = event.modifiers().shift();
        let key = logical_key(&event);
        if let Some(raw) = key_target(key, value.thumb(index), grid, shift, &theme.slider) {
            event.prevent_default();
            // `Change` then `End`: a key press settles at once, and callers commit
            // on `End`.
            let moved = grid.moved(value, index, raw);
            steps.emit.call(SliderChangeEvent::Change(moved));
            steps.emit.call(SliderChangeEvent::End(moved));
        }
    });

    let marks = visible_marks(&props.marks, min, max);
    let marks_labeled = marks.iter().any(|mark| mark.label.is_some());
    let (segments, dropped) = match vertical {
        true => (Vec::new(), false),
        false => track_segments(&props.segments, min, max),
    };
    if dropped {
        warn("Slider: a segment starting outside `min..max`, or at another's start, is dropped.");
    }
    let segment_text = use_localization().slider.segment;
    let (role, grouped, group_label) = group_naming(&props);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with("dragging", (drag.dragging)())
        .with("disabled", disabled)
        .with("readonly", props.readonly)
        .with("marks-labeled", marks_labeled)
        .with("plain", props.plain)
        .with("bars", bars.is_some())
        .with("vertical", vertical)
        .with("segments", !segments.is_empty())
        .into();

    let root_variables: Input<Variables> = slider_variables(&color).into();
    let track_class = use_css(Some(&SLIDER_TRACK_SX), CssLayer::Framework);
    let track_style = props
        .track
        .as_ref()
        .map(|background| format!("background: {background}"));

    let hidden = props.name.clone().map(|name| {
        rsx! { SliderHidden { live, name, disabled } }
    });
    let preview = previews.then(|| {
        rsx! {
            SliderHoverPreview {
                hovered: hover.hovered,
                muted: hover.muted,
                dragging: drag.dragging,
                min,
                max,
                size,
                label: props.label,
                segments: segments.clone(),
                segment_text,
                group_digits: props.group_digits,
            }
        }
    });

    use_box()
        .framework_sx(&SLIDER_ROOT_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&root_variables)
        .prepare()
        .attr("data-slot", props.slot)
        .attr("role", role)
        .attr("aria-labelledby", grouped)
        .attr("aria-label", group_label)
        .element(&elements.root)
        .event("onpointerdown", onpointerdown)
        .event("onpointermove", onpointermove)
        .event("onpointerup", drag.onpointerup)
        .event("onpointercancel", drag.onpointercancel)
        .event("onpointerleave", previews.then_some(onpointerleave))
        .event("onresize", previews.then_some(onresize))
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                div {
                    class: track_class,
                    "data-slot": SliderPart::Track.slot(),
                    style: track_style,
                    onmounted: elements.track.mount(),
                    SliderThumbs {
                        live,
                        min,
                        max,
                        step: grid.step,
                        min_range: grid.min_range,
                        size,
                        interactive,
                        disabled,
                        label: props.label,
                        marks,
                        captions_hidden: props.captions_hidden,
                        aria_labels: [props.aria_label, props.aria_label_to],
                        labelledby: props.labelledby,
                        describedby: props.describedby,
                        invalid: props.invalid,
                        readonly: props.readonly,
                        plain: props.plain,
                        bars,
                        vertical,
                        segments,
                        segment_text,
                        group_digits: props.group_digits,
                        focusable,
                        bubble_id,
                        thumb_elements: elements.thumbs,
                        onkeydown,
                        onthumbdown,
                        dragging: drag.dragging,
                        active: steps.active,
                        thumb_track,
                    }
                }
                {preview}
                {hidden}
            },
        )
}

/// The bar, the marks and the thumbs: everything a value move redraws.
#[derive(Props, Clone, PartialEq)]
struct SliderThumbsProps {
    live: Signal<Live>,
    min: f64,
    max: f64,
    step: f64,
    min_range: f64,
    size: Size,
    interactive: bool,
    disabled: bool,
    label: Option<Callback<f64, String>>,
    marks: Vec<SliderMark>,
    captions_hidden: bool,
    aria_labels: [Option<String>; 2],
    labelledby: Option<String>,
    describedby: Option<String>,
    invalid: bool,
    readonly: bool,
    plain: bool,
    bars: Option<Vec<f64>>,
    vertical: bool,
    segments: Vec<TrackSegment>,
    segment_text: &'static str,
    group_digits: bool,
    focusable: bool,
    bubble_id: Signal<String>,
    thumb_elements: [ElementHandle; 2],
    onkeydown: Callback<(usize, Event<KeyboardData>)>,
    onthumbdown: Callback<Event<PointerData>>,
    dragging: Signal<bool>,
    active: Signal<usize>,
    thumb_track: Signal<Option<ThumbTrack>>,
}

#[component]
fn SliderThumbs(props: SliderThumbsProps) -> Element {
    let SliderThumbsProps {
        live,
        min,
        max,
        step,
        min_range,
        size,
        interactive,
        focusable,
        bubble_id,
        thumb_elements,
        onkeydown,
        onthumbdown,
        dragging,
        active,
        thumb_track,
        ..
    } = props;
    let Live { value, thumb_fill } = live();
    let bar = value.bar(min, max);

    let vertical = props.vertical;
    let bar_class = use_css(
        Some(match (value, vertical) {
            (SliderCoreValue::Single(_), false) => &SLIDER_BAR_SX,
            (SliderCoreValue::Range { .. }, false) => &SLIDER_RANGE_BAR_SX,
            (SliderCoreValue::Single(_), true) => &SLIDER_BAR_VERTICAL_SX,
            (SliderCoreValue::Range { .. }, true) => &SLIDER_RANGE_BAR_VERTICAL_SX,
        }),
        CssLayer::Framework,
    );
    let (mark_sx, mark_label_sx, anchor_sx) = match vertical {
        false => (
            &SLIDER_MARK_SX,
            &SLIDER_MARK_LABEL_SX,
            &SLIDER_THUMB_ANCHOR_SX,
        ),
        true => (
            &SLIDER_MARK_VERTICAL_SX,
            &SLIDER_MARK_LABEL_VERTICAL_SX,
            &SLIDER_THUMB_ANCHOR_VERTICAL_SX,
        ),
    };
    let mark_class = use_css(Some(mark_sx), CssLayer::Framework);
    let mark_label_class = use_css(Some(mark_label_sx), CssLayer::Framework);
    let anchor_class = use_css(Some(anchor_sx), CssLayer::Framework);
    // One prepared style, cloned per thumb: the two are identical, and a
    // second `use_box` would be a second hook for nothing.
    let thumb_style = use_box().framework_sx(&SLIDER_THUMB_SX).prepare();
    // The bare value's bubble, `0,5` under `Formats::GERMAN`.
    let formats = use_formats();

    let captions_hidden = props.captions_hidden.then_some("true");
    let marks = props.marks.iter().map(|mark| {
        let mark_at = fraction(mark.value, min, max);
        let at = variables()
            .with(SLIDER_MARK_AT, Some(mark_at.to_string()))
            .render();
        // A state, not a var set only when filled: a raw `style` that drops a
        // declaration keeps its last value ([[codebase/css-vars]]).
        let filled = (bar.0 <= mark_at && mark_at <= bar.1).then_some("filled");
        let caption = mark.label.clone().map(|label| {
            rsx! {
                span {
                    class: mark_label_class.clone(),
                    "data-slot": SliderPart::MarkLabel.slot(),
                    style: "{at}",
                    "aria-hidden": captions_hidden,
                    {label}
                }
            }
        });
        rsx! {
            if !on_boundary(&props.segments, mark.value) {
                span {
                    class: mark_class.clone(),
                    "data-slot": SliderPart::Mark.slot(),
                    "data-state": filled,
                    style: "{at}",
                }
            }
            {caption}
        }
    });

    let aria_labels = &props.aria_labels;
    let range = matches!(value, SliderCoreValue::Range { .. });
    let thumbs = value.thumbs().enumerate().map(|(index, thumb_value)| {
        let (thumb_min, thumb_max) = value.bounds(index, min, max, step, min_range);
        let (text, bubble_text) = value_text(
            props.label,
            &props.segments,
            props.segment_text,
            thumb_value,
            formats,
            props.group_digits,
        );
        // `aria-labelledby` beats `aria-label`, so a range thumb lists itself
        // after the label ("Price Minimum"); a single thumb's label wins alone.
        let aria_label = aria_labels[index]
            .clone()
            .filter(|label| !label.trim().is_empty());
        let (own_id, labelledby) = match (&props.labelledby, &aria_label) {
            (Some(label), Some(_)) if range => {
                let own_id = format!("{label}-thumb-{index}");
                let labelledby = format!("{label} {own_id}");
                (Some(own_id), Some(labelledby))
            }
            _ => (None, props.labelledby.clone()),
        };

        // A `plain` slider has no bubble. It goes after the captions: the
        // value is already in `aria-valuenow`.
        let bubble_id = (!props.plain).then(|| format!("{}-value-{index}", bubble_id.read()));
        let describedby = [props.describedby.clone(), bubble_id.clone()]
            .into_iter()
            .flatten()
            .reduce(|captions, bubble| format!("{captions} {bubble}"));

        let thumb = thumb_style
            .clone()
            .attr("data-slot", SliderPart::Thumb.slot())
            .attr("role", "slider")
            .attr(
                "tabindex",
                if !props.disabled && focusable {
                    "0"
                } else {
                    "-1"
                },
            )
            .attr(
                "aria-orientation",
                if vertical { "vertical" } else { "horizontal" },
            )
            .attr("aria-valuemin", thumb_min)
            .attr("aria-valuemax", thumb_max)
            .attr("aria-valuenow", thumb_value)
            .attr("aria-valuetext", text)
            .attr("id", own_id)
            .attr("aria-label", aria_label)
            .attr("aria-labelledby", labelledby)
            .attr("aria-describedby", describedby)
            .attr("aria-invalid", props.invalid.then_some("true"))
            // No `aria-required`: ARIA 1.2 does not allow it on `slider`, and a
            // slider always holds a value anyway (todo 532).
            .attr("aria-disabled", props.disabled)
            // No `oninput` on an enabled slider: it looks enabled, so it reads as read-only.
            .attr(
                "aria-readonly",
                (props.readonly || !interactive && !props.disabled).then_some("true"),
            )
            .element(&thumb_elements[index])
            .event("onkeydown", move |event: Event<KeyboardData>| {
                onkeydown.call((index, event))
            })
            .event("onpointerdown", onthumbdown)
            .render(HtmlTag::Span, Vec::new(), rsx! {});

        let at = variables()
            .with(
                SLIDER_THUMB_AT,
                Some(fraction(thumb_value, min, max).to_string()),
            )
            .with(SLIDER_THUMB_FILL, thumb_fill.clone())
            .render();

        if props.plain {
            return rsx! {
                span { class: anchor_class.clone(), style: "{at}", {thumb} }
            };
        }

        // Clear of the thumb's hit area, which overhangs a sub-24px thumb by
        // up to 6px: the bubble took those presses (todo 649).
        let gap = Size::Sm;
        // A drag's bubble is pinned to the value, and takes over the id; a
        // disabled thumb opens none (todo 596). Kept mounted after its first drag.
        let track = thumb_track().filter(|_| !props.disabled);
        let shown = track.is_some_and(|track| !track.stale && dragging() && active() == index);
        let pinned = track.map(|track| {
            rsx! {
                TooltipPinned {
                    label: rsx! { {bubble_text.clone()} },
                    anchor: track.at(fraction(thumb_value, min, max)),
                    size,
                    gap,
                    side: vertical.then_some(Side::End),
                    id: if shown { bubble_id.clone() } else { None },
                    rtl: track.rtl,
                    shown,
                }
            }
        });
        let (open, label_id) = match (props.disabled, shown) {
            (true, _) => (Some(false), bubble_id),
            (false, true) => (Some(false), None),
            (false, false) => (None, bubble_id),
        };
        rsx! {
            span { class: anchor_class.clone(), style: "{at}",
                {pinned}
                Tooltip {
                    label: rsx! { {bubble_text} },
                    size,
                    gap,
                    side: vertical.then_some(Side::End),
                    open,
                    label_id,
                    // No bridge: it would take the thumb's presses, and Blitz
                    // hits even an overflowing box.
                    sx: sx().selector("&::before", sx().display("none")),
                    {thumb}
                }
            }
        }
    });

    let bars_class = use_css(Some(&SLIDER_BARS_SX), CssLayer::Framework);
    let bars = props.bars.as_ref().map(|heights| {
        let filled = filled_bars(bar.1, heights.len());
        rsx! {
            div { class: bars_class, "data-slot": SliderPart::Bars.slot(), "aria-hidden": "true",
                for (index, height) in heights.iter().enumerate() {
                    span {
                        key: "{index}",
                        "data-state": (index < filled).then_some("filled"),
                        style: "height: {height.clamp(0.0, 1.0) * 100.0:.0}%",
                    }
                }
            }
        }
    });

    let segments_class = use_css(Some(&SLIDER_SEGMENTS_SX), CssLayer::Framework);
    let segment_class = use_css(Some(&SLIDER_SEGMENT_SX), CssLayer::Framework);
    let segment_fill_class = use_css(Some(&SLIDER_SEGMENT_FILL_SX), CssLayer::Framework);
    let segmented = !props.segments.is_empty();
    let segments = segmented.then(|| {
        let pieces = props.segments.iter().map(|segment| {
            let (start, end) = (fraction(segment.start, min, max), fraction(segment.end, min, max));
            let at = variables()
                .with(SLIDER_SEGMENT_AT, Some(start.to_string()))
                .with(SLIDER_SEGMENT_SPAN, Some((end - start).to_string()))
                .render();
            let filled = variables()
                .with(SLIDER_SEGMENT_FILLED, Some(segment_filled(start, end, bar.1).to_string()))
                .render();
            rsx! {
                span { class: segment_class.clone(), "data-slot": SliderPart::Segment.slot(), style: at,
                    span {
                        class: segment_fill_class.clone(),
                        "data-slot": SliderPart::SegmentFill.slot(),
                        style: filled,
                    }
                }
            }
        });
        rsx! {
            div { class: segments_class, "data-slot": SliderPart::Segments.slot(), "aria-hidden": "true",
                {pieces}
            }
        }
    });

    let bar_style = bar_variables(bar);
    rsx! {
        {bars}
        {segments}
        if !props.plain && props.bars.is_none() && !segmented {
            div { class: bar_class, "data-slot": SliderPart::Bar.slot(), style: bar_style }
        }
        {marks}
        {thumbs}
    }
}

/// The posted values, in their own scope for the same reason as the thumbs.
#[component]
fn SliderHidden(live: Signal<Live>, name: String, disabled: bool) -> Element {
    let inputs = live.read().value.thumbs().map(move |thumb_value| {
        // `Some(true)` or nothing: native writes `false` as a string, which
        // reads as disabled.
        rsx! {
            input {
                r#type: "hidden",
                name: name.clone(),
                value: "{thumb_value}",
                disabled: disabled.then_some(true),
            }
        }
    });
    // A range posts its two values under one name, in track order:
    // `FormData::get_all` reads them back as a pair.
    rsx! { {inputs} }
}

#[cfg(test)]
mod tests {
    use super::{
        Dimensions, SliderSegment, ThumbTrack, TrackSegment, filled_bars, on_boundary,
        segment_filled, segment_label, track_segments, value_text,
    };
    use crate::localization::Formats;

    /// Todo 3736: the fill and thumb ring carry the value, so they need 3:1 on the page (1.4.11).
    #[test]
    fn every_status_fill_reaches_3_to_1_on_the_page() {
        use super::{SLIDER_COLOR, slider_variables};
        use crate::{
            components::{common::base_color, feedback::sheet_hex},
            css::Stylesheet,
            sx::ThemeAwareValue,
            theme::ThemeSet,
        };

        let mut short = Vec::new();
        for set in ThemeSet::CATALOGUE {
            for theme in [Some(set.light_theme()), set.dark_theme()]
                .into_iter()
                .flatten()
            {
                let css = Stylesheet::from(theme).as_str().to_string();
                for name in ["primary", "error", "info", "success", "warning"] {
                    let color = base_color(Some(&ThemeAwareValue::from(name)));
                    let fill = slider_variables(&color).to_string();
                    let fill = fill
                        .trim_start_matches(&format!("{}:", SLIDER_COLOR.name()))
                        .trim_end_matches(';')
                        .to_string();
                    let ratio = sheet_hex(&css, fill).contrast_ratio(theme.surface);
                    if ratio < 3.0 {
                        let scheme = theme.surface.color_scheme();
                        short.push(format!("{} {scheme} {name}: {ratio:.2}:1", set.name()));
                    }
                }
            }
        }
        assert!(short.is_empty(), "a fill fell under 3:1: {short:?}");
    }

    /// Todo 3412: a bare bubble groups its thousands only when asked.
    #[test]
    fn a_bare_bubble_groups_its_thousands_on_request() {
        let bubble = |value, formats, group| value_text(None, &[], "", value, formats, group).1;
        assert_eq!(bubble(12_500.5, &Formats::GERMAN, true), "12.500,5");
        assert_eq!(bubble(-1_234_567.0, &Formats::AMERICAN, true), "-1,234,567");
        assert_eq!(bubble(2026.0, &Formats::GERMAN, false), "2026");
        assert_eq!(bubble(0.5, &Formats::GERMAN, false), "0,5");
    }

    fn starts(segments: &[TrackSegment]) -> Vec<(f64, f64, Option<&str>)> {
        segments
            .iter()
            .map(|segment| (segment.start, segment.end, segment.label.as_deref()))
            .collect()
    }

    #[test]
    fn the_segments_are_sorted_bounded_and_led_from_min() {
        let given = [
            SliderSegment::labeled(30.0, "Outro"),
            SliderSegment::labeled(10.0, "Intro"),
            SliderSegment::labeled(10.0, "Again"),
            SliderSegment::labeled(-5.0, "Before"),
            SliderSegment::labeled(40.0, "At max"),
        ];
        let (segments, dropped) = track_segments(&given, 0.0, 40.0);
        assert!(dropped);
        assert_eq!(
            starts(&segments),
            [
                (0.0, 10.0, None),
                (10.0, 30.0, Some("Intro")),
                (30.0, 40.0, Some("Outro"))
            ]
        );
        let (none, dropped) = track_segments(&[SliderSegment::new(5.0)], 0.0, 1.0);
        assert!(
            none.is_empty() && dropped,
            "before the duration loads, `max` is 1"
        );
    }

    #[test]
    fn a_value_takes_its_segments_label() {
        let (segments, _) = track_segments(
            &[SliderSegment::new(0.0), SliderSegment::labeled(5.0, "Two")],
            0.0,
            10.0,
        );
        assert_eq!(segment_label(&segments, 0.0), None);
        assert_eq!(segment_label(&segments, 4.9), None);
        assert_eq!(segment_label(&segments, 5.0), Some("Two"));
        assert_eq!(segment_label(&segments, 10.0), Some("Two"));
    }

    /// Todo 2168: a mark where two segments meet would cover their gap.
    #[test]
    fn only_a_mark_between_two_segments_is_on_a_boundary() {
        let (segments, _) = track_segments(&[SliderSegment::labeled(2.0, "Paid")], 0.0, 3.0);
        assert!(on_boundary(&segments, 2.0));
        assert!(!on_boundary(&segments, 0.0), "the track's start");
        assert!(!on_boundary(&segments, 1.0));
        assert!(!on_boundary(&segments, 3.0), "the track's end");
    }

    #[test]
    fn a_segment_fills_up_to_the_value() {
        assert_eq!(segment_filled(0.25, 0.75, 0.0), 0.0);
        assert_eq!(segment_filled(0.25, 0.75, 0.5), 0.5);
        assert_eq!(segment_filled(0.25, 0.75, 1.0), 1.0);
        assert_eq!(segment_filled(0.5, 0.5, 0.7), 0.0, "an empty segment");
    }

    #[test]
    fn the_filled_bars_follow_the_value() {
        assert_eq!(filled_bars(0.0, 8), 0);
        assert_eq!(filled_bars(0.25, 8), 2);
        assert_eq!(filled_bars(0.3, 8), 2);
        assert_eq!(filled_bars(1.0, 8), 8);
        assert_eq!(filled_bars(1.5, 8), 8, "clamped to the bars there are");
        assert_eq!(filled_bars(0.5, 0), 0);
    }

    /// Todo 3963: an upright track's minimum is at the bottom, and the bubble rides its y.
    #[test]
    fn an_upright_track_maps_from_the_bottom() {
        let thumb = Dimensions {
            width: 20.0,
            height: 20.0,
        };
        let track = super::track_geometry(
            Dimensions {
                width: 4.0,
                height: 220.0,
            },
            (50.0, 100.0),
            thumb,
            (42.0, 0.0),
            false,
            true,
        );
        assert_eq!(track.fraction_at(110.0), Some(1.0), "the top end");
        assert_eq!(track.fraction_at(310.0), Some(0.0), "the bottom end");
        assert_eq!(track.fraction_at(160.0), Some(0.75));
        let at = track.at(0.75);
        assert_eq!((at.x, at.y), (42.0, 150.0));
        assert!(track.covers(0.75, 160.0) && !track.covers(0.75, 200.0));
    }

    /// Todo 2166: the hover preview maps the pointer over the thumb's travel, as a drag does.
    #[test]
    fn a_pointer_maps_to_the_fraction_under_it() {
        let track = |rtl| ThumbTrack {
            left: 100.0,
            travel: 200.0,
            top: 0.0,
            thumb: Dimensions {
                width: 20.0,
                height: 20.0,
            },
            rtl,
            vertical: false,
            stale: false,
        };
        assert_eq!(track(false).fraction_at(110.0), Some(0.0));
        assert_eq!(track(false).fraction_at(160.0), Some(0.25));
        assert_eq!(track(true).fraction_at(160.0), Some(0.75));
        assert_eq!(
            track(false).fraction_at(500.0),
            Some(1.0),
            "clamped past the end"
        );
        let flat = ThumbTrack {
            travel: 0.0,
            ..track(false)
        };
        assert_eq!(flat.fraction_at(110.0), None);
    }

    /// Todo 4265: a drag maps through the hover's `ThumbTrack`, unclamped past the end.
    #[test]
    fn a_drag_maps_the_pointer_as_the_preview_does() {
        let track = ThumbTrack {
            left: 100.0,
            travel: 200.0,
            top: 0.0,
            thumb: Dimensions {
                width: 20.0,
                height: 20.0,
            },
            rtl: true,
            vertical: false,
            stale: false,
        };
        let grid = super::Grid {
            min: 0.0,
            max: 50.0,
            step: 1.0,
            min_range: 0.0,
        };
        for client in [110.0, 160.0, 310.0] {
            let preview = track.fraction_at(client).map(|fraction| fraction * 50.0);
            assert_eq!(grid.value_at(&track, client), preview);
        }
        assert_eq!(grid.value_at(&track, 360.0), Some(-12.5));
    }
}
