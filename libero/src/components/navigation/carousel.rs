use std::{cell::Cell, rc::Rc, time::Duration};

use dioxus::prelude::*;

use crate::{
    components::{
        Box, HtmlTag, Input, Orientation, States, Variables, VisuallyHidden,
        common::{
            ChevronDownIcon, ChevronLeftIcon, ChevronRightIcon, ChevronUpIcon, PauseIcon, PlayIcon,
            base_props, focus_ring_sx, has_shortcut_modifier, input_from_str, inset_focus_ring_sx,
            shadow_sx, states, use_name_warning, variables,
        },
        layout::{
            ScrollArea, ScrollAreaBase, ScrollAreaHandle, ScrollPositionEvent, scroll_area_base,
            use_box, use_scroll_area,
        },
    },
    hooks::{
        Drag, DragMove, DragOptions, DragStart, ElementHandle, use_drag, use_element, use_id,
        use_localization, use_silent_focus_within, use_theme,
    },
    localization::{CarouselLabels, fill},
    platform::{
        ElementApi, TimerSubscription, arrow_target, key_taken, prefers_reduced_motion, timer,
        typing_target,
    },
    sx::{FORCED_COLORS, REDUCED_MOTION, StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        CAROUSEL_CONTROL_BACKGROUND, CAROUSEL_CONTROL_COLOR, CAROUSEL_CONTROL_HOVER_BACKGROUND,
        CAROUSEL_CONTROL_SIZE, CAROUSEL_CONTROLS_OFFSET, CAROUSEL_GAP, CAROUSEL_INDICATOR_COLOR,
        CAROUSEL_INDICATOR_CURRENT_COLOR, CAROUSEL_INDICATOR_CURRENT_LENGTH,
        CAROUSEL_INDICATOR_LENGTH, CAROUSEL_INDICATOR_THICKNESS, CAROUSEL_INDICATORS_GAP,
        CAROUSEL_PER_VIEW, CAROUSEL_RADIUS, CssVar, FOCUS_RING_HALO, FOCUS_RING_WIDTH,
        NamedColorCss, Size, SizeCss,
    },
};

pub use crate::theme::CarouselAlign;

input_from_str!(CarouselAlign);

static CAROUSEL_ROOT_SX: StaticSx = StaticSx::new(|| {
    // The controls sit over the viewport, so the root is their containing
    // block.
    sx().position("relative")
        .display("block")
        // A viewport takes its cross-axis size from its container, never from
        // its slides - `ScrollArea`'s root says the same thing. Without this
        // the root shrink-to-fits wherever it is not a plain block child (a
        // flex item, a grid cell), and then its width is the *track's*
        // max-content width. A horizontal track overflows that and gets
        // clamped back to the space available, so it fills by accident; a
        // vertical one is as wide as its widest slide's text and collapses to
        // a sliver.
        .width("100%")
});

static CAROUSEL_VIEWPORT_SX: StaticSx = StaticSx::new(|| {
    // The controls are absolute against *this*, not against the root: the root
    // is the viewport plus the indicator strip, and positioning against it put
    // a vertical carousel's next control 172px below the bottom of the strip,
    // among the dots. They are inset by `CAROUSEL_CONTROLS_OFFSET`, so the
    // `overflow: hidden` here does not clip them.
    sx().position("relative").overflow("hidden")
});

/// Merged onto `ScrollArea`'s own, which scrolls the carousel's axis and hides
/// the scrollbar: that is the indicators' job here, the strip still scrolls.
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
            // A carousel inside a scrollable page should not scroll the page when
            // it reaches its own end.
            .overscroll_behavior_x("contain")
            .overscroll_behavior_y("contain")
            .height(CAROUSEL_HEIGHT.value_or("auto"))
            .scroll_behavior("smooth")
            // Chrome and Safari do **not** disable `scroll-behavior: smooth` under
            // reduced motion - only Firefox does - so the guard is explicit. It
            // sits at the same specificity as the declaration it overrides and
            // after it, which is what settles the two.
            .media(REDUCED_MOTION, sx().scroll_behavior("auto"))
            // A drag is the pointer's own position: animating towards it lags,
            // and a mandatory snap pulls every write back to the slide it left, so
            // the strip sat still and then jumped a whole slide. After the
            // orientation arms, which it has to beat at equal specificity.
            .when(
                "dragging",
                sx().scroll_behavior("auto").scroll_snap_type("none"),
            )
            .when("seam", sx().scroll_behavior("auto"))
            // Inset: the viewport is `overflow: hidden` and exactly this size, so
            // an outset ring is clipped away entirely.
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
        // The slide and the track clip flush at its edges, so a focusable slide
        // content loses an outset ring (todo 618), as in `AspectRatio`. Doubled
        // to outrank a `Button`'s own ring, which ties it otherwise.
        .selector(
            "& > :focus-visible:focus-visible",
            inset_focus_ring_sx(&format!("calc(-1 * {})", FOCUS_RING_WIDTH.value())),
        )
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

