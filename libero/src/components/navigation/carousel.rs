use std::time::Duration;

use dioxus::prelude::*;

use crate::{
    components::{
        Box, HtmlTag, Input, Orientation, States, Variables, VisuallyHidden,
        common::{base_props, focus_ring_sx, input_from_str, states, variables},
        layout::use_box,
    },
    hooks::{
        DragMove, DragOptions, DragStart, ElementHandle, use_drag, use_element, use_id, use_theme,
    },
    platform::{ElementApi, TimerSubscription, timer},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        CAROUSEL_CONTROL_SIZE, CAROUSEL_CONTROLS_OFFSET, CAROUSEL_GAP, CAROUSEL_INDICATOR_COLOR,
        CAROUSEL_INDICATOR_CURRENT_COLOR, CAROUSEL_INDICATOR_CURRENT_LENGTH,
        CAROUSEL_INDICATOR_LENGTH, CAROUSEL_INDICATOR_THICKNESS, CAROUSEL_INDICATORS_GAP,
        CAROUSEL_PER_VIEW, CAROUSEL_RADIUS, CarouselDefaults, CssVar, Size, SizeCss,
    },
    utils::warn,
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

static CAROUSEL_TRACK_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        // `overridable`, not `value`: the props write the `-override` twin.
        .gap(CAROUSEL_GAP.overridable())
        .when(
            "horizontal",
            sx().flex_direction("row")
                .overflow_x("auto")
                .overflow_y("hidden"),
        )
        .when(
            "vertical",
            sx().flex_direction("column")
                .overflow_y("auto")
                .overflow_x("hidden"),
        )
        // The scrollbar is the indicators' job here; the strip still scrolls.
        .scrollbar_width("none")
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
        .media(
            "(prefers-reduced-motion: reduce)",
            sx().scroll_behavior("auto"),
        )
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
        .focus_visible(focus_ring_sx().outline_offset("-2px"))
});

static CAROUSEL_SLIDE_SX: StaticSx = StaticSx::new(|| {
    sx()
        // `min-width: 0` or a slide refuses to shrink below its content.
        .min_width("0")
        .min_height("0")
        .border_radius(CAROUSEL_RADIUS.value())
        .overflow("hidden")
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
        .background("white")
        .color("grey.7")
        .box_shadow(SizeCss::SHADOW.value(Size::Sm))
        .cursor("pointer")
        .hover(sx().background("grey.1"))
        // Disabled by `aria-disabled`, not `disabled`: the button keeps its
        // tab stop so focus is never dropped at either end.
        .when(
            "disabled",
            sx().opacity("0.4").cursor("default").background("white"),
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
        // Behind a var, not a literal: `background("grey.6")` would publish
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
        .media("(prefers-reduced-motion: reduce)", sx().transition("none"))
        .focus_visible(focus_ring_sx())
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
        .background("white")
        .color("grey.7")
        .box_shadow(SizeCss::SHADOW.value(Size::Sm))
        .cursor("pointer")
        .focus_visible(focus_ring_sx())
});

fn chevron(points: &'static str) -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            width: "60%",
            height: "60%",
            path { d: points }
        }
    }
}

fn pause_icon(paused: bool) -> Element {
    match paused {
        true => rsx! {
            svg { view_box: "0 0 24 24", fill: "currentColor", width: "55%", height: "55%",
                path { d: "M8 5v14l11-7z" }
            }
        },
        false => rsx! {
            svg { view_box: "0 0 24 24", fill: "currentColor", width: "55%", height: "55%",
                path { d: "M6 5h4v14H6zM14 5h4v14h-4z" }
            }
        },
    }
}

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

