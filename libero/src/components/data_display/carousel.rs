use std::{cell::Cell, rc::Rc, time::Duration};

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        accessibility::VisuallyHidden,
        buttons::ActionIcon,
        common::{
            Glyph, HtmlTag, Input, Orientation, States, Variables, base_props, focus_ring_sx,
            has_shortcut_modifier, input_from_str, inset_focus_ring_sx, shadow_sx, states,
            use_name_warning, variables,
        },
        layout::{
            Box, ScrollArea, ScrollAreaBase, ScrollAreaHandle, ScrollPositionEvent, inline_x,
            physical_x, scroll_area_base, use_box, use_scroll_area,
        },
    },
    context::IconSlot,
    hooks::{
        Drag, DragMove, DragOptions, DragStart, ElementHandle, use_drag, use_element,
        use_focus_within, use_id, use_localization, use_theme,
    },
    localization::{CarouselLabels, fill},
    platform::{
        ElementApi, TimerSubscription, arrow_target, key_taken, logical_key,
        prefers_reduced_motion, snaps_scroll, timer, typing_target, when_free,
    },
    sx::{FORCED_COLORS, REDUCED_MOTION, StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        CAROUSEL_CONTROL_BACKGROUND, CAROUSEL_CONTROL_COLOR, CAROUSEL_CONTROL_HOVER_BACKGROUND,
        CAROUSEL_CONTROL_SIZE, CAROUSEL_CONTROLS_OFFSET, CAROUSEL_GAP, CAROUSEL_INDICATOR_COLOR,
        CAROUSEL_INDICATOR_CURRENT_COLOR, CAROUSEL_INDICATOR_CURRENT_LENGTH,
        CAROUSEL_INDICATOR_LENGTH, CAROUSEL_INDICATOR_THICKNESS, CAROUSEL_INDICATORS_GAP,
        CAROUSEL_PER_VIEW, CAROUSEL_RADIUS, CssVar, FOCUS_RING_HALO, FOCUS_RING_OFFSET,
        FOCUS_RING_WIDTH, NamedColorCss, Size, SizeCss,
    },
};

pub use crate::theme::CarouselAlign;

input_from_str!(CarouselAlign);

static CAROUSEL_ROOT_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative")
        .display("block")
        // Sized by its container, not its slides: a shrink-to-fit vertical one collapses.
        .width("100%")
});

static CAROUSEL_VIEWPORT_SX: StaticSx = StaticSx::new(|| {
    // The controls' containing block, not the root, which also holds the dots.
    sx().position("relative").overflow("hidden")
});

/// Merged onto `ScrollArea`'s own; the indicators replace the hidden scrollbar.
static CAROUSEL_TRACK_SX: StaticSx = StaticSx::new(|| {
    scroll_area_base(
        sx().display("flex")
            // `overridable`, not `value`: the props write the `-override` twin.
            .gap(CAROUSEL_GAP.overridable())
            .when("horizontal", sx().flex_direction("row"))
            .when("vertical", sx().flex_direction("column"))
            .when("horizontal", sx().scroll_snap_type("x mandatory"))
            .when("vertical", sx().scroll_snap_type("y mandatory"))
            // Keeps a fast flick from skipping past a slide.
            .scroll_snap_stop("always")
            .overscroll_behavior_x("contain")
            .overscroll_behavior_y("contain")
            .height(CAROUSEL_HEIGHT.value_or("auto"))
            .scroll_behavior("smooth")
            // Chrome and Safari keep smooth scrolling under reduced motion.
            .media(REDUCED_MOTION, sx().scroll_behavior("auto"))
            // A drag follows the pointer: smoothing lags and a snap pulls it back.
            // After the orientation arms, which it beats at equal specificity.
            .when(
                "dragging",
                sx().scroll_behavior("auto").scroll_snap_type("none"),
            )
            .when("seam", sx().scroll_behavior("auto"))
            // Inset: the `overflow: hidden` viewport would clip an outset ring.
            .focus_visible(inset_focus_ring_sx("-2px")),
    )
});

static CAROUSEL_SLIDE_SX: StaticSx = StaticSx::new(|| {
    sx()
        // `min-width: 0` or a slide refuses to shrink below its content.
        .min_width("0")
        .min_height("0")
        .border_radius(CAROUSEL_RADIUS.value())
        .overflow("hidden")
        // The slide and the track clip flush at its edges: room for an outset
        // ring on focusable content at any depth (todos 618, 619).
        .padding(format!(
            "calc({} + {})",
            FOCUS_RING_OFFSET.value(),
            FOCUS_RING_WIDTH.value()
        ))
        .flex(format!(
            "0 0 calc((100% - ({} - 1) * {}) / {})",
            CAROUSEL_PER_VIEW.overridable(),
            CAROUSEL_GAP.overridable(),
            CAROUSEL_PER_VIEW.overridable()
        ))
        .when("align-start", sx().scroll_snap_align("start"))
        .when("align-center", sx().scroll_snap_align("center"))
        .when("align-end", sx().scroll_snap_align("end"))
});

static CAROUSEL_CONTROLS_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .display("flex")
        .justify_content("space-between")
        .align_items("center")
        // Only the buttons take the pointer; the strip underneath keeps it.
        .pointer_events("none")
        .when(
            "horizontal",
            sx().top("0")
                .bottom("0")
                .left(CAROUSEL_CONTROLS_OFFSET.value())
                .right(CAROUSEL_CONTROLS_OFFSET.value()),
        )
        .when(
            "vertical",
            sx().left("0")
                .right("0")
                .top(CAROUSEL_CONTROLS_OFFSET.value())
                .bottom(CAROUSEL_CONTROLS_OFFSET.value())
                .flex_direction("column"),
        )
});

/// Every control's fill and glyph. A var is opaque to `background()`, so the
/// focus contrast and halo are declared by hand.
fn control_colors_sx() -> Sx {
    sx().background(CAROUSEL_CONTROL_BACKGROUND.value())
        .color(CAROUSEL_CONTROL_COLOR.value())
        .var(
            CssVar::Owned(NamedColorCss::FOCUS_CONTRAST.name().to_string()),
            CAROUSEL_CONTROL_COLOR.value(),
        )
        .var(FOCUS_RING_HALO, CAROUSEL_CONTROL_BACKGROUND.value())
}

static CAROUSEL_CONTROL_SX: StaticSx = StaticSx::new(|| {
    sx().pointer_events("auto")
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .width(CAROUSEL_CONTROL_SIZE.value())
        .height(CAROUSEL_CONTROL_SIZE.value())
        .padding("0")
        .border_width("0")
        .border_radius("50%")
        .and(control_colors_sx())
        .and(shadow_sx(SizeCss::SHADOW.value(Size::Sm)))
        .cursor("pointer")
        .selector("& > svg", sx().width("60%").height("60%"))
        // The row runs right to left, so the arrows point the other way.
        .when(
            "horizontal",
            sx().rtl(sx().selector("& > svg", sx().transform("scaleX(-1)"))),
        )
        .hover(sx().background(CAROUSEL_CONTROL_HOVER_BACKGROUND.value()))
        // Disabled keeps its tab stop, so focus is never dropped at either end.
        .when(
            "disabled",
            sx().opacity("0.4")
                .cursor("default")
                .background(CAROUSEL_CONTROL_BACKGROUND.value()),
        )
        .focus_visible(focus_ring_sx())
});

static CAROUSEL_INDICATORS_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .justify_content("center")
        .align_items("center")
        .gap(CAROUSEL_INDICATORS_GAP.value())
        .margin_top("sm")
        .when("vertical", sx().flex_direction("column"))
});

static CAROUSEL_INDICATOR_SX: StaticSx = StaticSx::new(|| {
    sx().padding("0")
        .border_width("0")
        .border_radius("999px")
        // A var, not a literal, so the ring contrasts with the page, not the dot.
        .background(CAROUSEL_INDICATOR_COLOR.value())
        .cursor("pointer")
        .transition("background-color 150ms")
        .when(
            "horizontal",
            sx().width(CAROUSEL_INDICATOR_LENGTH.value())
                .height(CAROUSEL_INDICATOR_THICKNESS.value()),
        )
        .when(
            "vertical",
            sx().height(CAROUSEL_INDICATOR_LENGTH.value())
                .width(CAROUSEL_INDICATOR_THICKNESS.value()),
        )
        // Not colour alone (1.4.1): the current dot is also longer.
        .when(
            "current",
            sx().background(CAROUSEL_INDICATOR_CURRENT_COLOR.value())
                .when(
                    "horizontal",
                    sx().width(CAROUSEL_INDICATOR_CURRENT_LENGTH.value()),
                )
                .when(
                    "vertical",
                    sx().height(CAROUSEL_INDICATOR_CURRENT_LENGTH.value()),
                ),
        )
        .media(REDUCED_MOTION, sx().transition("none"))
        // Forced colours paint every dot `Canvas`, so the strip vanished.
        .media(
            FORCED_COLORS,
            sx().background("CanvasText")
                .when("current", sx().background("Highlight")),
        )
        .focus_visible(focus_ring_sx())
        // 2.5.8: an invisible 24px box centred on the thin dot takes the pointer.
        .position("relative")
        .selector(
            "&::before",
            sx().content("\"\"")
                .position("absolute")
                .top("50%")
                .left("50%")
                .width("max(100%, 24px)")
                .height("max(100%, 24px)")
                .transform("translate(-50%, -50%)"),
        )
});