/// The fill and glyph every control shares, behind the theme's vars so a dark
/// theme has one place to change them. A var is opaque to `background()`, so
/// the `--lsx-focus-contrast` that `background("white")` used to publish is
/// declared by hand - the glyph colour, which is what reads against the fill -
/// or the ring would fall back to whatever the page around the carousel set.
/// The fill is the ring's halo, as `background()` publishes it.
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
        .hover(sx().background(CAROUSEL_CONTROL_HOVER_BACKGROUND.value()))
        // Disabled by `aria-disabled`, not `disabled`: the button keeps its
        // tab stop so focus is never dropped at either end.
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
        // Behind a var, not a literal: `background("muted.6")` would publish
        // this dot's own `--lsx-focus-contrast`, and the ring - drawn outside
        // the dot, on the page - would then contrast against the dot instead
        // of against what it sits on. On the current dot that was a white ring
        // on a white page.
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
        // Colour alone would not carry it: `primary.6` and `grey.6` are within
        // about 1.07:1 of each other, so in greyscale or with a colour vision
        // deficiency the current dot would be its neighbours' twin. Length is
        // the second channel, the way Mantine widens the active dot.
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
        // Forced colours paint every dot `Canvas`: the strip vanished, and the
        // current dot with it.
        .media(
            FORCED_COLORS,
            sx().background("CanvasText")
                .when("current", sx().background("Highlight")),
        )
        .focus_visible(focus_ring_sx())
        // WCAG 2.5.8: the dot is drawn 5px thick, so an invisible box at least
        // 24px on both axes and centred on it takes the pointer, as `Slider`'s thumb.
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
        // Beside Next's column, not in it: on a short strip the corner is where
        // Next sits, and the pause button covered it.
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

/// The indices that can actually be snapped to, inclusive.
///
/// Two things narrow it. With `per_view` above 1 the final slides share the
/// viewport, so the strip runs out of scroll before it runs out of slides - six
/// slides three-up stop at 3, not 5. And the alignment slides the whole window:
/// the same strip centred reaches 1 through 4, and end-aligned 2 through 5,
/// because at either end the browser clamps the scroll and the slide sitting in
/// the aligned position is not the one that was asked for.
///
/// At `per_view: 1.0` all three alignments coincide and this is `0..=count-1`,
/// which is why nothing noticed the alignment was missing.
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

/// Where `index` stands among the positions the strip can actually rest at,
/// and how many there are - what the status and the dots count, as Mantine
/// counts Embla's snaps rather than slides. Six slides three-up rest at four
/// places, so the status runs "1 of 4" to "4 of 4", and a strip whose slides
/// all fit rests at one: "1 of 1". A looping strip has a position per slide.
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

/// Whether strip position `position` lies wholly outside the viewport while the
/// strip rests on `rest` - the slides that go `inert`.
///
/// In pitches (`u = slide + gap`) slide `k` covers `k..k + 1 - gap/u` and the
/// viewport `p..p + per_view - gap/u`, where `p` is the aligned rest clamped to
/// the scroll range. The gap is not measured, so it is taken as zero on both
/// sides: that only ever keeps a slide live, never hides one that shows. A
/// peeking slide therefore stays live.
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

/// A dot's own id, derived from the track's so two carousels on one page do
/// not collide.
fn indicator_id(track_id: &str, index: usize) -> String {
    format!("{track_id}-indicator-{index}")
}

/// Moves focus onto the dot the arrows just made current. Synchronous, and a
/// miss is not worth reporting: the dot is there in the same render.
fn focus_indicator(root: ElementHandle, track_id: &str, index: usize) {
    let selector = format!("#{}", indicator_id(track_id, index));
    let _ = root.query_selector(&selector).and_then(|dot| dot.focus());
}

/// How far the snapped slide's index sits from the scroll offset's, in slides.
///
/// `scroll-snap-align` decides which edge of a slide meets which edge of the
/// viewport, so it shifts the whole mapping. Writing the pitch as
/// `u = slide + gap`, the viewport is `per_view * u - gap` wide, and slide `k`
/// comes to rest at `u * (k - shift)`: nothing for `start`, half the extra
/// slides for `center`, all of them for `end`. It is a function of `per_view`
/// alone, so folding it in still measures nothing.
fn align_shift(per_view: f64, align: CarouselAlign) -> f64 {
    match align {
        CarouselAlign::Start => 0.0,
        CarouselAlign::Center => (per_view - 1.0) / 2.0,
        CarouselAlign::End => per_view - 1.0,
    }
}

/// The snapped index for a scroll offset, from the three numbers `ScrollData`
/// hands over.
///
/// Exact for an equal-width snap strip without measuring anything: the maximum
/// offset is `(count - per_view) * u`, so dividing by it cancels both the slide
/// width and the gap, and [`align_shift`] puts the result back on the slide the
/// viewport is actually showing.
///
/// At the two ends the browser clamps the scroll, so under `center` or `end`
/// the first and last slides cannot be brought to that position - and the index
/// reported there is the slide genuinely in it, not the one that was asked for.
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