/// The keys the track acts on, so a slide can keep them.
///
/// A caller's `TextField` inside a slide is not code this component owns and
/// cannot be asked to stop propagating - and the library's own text inputs do
/// not. Without this, a bubble-phase handler calling `prevent_default` would
/// eat caret movement, Home/End and option selection from any focusable slide
/// content *and* advance the carousel underneath it. That is a direct
/// consequence of leaving offscreen slides reachable, so it is this
/// component's problem rather than the caller's.
///
/// Only the keys this orientation's track acts on are stopped. The cross-axis
/// arrows are not among them: the track lets those through untouched, so a
/// slide has to as well, or an ancestor would hear them from the track and not
/// from a button inside a slide.
fn carousel_key(key: &Key, orientation: Orientation) -> bool {
    match orientation {
        Orientation::Horizontal => {
            matches!(key, Key::ArrowLeft | Key::ArrowRight | Key::Home | Key::End)
        }
        Orientation::Vertical => {
            matches!(key, Key::ArrowUp | Key::ArrowDown | Key::Home | Key::End)
        }
    }
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

/// Scrolls the track so `index` is the snapped slide.
///
/// The reads are created here and awaited in the task: off the web each is a
/// round-trip ([[codebase/platform-api]]).
fn scroll_to_index(
    track: ElementHandle,
    index: usize,
    count: usize,
    per_view: f64,
    orientation: Orientation,
    align: CarouselAlign,
) {
    let (content, viewport) = (track.scroll_size(), track.dimensions());
    spawn(async move {
        let (Ok(content), Ok(viewport)) = (content.await, viewport.await) else {
            return;
        };
        let max = match orientation {
            Orientation::Horizontal => content.width - viewport.width,
            Orientation::Vertical => content.height - viewport.height,
        }
        .max(0.0);
        let offset = offset_for(index, max, count, per_view, align);
        let _ = match orientation {
            Orientation::Horizontal => track.scroll_to(offset, 0.0),
            Orientation::Vertical => track.scroll_to(0.0, offset),
        };
    });
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
    track: ElementHandle,
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

    /// The offset and scrollable range along this carousel's own axis.
    fn metrics(self, data: &ScrollData) -> (f64, f64) {
        match self.orientation {
            Orientation::Horizontal => (
                data.scroll_left(),
                (data.scroll_width() - data.client_width()).max(0) as f64,
            ),
            Orientation::Vertical => (
                data.scroll_top(),
                (data.scroll_height() - data.client_height()).max(0) as f64,
            ),
        }
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
        /// on hover and on focus within, per WCAG 2.2.2.
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
/// ```ignore
/// Carousel {
///     aria_label: "Product photos",
///     per_view: 3.0,
///     indicators: true,
///     slides: photos.iter().map(|p| rsx! { Image { src: "{p.url}", alt: "{p.alt}" } }).collect(),
/// }
/// ```
#[component]
pub fn Carousel(props: CarouselProps) -> Element {
    let theme = use_theme();
    let track = use_element();
    // The indicators sit outside the track, so the roving focus looks them up
    // from the root.
    let root_handle = use_element();
    let track_id = use_id();

    let count = props.slides.len();
    let per_view = props.per_view.copied_or(theme.carousel.per_view).max(0.1);
    // `Orientation` defaults to vertical; a carousel does not.
    let orientation = props.orientation.copied_or(Orientation::Horizontal);
    let align = props.align.copied_or(theme.carousel.align);
    let controls = props.controls.unwrap_or(theme.carousel.controls);
    let indicators = props.indicators.unwrap_or(theme.carousel.indicators);
    let (first, last) = index_range(count, per_view, align);

    // Two indices, deliberately: `current` follows the scroll frame by frame so
    // the indicators and the controls feel live, `settled` only moves when the
    // scroll comes to rest, so the live region does not read every frame.
    let mut current = use_signal(|| props.index.unwrap_or(0));
    let mut settled = use_signal(|| props.index.unwrap_or(0));

    let mut paused = use_signal(|| false);
    let mut hovered = use_signal(|| false);
    let mut focused = use_signal(|| false);
    let mut subscription = use_signal(|| None::<Box<dyn TimerSubscription>>);
    let mut dragging = use_signal(|| false);
    let mut seam = use_signal(|| false);
    let mut drag_origin = use_signal(|| 0.0_f64);

    // A named region beats an unnamed one even when the name is generic, so
    // the theme's stands in - and the warning still says to do better.
    if props.aria_label.is_none() {
        warn(
            "Carousel: no `aria_label`, falling back to the theme's. A region needs a name of its own to be told apart in a landmark list.",
        );
    }
    let aria_label = props
        .aria_label
        .clone()
        .unwrap_or_else(|| theme.carousel.label.to_string());

    // One clone per visible slide at each end, so the strip can scroll a full
    // viewport past either edge before the seam is crossed.
    let clones = match props.r#loop && count > 1 {
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
        onindexchange: props.onindexchange,
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

    // A looping strip opens on a clone unless it is put right: index 0 is
    // `clones` items in.
    use_effect(use_reactive!(|nav, clones| {
        if clones > 0 {
            nav.scroll_to_raw(nav.raw_for(*nav.current.peek()));
        }
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

    // A caller driving `index` from outside. The whole tuple is reactive, so
    // the closure is rebuilt whenever any of it changes rather than capturing
    // the first render's values.
    let controlled = props.index;
    use_effect(use_reactive!(|controlled, nav| {
        let Some(index) = controlled else {
            return;
        };
        // Through `nav`, like every other mover: a looping strip keeps the real
        // slide `clones` positions in, and the clamp has to be the same one
        // `go_to` uses or a controlled index is legal through one path and
        // silently trimmed through the other.
        let asked = index;
        let index = nav.clamp_index(asked);
        if index != *current.peek() {
            current.set(index);
            settled.set(index);
        }
        nav.scroll_to_raw(nav.raw_for(index));
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

    let running = props.autoplay && !paused() && !hovered() && !focused() && count > 1;
    let delay = props
        .autoplay_delay
        .unwrap_or(theme.carousel.autoplay_delay);
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
        let handle = timer.every(
            Duration::from_millis(delay.max(1) as u64),
            Box::new(move || {
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

    let onscroll = move |event: Event<ScrollData>| {
        let (offset, max) = nav.metrics(&event.data());
        let raw = index_at(offset, max, nav.strip_count(), per_view, align);
        let next = nav.real_for(raw);
        if next != *current.peek() {
            current.set(next);
        }
    };

    // Only a settled scroll moves `settled`, which is what the live region and
    // `onindexchange` read - a scroll in progress moves `current` alone.
    let onscrollend = move |event: Event<ScrollData>| {
        // A drag writes an instant scroll per pointer move, and each of those
        // ends too. None is a settle: treating them as one reported the index
        // mid-drag and, on a looping strip, jumped the seam out from under the
        // pointer on every move past half a slide. Releasing switches the snap
        // back on, and the scroll that settles the strip ends after this.
        if *dragging.peek() {
            return;
        }
        let (offset, max) = nav.metrics(&event.data());
        let raw = index_at(offset, max, nav.strip_count(), per_view, align);
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
    // and `carousel_key` below is why it does not have to.
    let onkeydown = move |event: Event<KeyboardData>| {
        let key = event.key();
        let (previous, next) = match orientation {
            Orientation::Horizontal => (Key::ArrowLeft, Key::ArrowRight),
            Orientation::Vertical => (Key::ArrowUp, Key::ArrowDown),
        };
        let wraps = nav.clones > 0;
        let target = match key {
            key if key == previous => match (wraps, current()) {
                (true, 0) => count.saturating_sub(1),
                (_, index) => index.saturating_sub(1).max(first),
            },
            key if key == next => match wraps && current() >= count.saturating_sub(1) {
                true => 0,
                false => (current() + 1).min(last),
            },
            Key::Home => match wraps {
                true => 0,
                false => first,
            },
            Key::End => match wraps {
                true => count.saturating_sub(1),
                false => last,
            },
            _ => return,
        };
        // Bubble phase is enough: a default action is preventable from any
        // phase, and no slide stops propagation. Without this the native
        // scroll runs as well and lands between two snap points.
        event.prevent_default();
        nav.go_to(target);
    };

    // Mouse drag-to-scroll. Deliberately **not** given `drag_handle_sx()`:
    // that is `touch-action: none`, and it would take away the native touch
    // swipe that is the whole reason this is a scroll container. So a finger
    // scrolls the platform's way and a mouse drags, on every backend.
    let draggable = props.draggable;
    let drag = use_drag(DragOptions {
        capture: track,
        on_start: Callback::new(move |start: DragStart| {
            if !draggable {
                start.cancel.call(());
                return;
            }
            dragging.set(true);
            let offset = track.scroll_offset();
            spawn(async move {
                if let Ok((x, y)) = offset.await {
                    drag_origin.set(match orientation {
                        Orientation::Horizontal => x,
                        Orientation::Vertical => y,
                    });
                }
            });
        }),
        on_move: Callback::new(move |moved: DragMove| {
            let delta = moved.delta();
            let target = match orientation {
                Orientation::Horizontal => drag_origin() - delta.x,
                Orientation::Vertical => drag_origin() - delta.y,
            }
            .max(0.0);
            let _ = match orientation {
                Orientation::Horizontal => track.scroll_to(target, 0.0),
                Orientation::Vertical => track.scroll_to(0.0, target),
            };
        }),
        // No settle of our own: releasing hands the strip back to the
        // browser, which snaps and fires `onscrollend`.
        on_end: Callback::new(move |()| dragging.set(false)),
    });

    let slide_label = props.slide_label;
    let label_for = move |index: usize| match &slide_label {
        Some(label) => label.call(index),
        None => CarouselDefaults::format_label(theme.carousel.slide_label, index, count),
    };

    let root_states: Input<States> = props
        .states
        .clone()
        .unwrap_or_default()
        .with(orientation.state_name(), true)
        .into();
    let track_states: Input<States> = states()
        .with(orientation.state_name(), true)
        .with("dragging", dragging())
        // The seam jump has to be instant, or the strip visibly rewinds.
        .with("seam", seam())
        .into();
    let variables: Input<Variables> =
        carousel_variables(per_view, props.gap.as_ref().copied(), props.height.as_ref()).into();

    let status = CarouselDefaults::format_label(theme.carousel.status_label, settled(), count);

    // The strip: the tail cloned onto the front, the slides, the head cloned
    // onto the back. Without looping the clones are empty and this is just the
    // slides.
    let strip: Vec<(usize, Element, bool)> = {
        let slides = &props.slides;
        let mut strip = Vec::with_capacity(nav.strip_count());
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
    };

    let track_body = strip
        .into_iter()
        .enumerate()
        .map(|(position, (index, slide, is_clone))| {
            // No `current` token: nothing in `CAROUSEL_SLIDE_SX` styles one,
            // and `data-current` below is what a caller actually reads.
            let slide_states: Input<States> = states().with(align.state_name(), true).into();
            rsx! {
                Box {
                    key: "{position}",
                    framework_sx: &CAROUSEL_SLIDE_SX,
                    states: slide_states,
                    role: if is_clone { None } else { Some("group") },
                    aria_roledescription: if is_clone { None } else { Some("slide") },
                    aria_label: if is_clone { None } else { Some(label_for(index)) },
                    // A clone is the same content twice over, so it is hidden
                    // rather than announced a second time.
                    aria_hidden: is_clone.then(|| "true".to_string()),
                    onkeydown: move |event: Event<KeyboardData>| {
                        if carousel_key(&event.key(), orientation) {
                            event.stop_propagation();
                        }
                    },
                    "data-current": (!is_clone && index == current()).then_some("true"),
                    {slide}
                }
            }
        });

    let track_element = use_box()
        .framework_sx(&CAROUSEL_TRACK_SX)
        .states(&track_states)
        .prepare()
        .element(&track)
        .event("onkeydown", onkeydown)
        .attr("id", track_id())
        // The track is the scrollable region, so it is the tab stop - the
        // opposite of `ScrollArea`'s default, and on purpose.
        .attr("tabindex", "0")
        .event("onscroll", onscroll)
        .event("onscrollend", onscrollend)
        // Mouse only: a touch is already scrolling the track natively, and
        // dragging it too would move it twice.
        .event("onpointerdown", move |event: Event<PointerData>| {
            if draggable && event.data().pointer_type() == "mouse" {
                drag.onpointerdown.call(event);
            }
        })
        .event("onpointermove", drag.onpointermove)
        .event("onpointerup", drag.onpointerup)
        .event("onpointercancel", drag.onpointercancel)
        .render(HtmlTag::Div, Vec::new(), rsx! { {track_body} })?;

    let root = use_box()
        .framework_sx(&CAROUSEL_ROOT_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&root_states)
        .variables(&variables)
        .prepare()
        .element(&root_handle)
        .attr("role", "region")
        .attr("aria-roledescription", "carousel")
        .attr("aria-label", aria_label)
        .event("onmouseenter", move |_: Event<MouseData>| hovered.set(true))
        .event("onmouseleave", move |_: Event<MouseData>| {
            hovered.set(false)
        })
        .event("onfocusin", move |_: Event<FocusData>| focused.set(true))
        .event("onfocusout", move |_: Event<FocusData>| focused.set(false));

    let track_id_value = track_id();
    // A looping carousel has no ends, so its controls never disable and its
    // indicator strip is one dot per real slide rather than one per scroll
    // position.
    let looping = clones > 0;
    let at_start = !looping && current() <= first;
    let at_end = !looping && current() >= last;
    // One dot per reachable position, which under a centred or end alignment
    // does not start at zero.
    let (low, high) = match looping {
        true => (0, count.saturating_sub(1)),
        false => (first, last),
    };
    let control_states = |disabled: bool| -> Input<States> {
        states()
            .with(orientation.state_name(), true)
            .with("disabled", disabled)
            .into()
    };
    let strip_states: Input<States> = states().with(orientation.state_name(), true).into();

    rsx! {
        {root.render(HtmlTag::Section, props.attributes, rsx! {
            VisuallyHidden {
                role: "status",
                // Off while it rotates on its own: an unattended change is not
                // worth interrupting a screen reader for, and it becomes
                // `polite` the moment the rotation stops. WCAG 2.2.2.
                aria_live: if running { "off" } else { "polite" },
                aria_atomic: "true",
                "{status}"
            }
            Box { framework_sx: &CAROUSEL_VIEWPORT_SX,
                {track_element}
                if controls {
                    Box { framework_sx: &CAROUSEL_CONTROLS_SX, states: strip_states.clone(),
                        Box {
                            component: "button",
                            r#type: "button",
                            framework_sx: &CAROUSEL_CONTROL_SX,
                            states: control_states(at_start),
                            aria_controls: track_id_value.clone(),
                            aria_disabled: at_start.to_string(),
                            aria_label: theme.carousel.previous_label,
                            onclick: move |_| match (looping, current()) {
                                (true, 0) => nav.go_to(count.saturating_sub(1)),
                                (_, index) if !at_start => nav.go_to(index.saturating_sub(1)),
                                _ => {}
                            },
                            {chevron(match orientation {
                                Orientation::Horizontal => "m15 18-6-6 6-6",
                                Orientation::Vertical => "m18 15-6-6-6 6",
                            })}
                        }
                        Box {
                            component: "button",
                            r#type: "button",
                            framework_sx: &CAROUSEL_CONTROL_SX,
                            states: control_states(at_end),
                            aria_controls: track_id_value.clone(),
                            aria_disabled: at_end.to_string(),
                            aria_label: theme.carousel.next_label,
                            onclick: move |_| match looping && current() >= count.saturating_sub(1) {
                                true => nav.go_to(0),
                                false if !at_end => nav.go_to(current() + 1),
                                false => {}
                            },
                            {chevron(match orientation {
                                Orientation::Horizontal => "m9 18 6-6-6-6",
                                Orientation::Vertical => "m6 9 6 6 6-6",
                            })}
                        }
                    }
                }
            }
            if props.autoplay {
                Box {
                    component: "button",
                    r#type: "button",
                    framework_sx: &CAROUSEL_PAUSE_SX,
                    aria_label: theme.carousel.pause_label,
                    aria_pressed: paused().to_string(),
                    onclick: move |_| paused.toggle(),
                    {pause_icon(paused())}
                }
            }
            if indicators {
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
                            aria_label: CarouselDefaults::format_label(
                                theme.carousel.indicator_label,
                                index,
                                count,
                            ),
                            aria_current: (index == current()).then(|| "true".to_string()),
                            // Roving: one tab stop for the whole strip. The
                            // arrows move the slide and the focus with it -
                            // leaving focus on a `tabindex="-1"` dot would
                            // strand the keyboard there.
                            id: indicator_id(&track_id(), index),
                            tabindex: if index == current() { "0" } else { "-1" },
                            onclick: move |_| nav.go_to(index),
                            onkeydown: move |event: Event<KeyboardData>| {
                                // Wraps only when the carousel does. Wrapping
                                // here unconditionally made the same key wrap
                                // through the dots and clamp through the
                                // track, so the two disagreed about whether
                                // this carousel loops.
                                let target = match event.key() {
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
                                    _ => return,
                                };
                                event.prevent_default();
                                nav.go_to(target);
                                focus_indicator(root_handle, &track_id(), target);
                            },
                        }
                    }
                }
            }
        })?}
    }
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
        let guard = css
            .find("(prefers-reduced-motion: reduce)")
            .expect("a reduced-motion block");

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

    /// A horizontal track ignores the vertical arrows, so a slide must not
    /// swallow them either - and the other way round.
    #[test]
    fn a_slide_keeps_only_the_keys_its_track_acts_on() {
        let (h, v) = (Orientation::Horizontal, Orientation::Vertical);

        assert!(carousel_key(&Key::ArrowRight, h));
        assert!(carousel_key(&Key::Home, h));
        assert!(!carousel_key(&Key::ArrowDown, h));
        assert!(carousel_key(&Key::ArrowDown, v));
        assert!(carousel_key(&Key::End, v));
        assert!(!carousel_key(&Key::ArrowRight, v));
        assert!(!carousel_key(&Key::Enter, h));
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

    /// Not looping is the same code with no clones, and has to stay a plain
    /// pass-through.
    #[test]
    fn without_clones_a_position_is_the_slide() {
        assert_eq!(strip_count(5, 0), 5);
        assert_eq!(real_for(3, 5, 0), 3);
        assert!(!is_clone(0, 5, 0));
        assert!(!is_clone(4, 5, 0));
    }
}