static CAROUSEL_PAUSE_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        // First in the DOM for Tab order, so the viewport would paint over it.
        .z_index("1")
        .bottom(CAROUSEL_CONTROLS_OFFSET.value())
        .right(CAROUSEL_CONTROLS_OFFSET.value())
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .width(CAROUSEL_CONTROL_SIZE.value())
        .height(CAROUSEL_CONTROL_SIZE.value())
        .padding("0")
        .border_width("0")
        .border_radius("50%")
        .and(control_colors_sx())
        .and(shadow_sx(SizeCss::SHADOW.value(Size::Sm)))
        .cursor("pointer")
        .selector("& > svg", sx().width("55%").height("55%"))
        // Beside Next's column: on a short strip it covered Next.
        .when(
            "beside-next",
            sx().right(format!(
                "calc(2 * {} + {})",
                CAROUSEL_CONTROLS_OFFSET.value(),
                CAROUSEL_CONTROL_SIZE.value()
            )),
        )
        .focus_visible(focus_ring_sx())
});

/// The snappable indices, inclusive: six slides three-up reach 0..=3 at start,
/// 1..=4 centred and 2..=5 end-aligned, since the browser clamps the scroll.
fn index_range(count: usize, per_view: f64, align: CarouselAlign) -> (usize, usize) {
    if count == 0 {
        return (0, 0);
    }
    let span = (count as f64 - per_view).max(0.0);
    let shift = align_shift(per_view, align);
    let first = (shift.round().max(0.0) as usize).min(count - 1);
    let last = (((span + shift).round().max(0.0)) as usize).min(count - 1);

    (first, last.max(first))
}

/// `index`'s place among the rest positions, and their count: the status and dots
/// count snaps, not slides ("1 of 4" for six three-up). Looping: one per slide.
fn snap_position(
    index: usize,
    count: usize,
    first: usize,
    last: usize,
    looping: bool,
) -> (usize, usize) {
    match looping {
        true => (index.min(count.saturating_sub(1)), count),
        false => (index.clamp(first, last) - first, last - first + 1),
    }
}

/// Whether `position` lies wholly outside the viewport at `rest`: it goes `inert`.
/// The gap counts as zero, which only keeps a peeking slide live.
fn outside_viewport(
    position: usize,
    rest: usize,
    strip: usize,
    per_view: f64,
    align: CarouselAlign,
) -> bool {
    const EPSILON: f64 = 1e-6;
    let range = (strip as f64 - per_view).max(0.0);
    let start = (rest as f64 - align_shift(per_view, align)).clamp(0.0, range);
    let slide = position as f64;

    slide + 1.0 <= start + EPSILON || slide >= start + per_view - EPSILON
}

/// The first and last strip positions at least half in the viewport while the
/// strip rests on `rest`: what the status range and a dot's number count.
fn showing(rest: usize, strip: usize, per_view: f64, align: CarouselAlign) -> (usize, usize) {
    const EPSILON: f64 = 1e-6;
    let range = (strip as f64 - per_view).max(0.0);
    let start = (rest as f64 - align_shift(per_view, align)).clamp(0.0, range);
    let end = start + per_view;
    let half = |k: &usize| end.min(*k as f64 + 1.0) - start.max(*k as f64) >= 0.5 - EPSILON;
    let first = (0..strip).find(half).unwrap_or(rest);
    let last = (0..strip).rev().find(half).unwrap_or(rest);
    (first, last)
}

/// The strip position that is slide `index`'s one live copy: the real slide if
/// it `shows`, else a clone that shows, else the real one (todo 544).
fn live_copy(index: usize, count: usize, clones: usize, shows: impl Fn(usize) -> bool) -> usize {
    let real = index + clones;
    let leading = (index + clones >= count).then(|| index + clones - count);
    let trailing = (index < clones).then(|| clones + count + index);
    [Some(real), leading, trailing]
        .into_iter()
        .flatten()
        .find(|&position| shows(position))
        .unwrap_or(real)
}

/// A dot's id, derived from the track's so two carousels do not collide.
fn indicator_id(track_id: &str, index: usize) -> String {
    format!("{track_id}-indicator-{index}")
}

/// Moves focus onto the dot the arrows just made current.
fn focus_indicator(root: ElementHandle, track_id: &str, index: usize) {
    let selector = format!("#{}", indicator_id(track_id, index));
    let _ = root.query_selector(&selector).and_then(|dot| dot.focus());
}

/// How far `scroll-snap-align` shifts the snapped index from the offset's, in
/// slides: none for `start`, half the extra slides for `center`, all for `end`.
fn align_shift(per_view: f64, align: CarouselAlign) -> f64 {
    match align {
        CarouselAlign::Start => 0.0,
        CarouselAlign::Center => (per_view - 1.0) / 2.0,
        CarouselAlign::End => per_view - 1.0,
    }
}

/// The snapped index for a scroll offset. Exact without measuring: dividing by
/// the max offset cancels slide width and gap.
fn index_at(offset: f64, max: f64, count: usize, per_view: f64, align: CarouselAlign) -> usize {
    let span = count as f64 - per_view;
    if max <= 0.0 || span <= 0.0 {
        return 0;
    }
    let raw = (offset / max * span + align_shift(per_view, align))
        .round()
        .max(0.0) as usize;
    let (first, last) = index_range(count, per_view, align);

    raw.clamp(first, last)
}

/// The inverse of [`index_at`], clamped to the scroll range.
fn offset_for(index: usize, max: f64, count: usize, per_view: f64, align: CarouselAlign) -> f64 {
    let span = count as f64 - per_view;
    if max <= 0.0 || span <= 0.0 {
        return 0.0;
    }
    ((index as f64 - align_shift(per_view, align)) / span * max).clamp(0.0, max)
}

/// The slides plus the clones at both ends (zero when not looping).
fn strip_count(count: usize, clones: usize) -> usize {
    count + 2 * clones
}

/// The real slide a strip position shows: leading clones are the tail, trailing the head.
fn real_for(raw: usize, count: usize, clones: usize) -> usize {
    if count == 0 {
        return 0;
    }
    (raw as isize - clones as isize).rem_euclid(count as isize) as usize
}

/// Whether a strip position is a cloned end: the seam has to be crossed.
fn is_clone(raw: usize, count: usize, clones: usize) -> bool {
    clones > 0 && (raw < clones || raw >= clones + count)
}

/// Whether a controlled scroll is instant: the first after mount, or one with a
/// slide swap. Only if it moves, or a raised `seam` never comes down.
fn instant_scroll(
    first_scroll: bool,
    swapped: bool,
    clones: usize,
    index: usize,
    from: usize,
    first: usize,
) -> bool {
    (first_scroll && (clones > 0 || index != first)) || (swapped && index != from)
}

/// From a caller swapping slides and `index` in one render (`Lightbox`): the move
/// is instant, or a smooth scroll fetches lazy pictures in passing (todo 323).
/// A count, so a swap survives a re-render before the effect.
#[derive(Clone, Default)]
pub(crate) struct CarouselJump(Rc<Cell<u64>>);

impl CarouselJump {
    /// Marks a swap, from the render that draws it.
    pub(crate) fn swapped(&self) {
        self.0.set(self.0.get() + 1);
    }

    fn count(&self) -> u64 {
        self.0.get()
    }
}

/// Drops the status and track tab stop when every slide fits (todo 564).
#[derive(Clone, Copy)]
pub(crate) struct CarouselQuietWhenFits;

/// Whether a swap arrived since last asked; reading consumes it, or every later
/// move would be instant too.
fn consume_swap(seen: &mut Option<u64>, swaps: Option<u64>) -> bool {
    swaps != std::mem::replace(seen, swaps)
}

/// Scrolls the track so `index` is the snapped slide, as a percent of the range.
fn scroll_to_index(
    track: ScrollAreaHandle,
    index: usize,
    count: usize,
    per_view: f64,
    orientation: Orientation,
    align: CarouselAlign,
) {
    let percent = offset_for(index, 100.0, count, per_view, align);
    match orientation {
        Orientation::Horizontal => track.scroll_to_percent(Some(percent), None),
        Orientation::Vertical => track.scroll_to_percent(None, Some(percent)),
    }
}

/// Per-instance only: no themed default, `auto` suits a horizontal strip.
const CAROUSEL_HEIGHT: CssVar = CssVar::new("--lsx-carousel-height");

fn carousel_variables(
    per_view: f64,
    gap: Option<Size>,
    height: Option<&ThemeAwareValue>,
) -> Variables {
    variables()
        .with(CAROUSEL_PER_VIEW.override_var(), Some(per_view.to_string()))
        .with(
            CAROUSEL_GAP.override_var(),
            gap.map(|gap| SizeCss::SPACING.value(gap)),
        )
        .with(
            CAROUSEL_HEIGHT,
            height.and_then(|height| height.resolve(Some(SizeCss::SPACING))),
        )
}