/// The inverse of [`index_at`], clamped to what the browser will actually
/// scroll to.
fn offset_for(index: usize, max: f64, count: usize, per_view: f64, align: CarouselAlign) -> f64 {
    let span = count as f64 - per_view;
    if max <= 0.0 || span <= 0.0 {
        return 0.0;
    }
    ((index as f64 - align_shift(per_view, align)) / span * max).clamp(0.0, max)
}

/// How many items a looping strip holds: the slides plus the clones at both
/// ends. `clones` is zero when not looping, and this is then the slide count.
fn strip_count(count: usize, clones: usize) -> usize {
    count + 2 * clones
}

/// The real slide a strip position shows. The leading clones are the tail and
/// the trailing ones are the head, so either side of the seam wraps around.
fn real_for(raw: usize, count: usize, clones: usize) -> usize {
    if count == 0 {
        return 0;
    }
    (raw as isize - clones as isize).rem_euclid(count as isize) as usize
}

/// Whether a strip position is a cloned end rather than a real slide - the
/// moment the seam has to be crossed.
fn is_clone(raw: usize, count: usize, clones: usize) -> bool {
    clones > 0 && (raw < clones || raw >= clones + count)
}

/// Whether a controlled index's scroll is instant: the first one after mount,
/// or one that arrives with a swap of the slides ([`CarouselJump`]). Only when
/// it moves at all: where the strip already is, no scroll settles, and a
/// raised `seam` would never come down. A looping strip mounts on a clone, so
/// its first scroll always moves.
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

/// Provided by a caller that replaces a controlled carousel's slides and its
/// `index` in the same render: `Lightbox`, on a second `open_with` while it
/// is open. A controlled move that arrives with a swap is instant. A smooth
/// scroll from the old index would pass over slides of the new set that are
/// not being shown, and a lazy picture on one of them is fetched as it goes
/// by (todo 323). A context rather than a prop: no caller outside the crate
/// swaps slides this way, and a remount instead would drop focus off the
/// controls.
///
/// A count of swaps, not a flag for the swapping render: the carousel's
/// effect compares it with the last count it saw, so the swap is not lost
/// when the provider renders again before that effect runs.
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

/// Whether a swap has arrived since the last time this was asked, and mark it
/// as read. Reading it is what consumes it: a count that stayed unread would
/// make every later controlled move instant too, and the carousel would have
/// lost its animation for good.
fn consume_swap(seen: &mut Option<u64>, swaps: Option<u64>) -> bool {
    swaps != std::mem::replace(seen, swaps)
}

/// Scrolls the track so `index` is the snapped slide. The offset is a share of
/// the range, so it goes over as a percent and `ScrollArea` measures the range.
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

/// Per-instance only: a carousel's height has no default worth publishing on
/// the theme, and `auto` is what a horizontal strip wants.
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

/// Everything a move needs, in one `Copy` bundle.
///
/// Built fresh on every render and captured by copy into each handler, rather
/// than closed over by a `use_callback` - a handler built once would keep the
/// first render's `count` and `per_view` and quietly navigate against a stale
/// strip.
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
    /// Slides cloned onto each end so a looping strip has something to scroll
    /// into past either edge. Zero when not looping.
    clones: usize,
}

impl Nav {
    /// How many items the strip actually holds - the slides plus the clones at
    /// both ends.
    fn strip_count(self) -> usize {
        strip_count(self.count, self.clones)
    }

    /// A real slide's position in the strip.
    fn raw_for(self, index: usize) -> usize {
        index + self.clones
    }

    /// The real slide a strip position shows, wrapping through the clones: the
    /// leading clones are the tail and the trailing ones are the head.
    fn real_for(self, raw: usize) -> usize {
        real_for(raw, self.count, self.clones)
    }

    /// Whether a strip position is one of the cloned ends rather than a real
    /// slide - the moment the seam has to be crossed.
    fn is_clone(self, raw: usize) -> bool {
        is_clone(raw, self.count, self.clones)
    }

    /// A deliberate move: control, key, indicator or timer. Sets both indices
    /// at once, because the caller asked for this one rather than scrolling
    /// into it.
    /// The furthest index this carousel will go to. A looping strip navigates
    /// over the real slides; a plain one stops where the scrolling does.
    fn clamp_index(self, index: usize) -> usize {
        match self.clones > 0 {
            true => index.min(self.count.saturating_sub(1)),
            false => index.clamp(self.first, self.last),
        }
    }

    /// Pulls both indices into the reachable window. No scroll: at rest the
    /// strip is already showing the slide this corrects to - the reading was
    /// wrong, not the position. And no callback: an uncontrolled carousel has
    /// no second party holding a wrong value, and a controlled one is told by
    /// the effect that reads its `index`.
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
        // Against `settled`, which is the last index the caller was told.
        // `End` pressed twice, or an arrow on the last dot, is not a change.
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

    /// Asks for the seam to be crossed: the strip has settled on a clone, and
    /// has to jump to the real slide showing the same thing. `seam` switches
    /// smooth scrolling off for the jump, or the strip visibly rewinds - so
    /// this only raises it, and the jump waits for the render that puts it in
    /// the DOM. See the effect that reads it.
    fn cross_seam(mut self) {
        self.seam.set(true);
    }