/// Everything a move needs, rebuilt each render and copied into handlers: a
/// `use_callback` would keep the first render's `count` and `per_view`.
#[derive(Clone, Copy, PartialEq)]
struct Nav {
    current: Signal<usize>,
    settled: Signal<usize>,
    seam: Signal<bool>,
    track: ScrollAreaHandle,
    count: usize,
    per_view: f64,
    orientation: Orientation,
    align: CarouselAlign,
    first: usize,
    last: usize,
    onindexchange: Option<EventHandler<usize>>,
    /// Slides cloned onto each end of a looping strip; zero otherwise.
    clones: usize,
}

impl Nav {
    fn strip_count(self) -> usize {
        strip_count(self.count, self.clones)
    }

    /// A real slide's position in the strip.
    fn raw_for(self, index: usize) -> usize {
        index + self.clones
    }

    fn real_for(self, raw: usize) -> usize {
        real_for(raw, self.count, self.clones)
    }

    fn is_clone(self, raw: usize) -> bool {
        is_clone(raw, self.count, self.clones)
    }

    /// Clamps to the real slides when looping, else to where scrolling stops.
    fn clamp_index(self, index: usize) -> usize {
        match self.clones > 0 {
            true => index.min(self.count.saturating_sub(1)),
            false => index.clamp(self.first, self.last),
        }
    }

    /// Pulls both indices into the reachable window. No scroll (the reading was
    /// wrong, not the position) and no callback.
    fn pull_into_range(mut self) {
        let held = *self.current.peek();
        let clamped = self.clamp_index(held);
        if clamped != held {
            self.current.set(clamped);
            self.settled.set(clamped);
        }
    }

    fn go_to(mut self, index: usize) {
        let index = self.clamp_index(index);
        // Against `settled`, the last index the caller was told.
        let changed = index != *self.settled.peek();
        self.current.set(index);
        self.settled.set(index);
        self.scroll_to_raw(self.raw_for(index));
        if changed && let Some(handler) = &self.onindexchange {
            handler.call(index);
        }
    }

    fn scroll_to_raw(self, raw: usize) {
        scroll_to_index(
            self.track,
            raw,
            self.strip_count(),
            self.per_view,
            self.orientation,
            self.align,
        );
    }

    /// Settled on a clone: raises `seam` (smooth scrolling off) so the jump to the
    /// real slide, in a later effect, does not visibly rewind.
    fn cross_seam(mut self) {
        self.seam.set(true);
    }

    /// The first and last real slide at least half showing while the strip
    /// rests on `index`. A looping strip can wrap: `from` after `to`.
    fn showing(self, index: usize) -> (usize, usize) {
        let (first, last) = showing(
            self.raw_for(index),
            self.strip_count(),
            self.per_view,
            self.align,
        );
        (self.real_for(first), self.real_for(last))
    }

    /// The strip position a scroll report (in percent) names.
    fn raw_at(self, x: f64, y: f64) -> usize {
        let percent = match self.orientation {
            Orientation::Horizontal => x,
            Orientation::Vertical => y,
        };
        index_at(
            percent,
            100.0,
            self.strip_count(),
            self.per_view,
            self.align,
        )
    }
}

base_props! {
    pub struct CarouselProps {
        /// The slides, in order.
        #[props(default)]
        slides: Vec<Element>,
        /// Each slide's accessible name; defaults to `"{n} of {m}"`.
        #[props(default)]
        slide_label: Option<Callback<usize, String>>,
        /// The current slide, when controlled.
        #[props(default)]
        index: Option<usize>,
        /// Fired once a scroll settles, and on every control, key and indicator.
        #[props(default)]
        onindexchange: Option<EventHandler<usize>>,
        /// Slides visible at once. Fractional peeks the next one.
        #[props(default, into)]
        per_view: Input<f64>,
        #[props(default, into)]
        gap: Input<Size>,
        #[props(default, into)]
        align: Input<CarouselAlign>,
        /// `"horizontal"` by default, unlike `Orientation`'s own default.
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// Required for a vertical carousel.
        #[props(default, into)]
        height: Input<ThemeAwareValue>,
        #[props(default)]
        controls: Option<bool>,
        #[props(default)]
        indicators: Option<bool>,
        /// Names the region; unset warns.
        #[props(default, into)]
        aria_label: Option<String>,
        /// Mouse drag-to-scroll over the track; touch already swipes.
        #[props(default)]
        draggable: bool,
        /// Advances on a timer, with a pause control (WCAG 2.2.2).
        #[props(default)]
        autoplay: bool,
        /// Milliseconds between advances.
        #[props(default)]
        autoplay_delay: Option<u32>,
        /// Wraps at both ends through cloned slides.
        #[props(default)]
        r#loop: bool,
    }
}

/// A scroll-snap strip that knows which slide it is on.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Carousel, Image};
/// # struct Photo { url: String, alt: String }
/// # fn app() -> Element {
/// # let photos: Vec<Photo> = Vec::new();
/// rsx! {
///     Carousel {
///         aria_label: "Product photos",
///         per_view: 3.0,
///         indicators: true,
///         slides: photos.iter().map(|p| rsx! { Image { src: "{p.url}", alt: "{p.alt}" } }).collect(),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/carousel>
#[component]
pub fn Carousel(props: CarouselProps) -> Element {
    let theme = use_theme();
    let labels = &use_localization().carousel;
    let track = use_scroll_area();
    // The indicators sit outside the track, so roving focus finds them from the root.
    let root_handle = use_element();
    let track_id = use_id();
    // The status region describes the track, so the tab stop says where it is.
    let status_id = use_id();

    let jump = try_use_context::<CarouselJump>();
    let quiet_when_fits = try_use_context::<CarouselQuietWhenFits>().is_some();

    let count = props.slides.len();
    let per_view = props.per_view.copied_or(theme.carousel.per_view).max(0.1);
    let quiet = quiet_when_fits && count as f64 <= per_view;
    // `Orientation` defaults to vertical; a carousel does not.
    let orientation = props.orientation.copied_or(Orientation::Horizontal);
    let align = props.align.copied_or(theme.carousel.align);
    // No slides: no chrome, no landmark, no tab stop. The root keeps its box.
    let empty = count == 0;
    let controls = !empty && props.controls.unwrap_or(theme.carousel.controls);
    let indicators = !empty && props.indicators.unwrap_or(theme.carousel.indicators);
    let (first, last) = index_range(count, per_view, align);

    // Read in render, so a swap re-runs the controlled-index effect.
    let swaps = jump.as_ref().map(CarouselJump::count);

    let setup = CarouselSetup {
        track,
        count,
        per_view,
        orientation,
        align,
        first,
        last,
        onindexchange: props.onindexchange,
        controlled: props.index,
        swaps,
        r#loop: props.r#loop,
        autoplay: props.autoplay,
        autoplay_delay: props
            .autoplay_delay
            .unwrap_or(theme.carousel.autoplay_delay),
        named: props.aria_label.is_some(),
        quiet,
    };
    let state = use_carousel_state(setup);
    let drag = use_carousel_drag(setup, state, props.draggable);

    let view = CarouselView {
        setup,
        state,
        labels,
        track_id,
        status_id,
        root: root_handle,
    };

    // A generic name beats none; the warning still asks for a better one.
    let aria_label = props
        .aria_label
        .clone()
        .unwrap_or_else(|| labels.label.to_string());

    let root_states: Input<States> = props
        .states
        .clone()
        .unwrap_or_default()
        .with(orientation.state_name(), true)
        .into();
    let variables: Input<Variables> =
        carousel_variables(per_view, props.gap.as_ref().copied(), props.height.as_ref()).into();

    let focus = use_focus_within(
        move || vec![root_handle.mounted()],
        move |change| {
            let mut focused = state.focused;
            focused.set(change.within)
        },
    );

    let root = use_box()
        .framework_sx(&CAROUSEL_ROOT_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&root_states)
        .variables(&variables)
        .prepare()
        .element(&root_handle)
        .attr("role", (!empty).then_some("region"))
        .attr("aria-roledescription", (!empty).then_some("carousel"))
        .attr("aria-label", (!empty).then(|| aria_label.clone()))
        .event("onmouseenter", move |_: Event<MouseData>| {
            let mut hovered = state.hovered;
            hovered.set(true)
        })
        .event("onmouseleave", move |_: Event<MouseData>| {
            let mut hovered = state.hovered;
            hovered.set(false)
        })
        .event("onfocusin", focus.focusin(0))
        .event("onfocusout", focus.focusout(0));

    let body = carousel_slides(view, &props.slides, props.slide_label);

    root.render(
        HtmlTag::Section,
        props.attributes,
        rsx! {
            if !empty && !quiet {
                CarouselStatus { view }
            }
            // First in Tab order, as APG's rotation control (todo 548).
            if props.autoplay && !empty {
                {carousel_pause_button(view, controls)}
            }
            Box { framework_sx: &CAROUSEL_VIEWPORT_SX,
                {carousel_track(view, aria_label, props.draggable, drag, body)}
                if controls {
                    CarouselControls { view }
                }
            }
            if indicators {
                CarouselIndicators { view }
            }
        },
    )
}

/// What `Carousel`'s state hook reads, as one reactive effect argument.
#[derive(Clone, Copy, PartialEq)]
struct CarouselSetup {
    track: ScrollAreaHandle,
    count: usize,
    per_view: f64,
    orientation: Orientation,
    align: CarouselAlign,
    first: usize,
    last: usize,
    onindexchange: Option<EventHandler<usize>>,
    /// The caller's `index`, when one drives the carousel from outside.
    controlled: Option<usize>,
    /// `CarouselJump`'s swap counter, or `None` without a provider.
    swaps: Option<u64>,
    r#loop: bool,
    autoplay: bool,
    autoplay_delay: u32,
    /// Whether the caller named the region, for the name warning.
    named: bool,
    /// Every slide fits under [`CarouselQuietWhenFits`]: no status, no track tab stop.
    quiet: bool,
}

/// `Carousel`'s live state: `nav`, what pauses autoplay, and the drag.
#[derive(Clone, Copy, PartialEq)]
struct CarouselState {
    nav: Nav,
    paused: Signal<bool>,
    hovered: Signal<bool>,
    focused: Signal<bool>,
    /// A pointer is down on the pause control, so the focus it brings is no entry.
    pressing: Signal<bool>,
    dragging: Signal<bool>,
    drag_origin: Signal<f64>,
    /// The controlled index the effect last applied. Until it runs, a new one
    /// stands in for `settled` when the slides go `inert` - see `carousel_slides`.
    applied: Signal<Option<usize>>,
    /// Autoplay is on and nothing is holding it back.
    running: bool,
}

/// `template` with `{n}` one-based (read aloud) and `{m}` the count.
pub(crate) fn numbered(template: &str, index: usize, count: usize) -> String {
    fill(template, &[("n", &(index + 1)), ("m", &count)])
}

/// Everything the carousel's chrome needs: setup, state, strings and ids.
#[derive(Clone, Copy, PartialEq)]
struct CarouselView {
    setup: CarouselSetup,
    state: CarouselState,
    labels: &'static CarouselLabels,
    track_id: Signal<String>,
    status_id: Signal<String>,
    root: ElementHandle,
}

/// The carousel's state and every effect that moves the strip.
fn use_carousel_state(setup: CarouselSetup) -> CarouselState {
    let CarouselSetup {
        track,
        count,
        per_view,
        orientation,
        align,
        first,
        last,
        onindexchange,
        controlled,
        swaps,
        autoplay,
        autoplay_delay: delay,
        named,
        ..
    } = setup;

    // `current` follows the scroll live; `settled` moves at rest, for the live region.
    let mut current = use_signal(|| controlled.unwrap_or(0));
    let mut settled = use_signal(|| controlled.unwrap_or(0));

    // Reduced motion starts paused: the pause control starts it (todo 551).
    let paused = use_signal(prefers_reduced_motion);
    let hovered = use_signal(|| false);
    let focused = use_signal(|| false);
    let mut subscription = use_signal(|| None::<Box<dyn TimerSubscription>>);
    let dragging = use_signal(|| false);
    let mut seam = use_signal(|| false);
    let drag_origin = use_signal(|| 0.0_f64);

    use_name_warning(
        named,
        "Carousel: no `aria_label`, falling back to the theme's. A region needs a name of its own to be told apart in a landmark list.",
    );

    // One clone per visible slide at each end: a full viewport past either edge.
    let clones = match setup.r#loop && count > 1 {
        true => (per_view.ceil() as usize).min(count),
        false => 0,
    };
    let nav = Nav {
        current,
        settled,
        seam,
        track,
        count,
        per_view,
        orientation,
        align,
        first,
        last,
        onindexchange,
        clones,
    };

    // An effect, not a seed: the window moves with `align`, `per_view` and `count`,
    // and at rest no scroll settles to correct the index.
    use_effect(use_reactive!(|nav| {
        nav.pull_into_range();
    }));

    // The seam jump, in an effect: `seam`'s instant scrolling reaches the DOM a
    // render later, and a jump from the handler rewound the whole strip.
    use_effect(use_reactive!(|nav| {
        if seam() {
            nav.scroll_to_raw(nav.raw_for(*nav.current.peek()));
        }
    }));

    // A looping strip opens on a clone: the first placement raises `seam` so it is
    // instant. Placed after the seam effect, which would scroll before `seam` lands.
    let mut placed = use_signal(|| false);
    use_effect(use_reactive!(|nav, clones| {
        if clones > 0 {
            match std::mem::replace(&mut *placed.write(), true) {
                false => seam.set(true),
                true => nav.scroll_to_raw(nav.raw_for(*nav.current.peek())),
            }
        }
    }));

    // A controlled `index`, with the whole tuple reactive.
    let mut mounting = use_signal(|| true);
    let mut applied = use_signal(|| None::<usize>);
    let mut swaps_seen = use_signal(|| swaps);
    use_effect(use_reactive!(|controlled, nav, swaps| {
        let Some(index) = controlled else {
            return;
        };
        applied.set(Some(index));
        // Through `nav`, so the clamp matches `go_to`'s.
        let asked = index;
        let index = nav.clamp_index(asked);
        // Where the strip is before this move: `current` follows the scroll.
        let from = *current.peek();
        if index != from {
            current.set(index);
            settled.set(index);
        }
        // A smooth first scroll swept past every earlier slide (~1.5s to 6 of 6):
        // `seam` makes it instant, via the seam effect.
        let first_scroll = std::mem::replace(&mut *mounting.write(), false);
        let swapped = consume_swap(&mut swaps_seen.write(), swaps);
        match instant_scroll(first_scroll, swapped, nav.clones, index, from, nav.first) {
            true => seam.set(true),
            false => nav.scroll_to_raw(nav.raw_for(index)),
        }
        // An unshowable index: report the clamped one, or caller and clamp disagree.
        if index != asked
            && let Some(handler) = &nav.onindexchange
        {
            handler.call(index);
        }
    }));

    // Focus entering stops rotation until Play (todo 548). A move within, or a
    // press on the pause control, is no entry.
    let mut was_focused = use_signal(|| false);
    let mut pressing = use_signal(|| false);
    use_effect(use_reactive!(|autoplay| {
        let within = focused();
        let entered = within && !*was_focused.peek();
        was_focused.set(within);
        if !entered {
            return;
        }
        let pressed = std::mem::replace(&mut *pressing.write(), false);
        if autoplay && !pressed && !*paused.peek() {
            let mut paused = paused;
            paused.set(true);
        }
    }));

    // Hover only pauses: leaving resumes.
    let running = autoplay && !paused() && !hovered() && count > 1;
    // The timer callback has no scope and a move spawns, so a tick only counts;
    // the effect below moves. `go_to` there panicked in `spawn` on the web.
    let ticks = use_signal(|| 0usize);
    let mut advanced = use_signal(|| 0usize);
    use_effect(use_reactive!(|running, delay| {
        if !running {
            subscription.set(None);
            return;
        }
        let Some(timer) = timer() else {
            subscription.set(None);
            return;
        };
        // Reduced motion switched on while rotating pauses it; a Play pressed
        // under it already stays pressed (todo 567).
        let reduced = std::cell::Cell::new(prefers_reduced_motion());
        let handle = timer.every(
            Duration::from_millis(delay.max(1) as u64),
            Box::new(move || {
                let now = prefers_reduced_motion();
                if now && !reduced.replace(now) {
                    let mut paused = paused;
                    paused.set(true);
                    return;
                }
                reduced.set(now);
                let mut ticks = ticks;
                ticks += 1;
            }),
        );
        subscription.set(Some(handle));
    }));
    use_effect(use_reactive!(|nav, last| {
        let tick = ticks();
        if tick == *advanced.peek() {
            return;
        }
        advanced.set(tick);
        let end = match nav.clones > 0 {
            true => nav.count.saturating_sub(1),
            false => last,
        };
        let next = match *nav.current.peek() >= end {
            // Autoplay wraps; the controls deliberately do not.
            true => 0,
            false => *nav.current.peek() + 1,
        };
        nav.go_to(next);
    }));

    CarouselState {
        nav,
        paused,
        hovered,
        focused,
        pressing,
        dragging,
        drag_origin,
        applied,
        running,
    }
}