    /// The strip position a scroll report names: `index_at` only needs the
    /// offset as a share of the range, which is exactly the percent.
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
        /// The slides, in order. Not `children`: reasoning about order and
        /// count is this component's whole job, and dioxus cannot inspect
        /// children ([[principles/component-api-design]] §4).
        #[props(default)]
        slides: Vec<Element>,
        /// Each slide group's accessible name. Defaults to the theme's
        /// `"{n} of {m}"`.
        #[props(default)]
        slide_label: Option<Callback<usize, String>>,
        /// The current slide. Set it and the carousel follows; leave it unset
        /// and the carousel keeps its own.
        #[props(default)]
        index: Option<usize>,
        /// Fired once a scroll settles, and on every control, key and
        /// indicator.
        #[props(default)]
        onindexchange: Option<EventHandler<usize>>,
        /// Slides visible at once. Fractional peeks the next one.
        #[props(default, into)]
        per_view: Input<f64>,
        #[props(default, into)]
        gap: Input<Size>,
        #[props(default, into)]
        align: Input<CarouselAlign>,
        /// `"horizontal"` by default - unlike `Orientation`'s own default.
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// Required for a vertical carousel, which has nothing to take its
        /// height from.
        #[props(default, into)]
        height: Input<ThemeAwareValue>,
        #[props(default)]
        controls: Option<bool>,
        #[props(default)]
        indicators: Option<bool>,
        /// Names the region. There is no sensible default for a set of
        /// slides, so leaving it unset is a `warn()`.
        #[props(default, into)]
        aria_label: Option<String>,
        /// Mouse drag-to-scroll over the track. Touch already swipes - that is
        /// the platform's own scroll - and this never applies
        /// `touch-action: none`, which would take it away.
        #[props(default)]
        draggable: bool,
        /// Advances on a timer. Ships with a pause control, and pauses itself
        /// on hover and on focus within, per WCAG 2.2.2. Starts paused under
        /// reduced motion.
        #[props(default)]
        autoplay: bool,
        /// Milliseconds between advances. Defaults to the theme's.
        #[props(default)]
        autoplay_delay: Option<u32>,
        /// Wraps at both ends, by cloning enough slides onto each end for the
        /// strip to scroll past the edge and jumping back across the seam once
        /// it settles. The clones are `aria-hidden`, so the content is not
        /// duplicated for a screen reader.
        #[props(default)]
        r#loop: bool,
    }
}

/// A scroll-snap strip that knows which slide it is on.
///
/// The index is derived from the scroll position rather than from intent,
/// which is what makes a native touch swipe, a keyboard arrow and a control
/// click all end up in the same place.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Carousel, Image};
/// # fn app() -> Element {
/// # struct Photo { url: String, alt: String }
/// # let photos: Vec<Photo> = Vec::new();
/// # rsx! {
/// Carousel {
///     aria_label: "Product photos",
///     per_view: 3.0,
///     indicators: true,
///     slides: photos.iter().map(|p| rsx! { Image { src: "{p.url}", alt: "{p.alt}" } }).collect(),
/// }
/// # } }
/// ```
#[component]
pub fn Carousel(props: CarouselProps) -> Element {
    let theme = use_theme();
    let labels = &use_localization().carousel;
    let track = use_scroll_area();
    // The indicators sit outside the track, so the roving focus looks them up
    // from the root.
    let root_handle = use_element();
    let track_id = use_id();
    // The status region describes the track, so the tab stop says where it is.
    let status_id = use_id();

    let jump = try_use_context::<CarouselJump>();

    let count = props.slides.len();
    let per_view = props.per_view.copied_or(theme.carousel.per_view).max(0.1);
    // `Orientation` defaults to vertical; a carousel does not.
    let orientation = props.orientation.copied_or(Orientation::Horizontal);
    let align = props.align.copied_or(theme.carousel.align);
    // No slides is not "one slide": there is no position to announce, no dot
    // to go to and nothing to rotate, so none of the chrome renders. The root
    // stays, so a caller's size and placement hold while slides load, but it
    // is no landmark and the empty track is no tab stop - a named region with
    // nothing in it is noise in a screen reader's landmark list.
    let empty = count == 0;
    let controls = !empty && props.controls.unwrap_or(theme.carousel.controls);
    let indicators = !empty && props.indicators.unwrap_or(theme.carousel.indicators);
    let (first, last) = index_range(count, per_view, align);

    // Read in render, so a slide swap is a dependency of the effect that
    // applies a controlled index, which then runs even when the swap leaves
    // the index where it was.
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

    // A named region beats an unnamed one even when the name is generic, so
    // the localization's stands in - and `use_carousel_state`'s warning still
    // says to do better.
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

    // Blitz's Tab fires neither `focusin` nor `focusout`: the silent move does.
    use_silent_focus_within(root_handle, move |within| {
        let mut focused = state.focused;
        focused.set(within)
    });

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
        .event("onfocusin", move |_: Event<FocusData>| {
            let mut focused = state.focused;
            focused.set(true)
        })
        .event("onfocusout", move |_: Event<FocusData>| {
            let mut focused = state.focused;
            focused.set(false)
        });

    let body = carousel_slides(view, &props.slides, props.slide_label);

    root.render(
        HtmlTag::Section,
        props.attributes,
        rsx! {
            if !empty {
                CarouselStatus { view }
            }
            Box { framework_sx: &CAROUSEL_VIEWPORT_SX,
                {carousel_track(view, aria_label, props.draggable, drag, body)}
                if controls {
                    CarouselControls { view }
                }
            }
            if props.autoplay && !empty {
                {carousel_pause_button(view, controls)}
            }
            if indicators {
                CarouselIndicators { view }
            }
        },
    )
}

/// Everything `Carousel`'s state hook reads off the props and the theme, in
/// one reactive argument: an effect keyed on this re-runs when any of it
/// moves.
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
}

/// `Carousel`'s live state: the strip mover and the two indices in `nav`, the
/// three reasons autoplay pauses, and the drag. Every effect that moves the
/// strip lives in [`use_carousel_state`], so the component body is left with
/// rendering.
#[derive(Clone, Copy, PartialEq)]
struct CarouselState {
    nav: Nav,
    paused: Signal<bool>,
    hovered: Signal<bool>,
    focused: Signal<bool>,
    dragging: Signal<bool>,
    drag_origin: Signal<f64>,
    /// The controlled index the effect last applied. Until it runs, a new one
    /// stands in for `settled` when the slides go `inert` - see `carousel_slides`.
    applied: Signal<Option<usize>>,
    /// Autoplay is on and nothing is holding it back.
    running: bool,
}

/// `template` with `{n}` one-based, because it is read aloud, and `{m}` the
/// count.
pub(crate) fn numbered(template: &str, index: usize, count: usize) -> String {
    fill(template, &[("n", &(index + 1)), ("m", &count)])
}

/// One argument for every part of the carousel's chrome: the resolved props,
/// the live state, the strings and the ids.
#[derive(Clone, Copy, PartialEq)]
struct CarouselView {
    setup: CarouselSetup,
    state: CarouselState,
    labels: &'static CarouselLabels,
    track_id: Signal<String>,
    status_id: Signal<String>,
    root: ElementHandle,
}

/// The carousel's own state, and every effect that moves the strip: the
/// reachable window, the looping strip's opening position, the seam jump, a
/// controlled `index`, and autoplay.
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

    // Two indices, deliberately: `current` follows the scroll frame by frame so
    // the indicators and the controls feel live, `settled` only moves when the
    // scroll comes to rest, so the live region does not read every frame.
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

    // A named region beats an unnamed one even when the name is generic, so
    // the theme's stands in - and the warning still says to do better.
    use_name_warning(
        named,
        "Carousel: no `aria_label`, falling back to the theme's. A region needs a name of its own to be told apart in a landmark list.",
    );

    // One clone per visible slide at each end, so the strip can scroll a full
    // viewport past either edge before the seam is crossed.
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

    // The reachable window moves at runtime - `align` and `per_view` both move
    // it, and `count` can shrink under a controlled index - so this is an
    // effect keyed on the range rather than a seed. `use_signal`'s initialiser
    // runs once, which would leave a carousel whose window later excludes its
    // index with no tab stop in the indicator strip, no `aria-current`, and
    // nothing to correct it, since at rest no scroll settles.
    use_effect(use_reactive!(|nav| {
        nav.pull_into_range();
    }));

    // The seam jump itself, from an effect rather than from the handler that
    // asks for it. The jump is only instant under the `seam` state's
    // `scroll-behavior: auto`, and a state raised in a handler is not in the
    // DOM until the next render. Scrolling from the handler started while the
    // track was still `smooth`, and rewound across the whole strip - the one
    // thing the clones exist to prevent. `current` is already the real slide:
    // the settle that found the clone wrote it.
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

    // A caller driving `index` from outside. The whole tuple is reactive, so
    // the closure is rebuilt whenever any of it changes rather than capturing
    // the first render's values.
    let mut mounting = use_signal(|| true);
    let mut applied = use_signal(|| None::<usize>);
    let mut swaps_seen = use_signal(|| swaps);
    use_effect(use_reactive!(|controlled, nav, swaps| {
        let Some(index) = controlled else {
            return;
        };
        applied.set(Some(index));
        // Through `nav`, like every other mover: a looping strip keeps the real
        // slide `clones` positions in, and the clamp has to be the same one
        // `go_to` uses or a controlled index is legal through one path and
        // silently trimmed through the other.
        let asked = index;
        let index = nav.clamp_index(asked);
        // Where the strip is before this move: `current` follows the scroll.
        let from = *current.peek();
        if index != from {
            current.set(index);
            settled.set(index);
        }
        // The strip mounts at offset 0, so a smooth first scroll to a
        // controlled index swept past every earlier slide (a lightbox opened
        // on 6 of 6 took about 1.5s). `seam` is the state that switches smooth
        // scrolling off; like the seam jump, the scroll waits for the render
        // that puts it in the DOM - the effect that reads it - and the settle
        // on a real slide lowers it again.
        let first_scroll = std::mem::replace(&mut *mounting.write(), false);
        let swapped = consume_swap(&mut swaps_seen.write(), swaps);
        match instant_scroll(first_scroll, swapped, nav.clones, index, from, nav.first) {
            true => seam.set(true),
            false => nav.scroll_to_raw(nav.raw_for(index)),
        }
        // The caller is holding an index that cannot be shown, so say which
        // one is - here, where every such index arrives, and not only at
        // mount. Otherwise it pushes the unreachable value back on every
        // render, the clamp undoes it on every render, and the two disagree
        // until something else moves the carousel.
        if index != asked
            && let Some(handler) = &nav.onindexchange
        {
            handler.call(index);
        }
    }));

    let running = autoplay && !paused() && !hovered() && !focused() && count > 1;
    // The timer's callback runs outside every scope (`TimerApi`), and a move
    // needs one: `scroll_to_index` spawns. So a tick only counts, and the
    // effect below it, which has a scope, does the moving. A `go_to` from the
    // callback set both indices and then panicked in `spawn` on the web - the
    // dots advanced and the strip never did.
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
        // A looping strip runs over every slide; a plain one stops where the
        // scrolling does.
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
        dragging,
        drag_origin,
        applied,
        running,
    }
}