/// Mouse drag-to-scroll over the track. No `drag_handle_sx()`: its
/// `touch-action: none` would kill the native touch swipe.
fn use_carousel_drag(setup: CarouselSetup, state: CarouselState, draggable: bool) -> Drag {
    let track = setup.track;
    let orientation = setup.orientation;
    let nav = state.nav;
    let mut dragging = state.dragging;
    let mut drag_origin = state.drag_origin;
    // Under RTL a drag to the right heads for the end, as in `Scroller`.
    let mut rtl = use_hook(|| CopyValue::new(false));

    use_drag(DragOptions {
        capture: track.element,
        onstart: Callback::new(move |start: DragStart| {
            if !draggable {
                start.cancel.call(());
                return;
            }
            dragging.set(true);
            let offset = track.element.scroll_offset();
            rtl.set(track.element.is_rtl());
            spawn(async move {
                if let Ok((x, y)) = offset.await {
                    drag_origin.set(match orientation {
                        Orientation::Horizontal => inline_x(x),
                        Orientation::Vertical => y,
                    });
                }
            });
        }),
        onmove: Callback::new(move |moved: DragMove| {
            let delta = moved.delta();
            let target = match orientation {
                Orientation::Horizontal => drag_origin() - physical_x(delta.x, rtl()),
                Orientation::Vertical => drag_origin() - delta.y,
            }
            .max(0.0);
            match orientation {
                Orientation::Horizontal => track.scroll_to(target, 0.0),
                Orientation::Vertical => track.scroll_to(0.0, target),
            }
        }),
        // Releasing hands the strip back to the browser, which snaps and
        // fires `onscrollend`. Blitz does neither: the nearest slide is gone to.
        onend: Callback::new(move |()| {
            dragging.set(false);
            if !snaps_scroll() {
                settle_nearest(nav);
            }
        }),
    })
}

/// Goes to the slide nearest the track's scroll offset.
fn settle_nearest(nav: Nav) {
    let element = nav.track.element;
    let (offset, content, view) = (
        element.scroll_offset(),
        element.scroll_size(),
        element.dimensions(),
    );
    spawn(async move {
        let (Ok((x, y)), Ok(content), Ok(view)) = (offset.await, content.await, view.await) else {
            return;
        };
        let percent = |at: f64, range: f64| match range > 0.0 {
            true => at / range * 100.0,
            false => 0.0,
        };
        let raw = nav.raw_at(
            percent(inline_x(x), content.width - view.width),
            percent(y, content.height - view.height),
        );
        // A move reads the track, which a task finds borrowed natively.
        when_free(move || nav.go_to(nav.real_for(raw)));
    });
}

/// The strip: the cloned tail, the slides, the cloned head (clones only when looping).
fn carousel_slides(
    view: CarouselView,
    slides: &[Element],
    slide_label: Option<Callback<usize, String>>,
) -> Vec<Element> {
    let CarouselView {
        setup,
        state,
        labels,
        ..
    } = view;
    let nav = state.nav;
    let count = setup.count;
    let clones = nav.clones;

    let label_for = move |index: usize| match &slide_label {
        Some(label) => label.call(index),
        None => numbered(labels.slide, index, count),
    };

    let mut strip: Vec<(usize, Element, bool)> = Vec::with_capacity(nav.strip_count());
    for offset in 0..clones {
        let real = count + offset - clones;
        strip.push((real, slides[real].clone(), true));
    }
    for (index, slide) in slides.iter().enumerate() {
        strip.push((index, slide.clone(), false));
    }
    for (index, slide) in slides.iter().enumerate().take(clones) {
        strip.push((index, slide.clone(), true));
    }

    strip
        .into_iter()
        .enumerate()
        .map(|(position, (index, slide, is_clone))| {
            let label = label_for(index);
            rsx! {
                CarouselSlide {
                    key: "{position}",
                    nav,
                    controlled: setup.controlled,
                    applied: state.applied,
                    position,
                    index,
                    is_clone,
                    label,
                    slide,
                }
            }
        })
        .collect()
}

/// One slide group, its own scope so a move redraws only the slides that flip.
#[component]
fn CarouselSlide(
    slide: Element,
    nav: Nav,
    controlled: Option<usize>,
    applied: Signal<Option<usize>>,
    position: usize,
    index: usize,
    is_clone: bool,
    label: String,
) -> Element {
    let flags =
        use_memo(use_reactive!(|nav,
                                controlled,
                                position,
                                index,
                                is_clone| {
            // `settled`, not `current`, or slides flip every frame. An unapplied
            // controlled index counts already: `Lightbox`'s focus effect may run first.
            let rest = match controlled {
                Some(held) if Some(held) != *applied.peek() => nav.clamp_index(held),
                _ => (nav.settled)(),
            };
            let shows = |position| {
                !outside_viewport(
                    position,
                    nav.raw_for(rest),
                    nav.strip_count(),
                    nav.per_view,
                    nav.align,
                )
            };
            // One live copy per slide: a showing clone stands in for its twin.
            let live = live_copy(index, nav.count, nav.clones, shows) == position;
            // Offscreen or a copy: out of Tab and reading order.
            let hidden = !live || !shows(position);
            (live, hidden, !is_clone && index == (nav.current)())
        }));
    let (live, hidden, current) = flags();
    let track = nav.track;
    // No `current` token: callers read `data-current`.
    let slide_states: Input<States> = states().with(nav.align.state_name(), true).into();

    rsx! {
        Box {
            framework_sx: &CAROUSEL_SLIDE_SX,
            states: slide_states,
            role: live.then_some("group"),
            aria_roledescription: live.then_some("slide"),
            aria_label: live.then_some(label),
            aria_hidden: (!live).then(|| "true".to_string()),
            inert: hidden.then_some(true),
            // Focus in a slide that just went `inert` would drop to `<body>`:
            // the track takes it instead.
            onfocusout: move |_| {
                if hidden {
                    let _ = track.element.focus();
                }
            },
            "data-current": current.then_some("true"),
            {slide}
        }
    }
}

/// The scroll container and tab stop: the strip, scroll listeners, arrow keys and drag.
fn carousel_track(
    view: CarouselView,
    aria_label: String,
    draggable: bool,
    drag: Drag,
    body: Vec<Element>,
) -> Element {
    let CarouselView {
        setup,
        state,
        track_id,
        status_id,
        ..
    } = view;
    let nav = state.nav;
    let orientation = setup.orientation;
    let empty = setup.count == 0;
    let dragging = state.dragging;
    let mut current = nav.current;
    let mut settled = nav.settled;
    let mut seam = nav.seam;

    let mut onscroll = move |x: f64, y: f64| {
        let next = nav.real_for(nav.raw_at(x, y));
        if next != *current.peek() {
            current.set(next);
        }
    };

    // Only a settled scroll moves `settled`, which the live region and `onindexchange` read.
    let mut onscrollend = move |x: f64, y: f64| {
        // A drag's per-move scrolls end too, but are no settle: they jumped the seam.
        if *dragging.peek() {
            return;
        }
        let raw = nav.raw_at(x, y);
        // No scroll snap natively: go to the nearest slide, as a released drag does.
        if !snaps_scroll() {
            let real = nav.real_for(raw);
            when_free(move || nav.go_to(real));
            return;
        }
        let next = nav.real_for(raw);
        current.set(next);
        if next != *settled.peek() {
            settled.set(next);
            if let Some(handler) = &nav.onindexchange {
                handler.call(next);
            }
        }
        // Settled on a clone: jump to the real slide; that landing crosses nothing.
        match nav.is_clone(raw) {
            true => nav.cross_seam(),
            false => {
                if *seam.peek() {
                    seam.set(false);
                }
            }
        }
    };

    // On the track, so the sibling controls' keys never reach it. The guards skip
    // keys slide content needs: taken presses, text entry, native arrow inputs.
    let onkeydown = move |event: Event<KeyboardData>| {
        if key_taken(&event)
            || typing_target(&event)
            || arrow_target(&event)
            || has_shortcut_modifier(&event)
        {
            return;
        }
        let Some(target) = track_key_target(logical_key(&event), orientation, nav, current())
        else {
            return;
        };
        // Or the native scroll runs too and lands between two snap points.
        event.prevent_default();
        nav.go_to(target);
    };

    let track_states: Input<States> = states()
        .with(orientation.state_name(), true)
        .with("dragging", dragging())
        // The seam jump has to be instant, or the strip visibly rewinds.
        .with("seam", seam())
        .into();

    rsx! {
        ScrollArea {
            handle: setup.track,
            scrollbars: match orientation {
                Orientation::Horizontal => "horizontal",
                Orientation::Vertical => "vertical",
            },
            scrollbar_visibility: "hidden",
            // The tab stop, unlike `ScrollArea`'s default; not when empty.
            focusable: !empty && !setup.quiet,
            framework_sx: ScrollAreaBase(&CAROUSEL_TRACK_SX),
            states: track_states,
            id: track_id(),
            // Named itself: landing inside does not reliably re-announce the region.
            role: (!empty).then_some("group"),
            aria_label: (!empty).then(|| aria_label.clone()),
            aria_describedby: (!empty && !setup.quiet).then_some(status_id()),
            onscroll: move |event: ScrollPositionEvent| match event {
                ScrollPositionEvent::Start(x, y) | ScrollPositionEvent::Change(x, y) => {
                    onscroll(x, y)
                }
                ScrollPositionEvent::End(x, y) => onscrollend(x, y),
            },
            onkeydown,
            // Mouse only: a touch is already scrolling the track natively, and
            // dragging it too would move it twice.
            onpointerdown: move |event: Event<PointerData>| {
                if draggable && event.data().pointer_type() == "mouse" {
                    drag.onpointerdown.call(event);
                }
            },
            onpointermove: drag.onpointermove,
            onpointerup: drag.onpointerup,
            onpointercancel: drag.onpointercancel,
            {body.into_iter()}
        }
    }
}