/// Mouse drag-to-scroll over the track. Deliberately **not** given
/// `drag_handle_sx()`: that is `touch-action: none`, and it would take away
/// the native touch swipe that is the whole reason the track is a scroll
/// container. So a finger scrolls the platform's way and a mouse drags, on
/// every backend.
fn use_carousel_drag(setup: CarouselSetup, state: CarouselState, draggable: bool) -> Drag {
    let track = setup.track;
    let orientation = setup.orientation;
    let mut dragging = state.dragging;
    let mut drag_origin = state.drag_origin;

    use_drag(DragOptions {
        capture: track.element,
        onstart: Callback::new(move |start: DragStart| {
            if !draggable {
                start.cancel.call(());
                return;
            }
            dragging.set(true);
            let offset = track.element.scroll_offset();
            spawn(async move {
                if let Ok((x, y)) = offset.await {
                    drag_origin.set(match orientation {
                        Orientation::Horizontal => x,
                        Orientation::Vertical => y,
                    });
                }
            });
        }),
        onmove: Callback::new(move |moved: DragMove| {
            let delta = moved.delta();
            let target = match orientation {
                Orientation::Horizontal => drag_origin() - delta.x,
                Orientation::Vertical => drag_origin() - delta.y,
            }
            .max(0.0);
            match orientation {
                Orientation::Horizontal => track.scroll_to(target, 0.0),
                Orientation::Vertical => track.scroll_to(0.0, target),
            }
        }),
        // No settle of our own: releasing hands the strip back to the
        // browser, which snaps and fires `onscrollend`.
        onend: Callback::new(move |()| dragging.set(false)),
    })
}

/// The strip: the tail cloned onto the front, the slides, the head cloned onto
/// the back, each in its own slide group. Without looping the clones are empty
/// and this is just the slides.
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
            let label = (!is_clone).then(|| label_for(index));
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