/// Where an arrow, `Home` or `End` on the track goes; `None` for a key the
/// carousel does not act on. A looping strip wraps, a plain one clamps to the
/// reachable window.
fn track_key_target(key: Key, orientation: Orientation, nav: Nav, current: usize) -> Option<usize> {
    let (previous, next) = match orientation {
        Orientation::Horizontal => (Key::ArrowLeft, Key::ArrowRight),
        Orientation::Vertical => (Key::ArrowUp, Key::ArrowDown),
    };
    let wraps = nav.clones > 0;
    let count = nav.count;
    Some(match key {
        key if key == previous => match (wraps, current) {
            (true, 0) => count.saturating_sub(1),
            (_, index) => index.saturating_sub(1).max(nav.first),
        },
        key if key == next => match wraps && current >= count.saturating_sub(1) {
            true => 0,
            false => (current + 1).min(nav.last),
        },
        Key::Home => match wraps {
            true => 0,
            false => nav.first,
        },
        Key::End => match wraps {
            true => count.saturating_sub(1),
            false => nav.last,
        },
        _ => return None,
    })
}

/// The previous/next pair, over the viewport. A looping carousel has no ends,
/// so its controls never disable. Its own scope, as the status and the dots:
/// only these read `current`/`settled`, so a move skips the track.
#[component]
fn CarouselControls(view: CarouselView) -> Element {
    let CarouselView { setup, state, .. } = view;
    let current = state.nav.current;
    let looping = state.nav.clones > 0;
    let (first, last) = (setup.first, setup.last);
    // Memos the strip hands down unread, so a move redraws only the button
    // whose end flipped.
    let at_start = use_memo(use_reactive!(
        |looping, first| !looping && current() <= first
    ));
    let at_end = use_memo(use_reactive!(|looping, last| !looping && current() >= last));
    let strip_states: Input<States> = states().with(setup.orientation.state_name(), true).into();

    rsx! {
        Box { framework_sx: &CAROUSEL_CONTROLS_SX, states: strip_states,
            CarouselControl { view, forward: false, disabled: at_start }
            CarouselControl { view, forward: true, disabled: at_end }
        }
    }
}

/// One control. Focusable when disabled at its end, so it keeps its tab stop.
#[component]
fn CarouselControl(view: CarouselView, forward: bool, disabled: Memo<bool>) -> Element {
    let CarouselView {
        setup,
        state,
        labels,
        track_id,
        ..
    } = view;
    let nav = state.nav;
    let count = setup.count;
    let orientation = setup.orientation;
    let current = nav.current;
    let looping = nav.clones > 0;
    let control_states: Input<States> = states().with(orientation.state_name(), true).into();

    rsx! {
        ActionIcon {
            sx: &CAROUSEL_CONTROL_SX,
            states: control_states,
            aria_controls: track_id(),
            disabled: disabled(),
            focusable_when_disabled: true,
            aria_label: if forward { labels.next } else { labels.previous },
            onclick: move |_| {
                let index = *current.peek();
                match (forward, looping) {
                    (false, true) if index == 0 => nav.go_to(count.saturating_sub(1)),
                    (true, true) if index >= count.saturating_sub(1) => nav.go_to(0),
                    (false, _) => nav.go_to(index.saturating_sub(1)),
                    (true, _) => nav.go_to(index + 1),
                }
            },
            {
                let (slot, icon) = match (orientation, forward) {
                    (Orientation::Horizontal, false) => {
                        (IconSlot::ChevronLeft, lucide::chevron_left::outlined)
                    }
                    (Orientation::Vertical, false) => {
                        (IconSlot::ChevronUp, lucide::chevron_up::outlined)
                    }
                    (Orientation::Horizontal, true) => {
                        (IconSlot::ChevronRight, lucide::chevron_right::outlined)
                    }
                    (Orientation::Vertical, true) => {
                        (IconSlot::ChevronDown, lucide::chevron_down::outlined)
                    }
                };
                rsx! { Glyph { slot, icon } }
            }
        }
    }
}

/// The live region: where the strip settled, out of the places it can rest.
#[component]
fn CarouselStatus(view: CarouselView) -> Element {
    let CarouselView {
        setup,
        state,
        labels,
        status_id,
        ..
    } = view;
    let nav = state.nav;
    let settled = nav.settled;
    let status = match nav.per_view > 1.0 {
        // Several up, the slides showing are named, not the resting position (todo 550).
        true => {
            let (from, to) = nav.showing(settled());
            match from == to {
                true => numbered(labels.status, from, setup.count),
                false => fill(
                    labels.status_range,
                    &[
                        ("from", &(from + 1)),
                        ("to", &(to + 1)),
                        ("n", &setup.count),
                    ],
                ),
            }
        }
        false => {
            let (position, positions) = snap_position(
                settled(),
                setup.count,
                setup.first,
                setup.last,
                nav.clones > 0,
            );
            numbered(labels.status, position, positions)
        }
    };

    rsx! {
        VisuallyHidden {
            id: status_id(),
            role: "status",
            // Off while autoplay rotates, `polite` once it stops (WCAG 2.2.2).
            aria_live: if state.running { "off" } else { "polite" },
            aria_atomic: "true",
            "{status}"
        }
    }
}

/// The autoplay toggle. Its name stays the same whether the slideshow runs or
/// not: `aria-pressed` carries the state.
fn carousel_pause_button(view: CarouselView, controls: bool) -> Element {
    let mut paused = view.state.paused;
    let labels = view.labels;
    let (focused, mut pressing) = (view.state.focused, view.state.pressing);
    let beside_next = controls && view.setup.orientation == Orientation::Horizontal;
    let pause_states: Input<States> = states().with("beside-next", beside_next).into();

    rsx! {
        Box {
            component: "button",
            r#type: "button",
            framework_sx: &CAROUSEL_PAUSE_SX,
            states: pause_states,
            aria_label: labels.pause,
            aria_pressed: paused().to_string(),
            // The focus this press brings in would stop rotation before the
            // click toggles it back on.
            onpointerdown: move |_| {
                if !*focused.peek() {
                    pressing.set(true);
                }
            },
            onclick: move |_| paused.toggle(),
            if paused() {
                Glyph { slot: IconSlot::Play, icon: lucide::play::outlined }
            } else {
                Glyph { slot: IconSlot::Pause, icon: lucide::pause::outlined }
            }
        }
    }
}

/// The dot strip: one dot per reachable position, which under a centred or end
/// alignment does not start at zero. A looping carousel is one dot per real
/// slide instead.
#[component]
fn CarouselIndicators(view: CarouselView) -> Element {
    let CarouselView {
        setup,
        state,
        labels,
        track_id,
        root,
        ..
    } = view;
    let nav = state.nav;
    let count = setup.count;
    let orientation = setup.orientation;
    let current = nav.current;
    let looping = nav.clones > 0;
    let (low, high) = match looping {
        true => (0, count.saturating_sub(1)),
        false => (setup.first, setup.last),
    };
    let strip_states: Input<States> = states().with(orientation.state_name(), true).into();

    rsx! {
        // A named group, so the dots read as one set (todo 549).
        Box {
            framework_sx: &CAROUSEL_INDICATORS_SX,
            states: strip_states,
            role: "group",
            aria_label: labels.indicators,
            for index in low..=high {
                Box {
                    key: "{index}",
                    component: "button",
                    r#type: "button",
                    framework_sx: &CAROUSEL_INDICATOR_SX,
                    states: states()
                        .with(orientation.state_name(), true)
                        .with("current", index == current()),
                    // By the first slide it shows; a looping dot by its own (todo 550).
                    aria_label: numbered(
                        labels.indicator,
                        if looping { index } else { nav.showing(index).0 },
                        count,
                    ),
                    aria_current: (index == current()).then(|| "true".to_string()),
                    // Roving: one tab stop; arrows move the focus with the slide.
                    id: indicator_id(&track_id(), index),
                    tabindex: if index == current() { "0" } else { "-1" },
                    onclick: move |_| nav.go_to(index),
                    onkeydown: move |event: Event<KeyboardData>| {
                        if has_shortcut_modifier(&event) {
                            return;
                        }
                        let Some(target) = indicator_key_target(logical_key(&event), index, low, high, looping)
                        else {
                            return;
                        };
                        event.prevent_default();
                        nav.go_to(target);
                        focus_indicator(root, &track_id(), target);
                    },
                }
            }
        }
    }
}

/// Where an arrow, `Home` or `End` on a dot goes. Wraps only when the carousel
/// does, so dots and track agree.
fn indicator_key_target(
    key: Key,
    index: usize,
    low: usize,
    high: usize,
    looping: bool,
) -> Option<usize> {
    Some(match key {
        Key::ArrowRight | Key::ArrowDown => match index >= high {
            true if looping => low,
            true => high,
            false => index + 1,
        },
        Key::ArrowLeft | Key::ArrowUp => match index <= low {
            true if looping => high,
            true => low,
            false => index - 1,
        },
        Key::Home => low,
        Key::End => high,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;

    /// Six slides three-up run out of scroll at index 3, not 5 - the last
    /// three share the viewport.
    #[test]
    fn the_last_index_is_where_the_scrolling_stops_not_the_last_slide() {
        let start = CarouselAlign::Start;

        assert_eq!(index_range(6, 1.0, start), (0, 5));
        assert_eq!(index_range(6, 3.0, start), (0, 3));
        assert_eq!(index_range(6, 2.5, start), (0, 4));
        assert_eq!(index_range(3, 3.0, start), (0, 0));
        assert_eq!(index_range(2, 5.0, start), (0, 0));
        assert_eq!(index_range(0, 1.0, start), (0, 0));
    }

    /// The status and the dots count resting positions, not slides.
    #[test]
    fn the_status_counts_where_the_strip_can_rest() {
        let (first, last) = index_range(6, 3.0, CarouselAlign::Center);
        assert_eq!(snap_position(1, 6, first, last, false), (0, 4));
        assert_eq!(snap_position(4, 6, first, last, false), (3, 4));

        // All six fit: one resting place, whichever slide was asked for.
        let (first, last) = index_range(6, 6.0, CarouselAlign::Center);
        assert_eq!(snap_position(5, 6, first, last, false), (0, 1));
        assert_eq!(snap_position(0, 6, first, last, false), (0, 1));

        // One-up and looping: a position per slide, as before.
        assert_eq!(snap_position(2, 6, 0, 5, false), (2, 6));
        assert_eq!(snap_position(5, 6, 0, 0, true), (5, 6));
    }

    /// The alignment slides the reachable window: the same strip reaches 0-3
    /// start-aligned, 1-4 centred and 2-5 end-aligned, because the browser
    /// clamps the scroll at both ends.
    #[test]
    fn the_alignment_moves_which_indices_are_reachable_at_all() {
        assert_eq!(index_range(6, 3.0, CarouselAlign::Start), (0, 3));
        assert_eq!(index_range(6, 3.0, CarouselAlign::Center), (1, 4));
        assert_eq!(index_range(6, 3.0, CarouselAlign::End), (2, 5));
    }

    /// One-up is why the missing alignment went unnoticed for a whole round:
    /// there all three coincide.
    #[test]
    fn at_one_up_every_alignment_is_the_same_mapping() {
        for align in [
            CarouselAlign::Start,
            CarouselAlign::Center,
            CarouselAlign::End,
        ] {
            assert_eq!(index_range(6, 1.0, align), (0, 5));
            assert_eq!(index_at(240.0, 600.0, 6, 1.0, align), 2);
        }
    }

    /// Built from the geometry, not `align_shift`, so it can catch a wrong model.
    /// Six 100px slides, 20px gaps, three up: viewport 340, pitch 120, range 360.
    #[test]
    fn the_reported_slide_is_the_one_in_the_aligned_position() {
        let (slide, gap, per_view, count) = (100.0, 20.0, 3.0, 6);
        let viewport = per_view * slide + (per_view - 1.0) * gap;
        let pitch = slide + gap;
        let max = count as f64 * slide + (count - 1) as f64 * gap - viewport;
        assert_eq!((viewport, pitch, max), (340.0, 120.0, 360.0));

        // Leading edges meet: slide k rests at k pitches.
        for k in 0..=3 {
            let offset = k as f64 * pitch;
            assert_eq!(
                index_at(offset, max, count, per_view, CarouselAlign::Start),
                k
            );
        }

        // Centres meet: k * pitch + slide/2 - viewport/2, clamped at both ends.
        for k in 1..=4 {
            let offset = (k as f64 * pitch + slide / 2.0 - viewport / 2.0).clamp(0.0, max);
            assert_eq!(
                index_at(offset, max, count, per_view, CarouselAlign::Center),
                k,
                "centred slide {k} rests at {offset}"
            );
        }

        // Trailing edges meet: k * pitch + slide - viewport.
        for k in 2..=5 {
            let offset = (k as f64 * pitch + slide - viewport).clamp(0.0, max);
            assert_eq!(
                index_at(offset, max, count, per_view, CarouselAlign::End),
                k,
                "end-aligned slide {k} rests at {offset}"
            );
        }
    }

    /// Index from a ratio: neither slide width nor gap is measured.
    #[test]
    fn an_index_round_trips_through_an_offset_at_any_slide_width() {
        for align in [
            CarouselAlign::Start,
            CarouselAlign::Center,
            CarouselAlign::End,
        ] {
            for (count, per_view) in [(6, 1.0), (6, 3.0), (7, 2.5), (2, 1.0)] {
                for max in [100.0, 999.0, 12345.6] {
                    let (first, last) = index_range(count, per_view, align);
                    for index in first..=last {
                        let offset = offset_for(index, max, count, per_view, align);
                        assert_eq!(
                            index_at(offset, max, count, per_view, align),
                            index,
                            "{align:?}, count {count}, per_view {per_view}, max {max}, index {index}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn a_half_scrolled_strip_snaps_to_the_nearer_slide() {
        let max = 400.0;
        // Four slides, one up: snap points at 0, 133.3, 266.6, 400.
        assert_eq!(index_at(60.0, max, 4, 1.0, CarouselAlign::Start), 0);
        assert_eq!(index_at(70.0, max, 4, 1.0, CarouselAlign::Start), 1);
        assert_eq!(index_at(399.0, max, 4, 1.0, CarouselAlign::Start), 3);
    }

    /// A strip that fits has one position, not a `NaN` index from a zero span.
    #[test]
    fn a_strip_with_nothing_to_scroll_stays_at_zero() {
        assert_eq!(index_at(0.0, 0.0, 3, 3.0, CarouselAlign::Start), 0);
        assert_eq!(index_at(50.0, 0.0, 3, 3.0, CarouselAlign::Start), 0);
        assert_eq!(offset_for(2, 0.0, 3, 3.0, CarouselAlign::Start), 0.0);
        assert_eq!(index_at(0.0, 100.0, 0, 1.0, CarouselAlign::Start), 0);
    }

    #[test]
    fn an_offset_never_leaves_the_scrollable_range() {
        assert_eq!(offset_for(99, 400.0, 4, 1.0, CarouselAlign::Start), 400.0);
        assert_eq!(offset_for(0, 400.0, 4, 1.0, CarouselAlign::Start), 0.0);
    }

    /// The props write `--lsx-carousel-*-override`; the bare var silently ignored them.
    #[test]
    fn the_slide_size_reads_the_per_instance_twin_not_the_bare_theme_var() {
        let slide = Stylesheet::from(&CAROUSEL_SLIDE_SX);
        let track = Stylesheet::from(&CAROUSEL_TRACK_SX);

        assert!(
            slide.as_str().contains("--lsx-carousel-per-view-override"),
            "{}",
            slide.as_str()
        );
        assert!(
            slide.as_str().contains("--lsx-carousel-gap-override"),
            "{}",
            slide.as_str()
        );
        assert!(
            track.as_str().contains("--lsx-carousel-gap-override"),
            "{}",
            track.as_str()
        );
    }

    /// A shrink-to-fit parent would size the root from the track, collapsing a
    /// vertical carousel to a sliver. Checks the declaration, not the layout.
    #[test]
    fn the_root_takes_its_width_from_its_container_not_from_its_slides() {
        let css = Stylesheet::from(&CAROUSEL_ROOT_SX);

        assert!(css.as_str().contains("width:100%"), "{}", css.as_str());
    }

    /// Chrome and Safari keep smooth scrolling under reduced motion, so the
    /// guard has to be ours and has to come after the declaration it undoes.
    #[test]
    fn smooth_scrolling_is_switched_off_under_reduced_motion() {
        let css = Stylesheet::from(&CAROUSEL_TRACK_SX);
        let css = css.as_str();
        let guard = css.find(REDUCED_MOTION).expect("a reduced-motion block");

        assert!(css[guard..].contains("scroll-behavior:auto"), "{css}");
        assert!(
            css[..guard].contains("scroll-behavior:smooth"),
            "the guard has to come after what it overrides: {css}"
        );
    }

    /// A mandatory snap undoes every drag write. The `dragging` arm shares the
    /// axis arms' specificity, so it wins only by coming after both.
    #[test]
    fn a_drag_switches_the_snap_off_after_the_axis_switched_it_on() {
        let css = Stylesheet::from(&CAROUSEL_TRACK_SX);
        let css = css.as_str();
        let off = css
            .find("scroll-snap-type:none")
            .expect("the dragging arm switches the snap off");

        for axis in [
            "scroll-snap-type:x mandatory",
            "scroll-snap-type:y mandatory",
        ] {
            let on = css.find(axis).expect(axis);
            assert!(
                on < off,
                "`{axis}` has to come before the drag's `none`: {css}"
            );
        }
    }

    /// Leading clones are the tail, trailing ones the head.
    #[test]
    fn a_strip_position_maps_back_onto_a_real_slide() {
        assert_eq!(strip_count(5, 1), 7);
        // The leading clone shows the last slide.
        assert_eq!(real_for(0, 5, 1), 4);
        assert_eq!(real_for(1, 5, 1), 0);
        assert_eq!(real_for(5, 5, 1), 4);
        // The trailing clone shows the first.
        assert_eq!(real_for(6, 5, 1), 0);
    }

    #[test]
    fn only_the_cloned_ends_are_a_seam() {
        assert!(is_clone(0, 5, 1));
        assert!(!is_clone(1, 5, 1));
        assert!(!is_clone(5, 5, 1));
        assert!(is_clone(6, 5, 1));
    }

    /// Three-up needs three clones each end, or the strip runs out before the seam.
    #[test]
    fn a_multi_up_strip_clones_a_whole_viewport_at_each_end() {
        assert_eq!(strip_count(6, 3), 12);
        assert_eq!(real_for(0, 6, 3), 3);
        assert_eq!(real_for(2, 6, 3), 5);
        assert_eq!(real_for(3, 6, 3), 0);
        assert_eq!(real_for(9, 6, 3), 0);
    }

    /// The two dot colours are about 1.07:1 apart, so the current dot's own
    /// length is what carries position without hue.
    #[test]
    fn the_current_dot_is_told_apart_by_length() {
        let css = Stylesheet::from(&CAROUSEL_INDICATOR_SX);

        assert!(
            css.as_str()
                .contains("--lsx-carousel-indicator-current-length"),
            "{}",
            css.as_str()
        );
    }

    /// WCAG 2.5.8: a 5px dot takes the pointer over a 24px box, not a fatter dot.
    #[test]
    fn a_dot_takes_the_pointer_over_24px() {
        let css = Stylesheet::from(&CAROUSEL_INDICATOR_SX);

        assert!(css.as_str().contains("::before"), "{}", css.as_str());
        assert!(css.as_str().contains("max(100%, 24px)"), "{}", css.as_str());
    }

    /// The ring sits outside the dot, so it contrasts with the surface; a literal
    /// `background()` made it white on white.
    #[test]
    fn a_dot_does_not_publish_its_own_focus_contrast() {
        let css = Stylesheet::from(&CAROUSEL_INDICATOR_SX);

        assert!(
            !css.as_str().contains("--lsx-focus-contrast:"),
            "{}",
            css.as_str()
        );
    }

    /// Control colours come from the theme; the ring colour from the glyph.
    #[test]
    fn the_controls_read_their_colours_from_the_theme() {
        for sheet in [
            Stylesheet::from(&CAROUSEL_CONTROL_SX),
            Stylesheet::from(&CAROUSEL_PAUSE_SX),
        ] {
            let css = sheet.as_str();
            assert!(
                css.contains("background:var(--lsx-carousel-control-background)"),
                "{css}"
            );
            assert!(
                css.contains("color:var(--lsx-carousel-control-color)"),
                "{css}"
            );
            assert!(
                css.contains("--lsx-focus-contrast:var(--lsx-carousel-control-color)"),
                "{css}"
            );
            assert!(
                css.contains("--lsx-focus-ring-halo:var(--lsx-carousel-control-background)"),
                "{css}"
            );
            assert!(!css.contains("--lsx-surface"), "{css}");
            assert!(!css.contains("--lsx-grey"), "{css}");
        }
        let control = Stylesheet::from(&CAROUSEL_CONTROL_SX);
        assert!(
            control
                .as_str()
                .contains("background:var(--lsx-carousel-control-hover-background)"),
            "{}",
            control.as_str()
        );
    }

    /// The viewport clips the track's edge, so an outset ring is invisible.
    #[test]
    fn the_track_ring_is_inset_because_the_viewport_clips() {
        let css = Stylesheet::from(&CAROUSEL_TRACK_SX);

        assert!(
            css.as_str().contains("outline-offset:-2px"),
            "{}",
            css.as_str()
        );
    }

    /// Which positions go `inert`: those wholly outside the viewport at rest.
    /// A peek stays live, and the clamp at either end counts.
    #[test]
    fn only_a_slide_wholly_outside_the_viewport_goes_inert() {
        let outside = |rest, strip, per_view, align| -> Vec<usize> {
            (0..strip)
                .filter(|&k| outside_viewport(k, rest, strip, per_view, align))
                .collect()
        };
        let (start, center, end) = (
            CarouselAlign::Start,
            CarouselAlign::Center,
            CarouselAlign::End,
        );

        // One up: everything but the slide showing.
        assert_eq!(outside(2, 5, 1.0, center), vec![0, 1, 3, 4]);
        // Three up, centred on 2: 1 to 3 show.
        assert_eq!(outside(2, 6, 3.0, center), vec![0, 4, 5]);
        // Centred on 1, the strip is clamped at 0: 0 to 2 show.
        assert_eq!(outside(1, 6, 3.0, center), vec![3, 4, 5]);
        assert_eq!(outside(5, 6, 3.0, end), vec![0, 1, 2]);
        // 1.5 up: the half slide peeking on the right stays live.
        assert_eq!(outside(0, 5, 1.5, start), vec![2, 3, 4]);
        // Centred on 2 it peeks a quarter each side.
        assert_eq!(outside(2, 5, 1.5, center), vec![0, 4]);
        // Everything fits: nothing to hide.
        assert!(outside(0, 3, 3.0, start).is_empty());
    }

    /// Todo 544: five slides three-up, centred on slide 0, show the clone of
    /// slide 4 on the left. That clone is the live copy; elsewhere the real one.
    #[test]
    fn a_showing_clone_is_its_slides_live_copy() {
        let shows = |position| !outside_viewport(position, 3, 11, 3.0, CarouselAlign::Center);
        assert_eq!(live_copy(4, 5, 3, shows), 2);
        assert_eq!(live_copy(0, 5, 3, shows), 3);
        assert_eq!(live_copy(1, 5, 3, shows), 4);
        // Nothing of slide 2 shows: the real one stays the live copy, offscreen.
        assert_eq!(live_copy(2, 5, 3, shows), 5);
        // Without clones a slide is its own copy.
        assert_eq!(live_copy(2, 5, 0, |_| false), 2);
    }

    /// Todo 550: the slides at least half showing, which a peek is not.
    #[test]
    fn the_range_counts_slides_at_least_half_showing() {
        let (start, center) = (CarouselAlign::Start, CarouselAlign::Center);
        assert_eq!(showing(0, 6, 3.0, start), (0, 2));
        assert_eq!(showing(2, 6, 3.0, center), (1, 3));
        // Clamped at the end: slides 3 to 5.
        assert_eq!(showing(5, 6, 3.0, start), (3, 5));
        // 1.5 up: the half-slide peek counts, a quarter does not.
        assert_eq!(showing(0, 6, 1.5, start), (0, 1));
        assert_eq!(showing(2, 6, 1.5, center), (2, 2));
        assert_eq!(showing(0, 1, 1.0, start), (0, 0));
    }

    /// Todo 619: a slide leaves room for the whole ring stripe of content flush
    /// with its edge; the global reset's `border-box` keeps it inside the basis.
    #[test]
    fn a_slide_pads_by_the_focus_ring() {
        let css = Stylesheet::from(&CAROUSEL_SLIDE_SX);
        let css = css.as_str();

        assert!(
            css.contains(
                "padding:calc(var(--lsx-focus-ring-offset) + var(--lsx-focus-ring-width))"
            ),
            "{css}"
        );
    }

    /// Not looping is the same code with no clones: a pass-through.
    #[test]
    fn without_clones_a_position_is_the_slide() {
        assert_eq!(strip_count(5, 0), 5);
        assert_eq!(real_for(3, 5, 0), 3);
        assert!(!is_clone(0, 5, 0));
        assert!(!is_clone(4, 5, 0));
    }

    /// Only a first controlled scroll that moves is instant: a no-op never
    /// settles to lower `seam`.
    #[test]
    fn only_a_first_scroll_that_moves_is_instant() {
        assert!(instant_scroll(true, false, 0, 5, 0, 0));
        assert!(!instant_scroll(false, false, 0, 5, 0, 0));
        assert!(!instant_scroll(true, false, 0, 0, 0, 0));
        assert!(!instant_scroll(true, false, 0, 2, 2, 2));
        // A looping strip mounts on a clone: its first scroll always moves.
        assert!(instant_scroll(true, false, 2, 0, 0, 0));
    }

    /// Todo 367: a swap counts once. The second read of the same count is no
    /// swap, or every later controlled move would take the instant path.
    #[test]
    fn a_swap_is_consumed_by_the_first_read() {
        let mut seen = Some(0);

        assert!(!consume_swap(&mut seen, Some(0)));
        assert!(consume_swap(&mut seen, Some(1)));
        assert!(!consume_swap(&mut seen, Some(1)));
        assert!(!consume_swap(&mut seen, Some(1)));
        assert!(consume_swap(&mut seen, Some(2)));
        assert!(!consume_swap(&mut seen, Some(2)));
    }

    /// No provider, no swaps.
    #[test]
    fn without_a_provider_nothing_is_a_swap() {
        let mut seen = None;

        assert!(!consume_swap(&mut seen, None));
        assert!(!consume_swap(&mut seen, None));
    }

    /// Todo 323: a move that arrives with a swap of the slides is instant
    /// too, whenever it leaves the slide the strip is on.
    #[test]
    fn a_move_with_a_slide_swap_is_instant() {
        assert!(instant_scroll(false, true, 0, 2, 5, 0));
        assert!(instant_scroll(false, true, 2, 2, 5, 0));
        assert!(!instant_scroll(false, true, 0, 5, 5, 0));
    }
}