/// One slide group. Its own scope, so a move redraws only the slides whose
/// `data-current` or `inert` flips, not the track and every slide.
#[component]
fn CarouselSlide(
    slide: Element,
    nav: Nav,
    controlled: Option<usize>,
    applied: Signal<Option<usize>>,
    position: usize,
    index: usize,
    is_clone: bool,
    label: Option<String>,
) -> Element {
    let flags =
        use_memo(use_reactive!(|nav,
                                controlled,
                                position,
                                index,
                                is_clone| {
            // Where the strip rests, for `inert`: `settled`, never `current`, or the
            // slides would flip on every scroll frame. A controlled index the effect
            // has not applied yet counts already, so the slide a caller moves to is
            // live in the same render - `Lightbox` focuses its picture from an effect
            // of its own, which may run before ours.
            let rest = match controlled {
                Some(held) if Some(held) != *applied.peek() => nav.clamp_index(held),
                _ => (nav.settled)(),
            };
            // Wholly offscreen at rest: out of the Tab order and the reading
            // order. A press on it still reaches the track (measured).
            let hidden = outside_viewport(
                position,
                nav.raw_for(rest),
                nav.strip_count(),
                nav.per_view,
                nav.align,
            );
            (hidden, !is_clone && index == (nav.current)())
        }));
    let (hidden, current) = flags();
    let track = nav.track;
    // No `current` token: nothing in `CAROUSEL_SLIDE_SX` styles one, and
    // `data-current` below is what a caller actually reads.
    let slide_states: Input<States> = states().with(nav.align.state_name(), true).into();

    rsx! {
        Box {
            framework_sx: &CAROUSEL_SLIDE_SX,
            states: slide_states,
            role: if is_clone { None } else { Some("group") },
            aria_roledescription: if is_clone { None } else { Some("slide") },
            aria_label: label,
            // A clone is the same content twice over, so it is hidden
            // rather than announced a second time.
            aria_hidden: is_clone.then(|| "true".to_string()),
            inert: hidden.then_some(true),
            // Focus inside a slide that has just gone `inert` - the
            // wheel, a drag, a native arrow on a button, a caller's
            // index - is blurred by the browser and lands on `<body>`.
            // The track takes it instead: it is what the keyboard
            // was on, one level up.
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

/// The scroll container, which is also the carousel's tab stop: the strip, the
/// two scroll listeners that keep the indices honest, the arrow keys and the
/// mouse drag.
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

    // Only a settled scroll moves `settled`, which is what the live region and
    // `onindexchange` read - a scroll in progress moves `current` alone.
    let mut onscrollend = move |x: f64, y: f64| {
        // A drag writes an instant scroll per pointer move, and each of those
        // ends too. None is a settle: treating them as one reported the index
        // mid-drag and, on a looping strip, jumped the seam out from under the
        // pointer on every move past half a slide. Releasing switches the snap
        // back on, and the scroll that settles the strip ends after this.
        if *dragging.peek() {
            return;
        }
        let raw = nav.raw_at(x, y);
        let next = nav.real_for(raw);
        current.set(next);
        if next != *settled.peek() {
            settled.set(next);
            if let Some(handler) = &nav.onindexchange {
                handler.call(next);
            }
        }
        // Settled on a clone: jump to the real slide showing the same thing.
        // The jump scrolls again and so settles again, but that landing is a
        // real slide and crosses nothing.
        match nav.is_clone(raw) {
            true => nav.cross_seam(),
            false => {
                if *seam.peek() {
                    seam.set(false);
                }
            }
        }
    };

    // On the **track**, not the root: the arrows belong to the scroll
    // container, which is also the tab stop. The controls, the pause button
    // and the indicators are siblings of the viewport rather than descendants
    // of the track, so their keys cannot reach this at all - the `Tree` rule
    // of preventing a bubble by structure instead of stopping it.
    //
    // A slide's own focusable content is the case structure cannot separate,
    // and the three guards below are how it is separated instead. A caller's
    // `TextField` inside a slide is not code this component owns and cannot be
    // asked to stop propagating - without them, `prevent_default` here eats
    // caret movement, Home/End and option selection from slide content *and*
    // advances the carousel underneath it.
    //
    // `key_taken` is the press something nearer already acted on, marked the
    // one way it is marked everywhere - `Slider`, `RadioGroup` and
    // `SegmentedControl` each prevent the default on the arrows, so every one
    // of our own controls is covered by it. The other two are the cases that
    // cannot mark themselves, because the browser's own default action is the
    // wanted behaviour and nothing prevents it: `typing_target` for a caret
    // move in a text entry, a `select` or a `contenteditable`, and
    // `arrow_target` for raw HTML a caller wrote - an `<input type="range">`
    // or an `<input type="radio">`, which step on an arrow without typing.
    let onkeydown = move |event: Event<KeyboardData>| {
        if key_taken(&event)
            || typing_target(&event)
            || arrow_target(&event)
            || has_shortcut_modifier(&event)
        {
            return;
        }
        let Some(target) = track_key_target(event.key(), orientation, nav, current()) else {
            return;
        };
        // Bubble phase is enough: a default action is preventable from any
        // phase, and no slide stops propagation. Without this the native
        // scroll runs as well and lands between two snap points.
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
            // The track is the scrollable region, so it is the tab stop - the
            // opposite of `ScrollArea`'s default, and on purpose. Empty, there
            // is nothing to scroll to.
            focusable: !empty,
            framework_sx: ScrollAreaBase(&CAROUSEL_TRACK_SX),
            states: track_states,
            id: track_id(),
            // The tab stop names itself: landing on a descendant does not
            // reliably re-announce the region, and the status says where.
            role: (!empty).then_some("group"),
            aria_label: (!empty).then(|| aria_label.clone()),
            aria_describedby: (!empty).then_some(status_id()),
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

/// One control. Disabled by `aria-disabled` at its end, so it keeps its tab stop.
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
    let disabled = disabled();
    let control_states: Input<States> = states()
        .with(orientation.state_name(), true)
        .with("disabled", disabled)
        .into();

    rsx! {
        Box {
            component: "button",
            r#type: "button",
            framework_sx: &CAROUSEL_CONTROL_SX,
            states: control_states,
            aria_controls: track_id(),
            aria_disabled: disabled.to_string(),
            aria_label: if forward { labels.next } else { labels.previous },
            onclick: move |_| {
                let index = *current.peek();
                match (forward, looping) {
                    (false, true) if index == 0 => nav.go_to(count.saturating_sub(1)),
                    (true, true) if index >= count.saturating_sub(1) => nav.go_to(0),
                    _ if disabled => {}
                    (false, _) => nav.go_to(index.saturating_sub(1)),
                    (true, _) => nav.go_to(index + 1),
                }
            },
            {match (orientation, forward) {
                (Orientation::Horizontal, false) => rsx! { ChevronLeftIcon {} },
                (Orientation::Vertical, false) => rsx! { ChevronUpIcon {} },
                (Orientation::Horizontal, true) => rsx! { ChevronRightIcon {} },
                (Orientation::Vertical, true) => rsx! { ChevronDownIcon {} },
            }}
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
    let (position, positions) = snap_position(
        settled(),
        setup.count,
        setup.first,
        setup.last,
        nav.clones > 0,
    );
    let status = numbered(labels.status, position, positions);

    rsx! {
        VisuallyHidden {
            id: status_id(),
            role: "status",
            // Off while it rotates on its own: an unattended change is not
            // worth interrupting a screen reader for, and it becomes
            // `polite` the moment the rotation stops. WCAG 2.2.2.
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
            onclick: move |_| paused.toggle(),
            if paused() {
                PlayIcon {}
            } else {
                PauseIcon {}
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
        Box { framework_sx: &CAROUSEL_INDICATORS_SX, states: strip_states,
            for index in low..=high {
                Box {
                    key: "{index}",
                    component: "button",
                    r#type: "button",
                    framework_sx: &CAROUSEL_INDICATOR_SX,
                    states: states()
                        .with(orientation.state_name(), true)
                        .with("current", index == current()),
                    aria_label: numbered(labels.indicator, index - low, high - low + 1),
                    aria_current: (index == current()).then(|| "true".to_string()),
                    // Roving: one tab stop for the whole strip. The
                    // arrows move the slide and the focus with it -
                    // leaving focus on a `tabindex="-1"` dot would
                    // strand the keyboard there.
                    id: indicator_id(&track_id(), index),
                    tabindex: if index == current() { "0" } else { "-1" },
                    onclick: move |_| nav.go_to(index),
                    onkeydown: move |event: Event<KeyboardData>| {
                        if has_shortcut_modifier(&event) {
                            return;
                        }
                        let Some(target) = indicator_key_target(event.key(), index, low, high, looping)
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

/// Where an arrow, `Home` or `End` on a dot goes; `None` for a key the strip
/// does not act on.
///
/// It wraps only when the carousel does. Wrapping here unconditionally made
/// the same key wrap through the dots and clamp through the track, so the two
/// disagreed about whether this carousel loops.
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

    /// Deliberately built from the geometry rather than from `align_shift`, so
    /// it can fail when the model is wrong - the round-trip test below shares
    /// the model and cannot.
    ///
    /// Six 100px slides, 20px gaps, three up: the viewport is 340px, the pitch
    /// 120px, the content 700px and the scrollable range 360px. Where slide `k`
    /// comes to rest follows from which of its edges meets which of the
    /// viewport's.
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

    /// The whole point of deriving the index from a ratio: neither the slide
    /// width nor the gap appears, so nothing has to be measured.
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

    /// A strip that fits entirely in its viewport has one position, and
    /// dividing by its zero span would otherwise be a `NaN` index.
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

    /// The props write `--lsx-carousel-*-override`, so the CSS has to read the
    /// overridable form. Reading the bare themed var compiles, renders, and
    /// silently ignores `per_view` and `gap` - which is what it did first.
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

    /// A shrink-to-fit parent - a flex item, a grid cell, the docs preview
    /// pane - would otherwise size the root from the *track's* max-content
    /// width. Horizontally that overflows and gets clamped back, so it fills
    /// by accident; vertically the widest slide's content is the whole width
    /// and the carousel collapses to a sliver. No test in this repo computes a
    /// layout, so this is the declaration, not the result.
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

    /// A mandatory snap pulls every programmatic write back to the slide it
    /// left, so a drag that keeps it on does not move the strip at all until
    /// it jumps a whole slide. The `dragging` arm shares its specificity with
    /// the orientation arms, so it only wins by coming after both.
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

    /// The leading clones are the tail and the trailing ones are the head, so
    /// a strip position either side of the seam maps back onto a real slide.
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

    /// Three-up needs three clones at each end, or the strip runs out of
    /// content before the seam is reached.
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

    /// A dot's ring is drawn outside the dot, so it has to contrast against
    /// the surface around it. A literal `background()` publishes the dot's own
    /// twin instead, and the ring on the current dot came out white on white.
    #[test]
    fn a_dot_does_not_publish_its_own_focus_contrast() {
        let css = Stylesheet::from(&CAROUSEL_INDICATOR_SX);

        assert!(
            !css.as_str().contains("--lsx-focus-contrast:"),
            "{}",
            css.as_str()
        );
    }

    /// The controls' colours come from the theme, and the ring colour the
    /// literal `white` used to publish is still published, from the glyph.
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

    /// The viewport clips at the track's own edge, so the ring is drawn inside
    /// it. Outset, it is there in the CSS and invisible on the page.
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

    /// Not looping is the same code with no clones, and has to stay a plain
    /// pass-through.
    #[test]
    fn without_clones_a_position_is_the_slide() {
        assert_eq!(strip_count(5, 0), 5);
        assert_eq!(real_for(3, 5, 0), 3);
        assert!(!is_clone(0, 5, 0));
        assert!(!is_clone(4, 5, 0));
    }

    /// Only the first controlled scroll is instant, and only one that moves:
    /// the settle that lowers `seam` never comes for a scroll to where the
    /// strip already is.
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

    /// No provider, no swaps: a plain carousel never takes the instant path
    /// through this door.
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
