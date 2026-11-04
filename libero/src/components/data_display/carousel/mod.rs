mod controls;
mod drag;
mod indicators;
mod slide;
mod state;
#[cfg(test)]
mod tests;
mod track;

use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use self::{
    controls::{CarouselControls, CarouselStatus, carousel_pause_button},
    drag::use_carousel_drag,
    indicators::CarouselIndicators,
    slide::carousel_slides,
    state::{CarouselSetup, CarouselView, index_range, use_carousel_state},
    track::carousel_track,
};
use crate::{
    components::{
        common::{
            HtmlTag, Input, Orientation, Part, States, Variables, base_props, input_from_str,
            parts_enum, responsive_var_sx, variables, with_own,
        },
        data_display::responsive_props::{and_sx, plain_var, var_sx},
        layout::{Box, use_box, use_scroll_area},
    },
    hooks::{use_element, use_focus_within, use_id, use_localization, use_responsive, use_theme},
    localization::fill,
    platform::page_task,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{CAROUSEL_GAP, CAROUSEL_PER_VIEW, CssVar, Responsive, SizeCss},
    utils::warn,
};

pub use crate::theme::CarouselAlign;

input_from_str!(CarouselAlign);
crate::sx::input::input_from_responsive!(f64);
crate::sx::input::input_from_responsive!(usize);

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

/// A caller's slide-and-`index` swap (`Lightbox`) moves instantly, not by a smooth
/// scroll that fetches lazy pictures (todo 323); a count, so it survives a re-render.
/// Also tells the track a slide holds the tab stop (todo 2833). The third field is the
/// last swap the track has jumped for; a swap reads as landed once it matches.
#[derive(Clone)]
pub(crate) struct CarouselJump(Rc<Cell<u64>>, Rc<Cell<bool>>, Signal<u64>);

impl Default for CarouselJump {
    fn default() -> Self {
        Self(Rc::default(), Rc::default(), Signal::new(0))
    }
}

impl PartialEq for CarouselJump {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl CarouselJump {
    /// Marks a swap, from the render that draws it.
    pub(crate) fn swapped(&self) {
        self.0.set(self.0.get() + 1);
    }

    /// Whether the last swap's jump is still to come; subscribes the render that reads it.
    pub(crate) fn jumping(&self) -> bool {
        (self.2)() != self.0.get()
    }

    /// Whether the showing slide is the tab stop, set from the render that draws the track.
    pub(crate) fn set_slide_stop(&self, stop: bool) {
        self.1.set(stop);
    }

    fn slide_stop(&self) -> bool {
        self.1.get()
    }

    fn count(&self) -> u64 {
        self.0.get()
    }

    fn landed(&self) -> Signal<u64> {
        self.2
    }
}

/// Per-instance only: no themed default, `auto` suits a horizontal strip.
const CAROUSEL_HEIGHT: CssVar = CssVar::new("--lsx-carousel-height");

fn carousel_variables(
    per_view: Option<String>,
    gap: &Input<Responsive<ThemeAwareValue>>,
    height: &Input<Responsive<ThemeAwareValue>>,
) -> Variables {
    variables()
        .with(CAROUSEL_PER_VIEW.override_var(), per_view)
        .with(
            CAROUSEL_GAP.override_var(),
            plain_var(gap, Some(SizeCss::SPACING)),
        )
        .with(CAROUSEL_HEIGHT, plain_var(height, Some(SizeCss::SPACING)))
}

/// The per-breakpoint `per_view`, `gap` and `height`, in the user layer under the caller's `sx`.
fn carousel_sx(props: &CarouselProps, per_views: &Responsive<f64>) -> Input<Sx> {
    let edges = per_views.map(|per_view| ThemeAwareValue::from(per_view.max(0.1).to_string()));
    let per_view = responsive_var_sx(&edges, CAROUSEL_PER_VIEW.override_var(), None);
    let gap = var_sx(
        &props.gap,
        CAROUSEL_GAP.override_var(),
        Some(SizeCss::SPACING),
    );
    let height = var_sx(&props.height, CAROUSEL_HEIGHT, Some(SizeCss::SPACING));
    with_own(and_sx(per_view, and_sx(gap, height)), &props.sx)
}

parts_enum! {
    /// [`Carousel`]'s inner parts. Direct paths, so a carousel in a slide keeps its own.
    pub enum CarouselPart {
        /// Track and controls; clips the strip.
        Viewport = "viewport" => "& > [data-slot='viewport']",
        /// The scrolling strip, the tab stop.
        Track = "track" => "& > [data-slot='viewport'] > [data-slot='track']",
        /// One slide; the current one also has `data-current="true"`.
        Slide = "slide" => "& > [data-slot='viewport'] > [data-slot='track'] > * > [data-slot='slide']",
        /// The strip holding previous and next.
        Controls = "controls" => "& > [data-slot='viewport'] > [data-slot='controls']",
        /// The previous or next button.
        Control = "control" => "& > [data-slot='viewport'] > [data-slot='controls'] > [data-slot='control']",
        Indicators = "indicators" => "& > [data-slot='indicators']",
        /// One dot.
        Indicator = "indicator" => "& > [data-slot='indicators'] > [data-slot='indicator']",
        /// The autoplay toggle.
        Pause = "pause" => "& > [data-slot='pause']",
    }
}

base_props! {
    parts(CarouselPart);
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
        /// Slides visible at once. Fractional peeks the next one. Per breakpoint too:
        /// `responsive(1.0).md(3.0)`; the widths are exact at once, while the status, the
        /// dots and the last stop of a server-rendered page follow after mount.
        #[props(default, into)]
        per_view: Input<Responsive<f64>>,
        /// Space between slides: a size word or any CSS, as `gap: "0"`.
        /// Per breakpoint too: `responsive(Size::Xs).md(Size::Lg)`.
        #[props(default, into)]
        gap: Input<Responsive<ThemeAwareValue>>,
        #[props(default, into)]
        align: Input<CarouselAlign>,
        /// `"horizontal"` by default, unlike `Orientation`'s own default. Per breakpoint
        /// too; a server-rendered page draws the base until mount.
        #[props(default, into)]
        orientation: Input<Responsive<Orientation>>,
        /// Required for a vertical carousel. Per breakpoint too:
        /// `responsive(ThemeAwareValue::from("240px")).md("400px".into())`.
        #[props(default, into)]
        height: Input<Responsive<ThemeAwareValue>>,
        /// Previous and Next; hidden while every slide fits.
        #[props(default)]
        controls: Option<bool>,
        #[props(default)]
        indicators: Option<bool>,
        /// Names the region; unset warns.
        #[props(default, into)]
        aria_label: Option<String>,
        /// Mouse drag-to-scroll over the track; touch already swipes. With `controls` and
        /// `indicators` both off, a drag is the only pointer way to move: it warns (WCAG 2.5.7).
        #[props(default)]
        draggable: bool,
        /// A drag or swipe rests where it ends instead of snapping to a slide; pair it with
        /// `draggable` for a mouse. Ignored with `loop`, whose strip re-seats on a slide.
        #[props(default)]
        drag_free: bool,
        /// Slides one control, arrow key or autoplay step moves; 1 by default.
        /// A looping strip caps it at the slides visible. Per breakpoint too:
        /// `responsive(1).md(3)`; a server-rendered page steps by the base until mount.
        #[props(default, into)]
        slides_to_scroll: Input<Responsive<usize>>,
        /// Advances on a timer, with a pause control (WCAG 2.2.2). Without `loop` it stops on the last slide.
        #[props(default)]
        autoplay: bool,
        /// Milliseconds between advances.
        #[props(default)]
        autoplay_delay: Option<u32>,
        /// Wraps at both ends through cloned slides. The clones repeat a slide's DOM:
        /// give interactive slide content no `id` or form `name`.
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

    let count = props.slides.len();
    let per_views = props.per_view.copied_or(theme.carousel.per_view.into());
    // The slide widths ride a var per breakpoint; the counts in Rust switch after mount.
    let per_view = use_responsive(&per_views)().max(0.1);
    let scroll_by = use_responsive(&props.slides_to_scroll.copied_or(1.into()))().max(1);
    // Every slide fits and a loop has none to wrap: status, track and dot would be dead stops.
    let quiet = count as f64 <= per_view && !(props.r#loop && count > 1);
    // `Orientation` defaults to vertical; a carousel does not. Resolved in Rust: the controls,
    // scrollbars and drag axis differ per orientation, so a responsive one switches after mount.
    let orientation =
        use_responsive(&props.orientation.copied_or(Orientation::Horizontal.into()))();
    let align = props.align.copied_or(theme.carousel.align);
    // No slides: no chrome, no landmark, no tab stop. The root keeps its box.
    let empty = count == 0;
    let wants_controls = props.controls.unwrap_or(theme.carousel.controls);
    let wants_indicators = props.indicators.unwrap_or(theme.carousel.indicators);
    // A quiet strip drops its controls too, as Mantine hides inactive ones (todo 4537).
    let controls = !empty && !quiet && wants_controls;
    let indicators = !empty && !quiet && wants_indicators;
    let (first, last) = index_range(count, per_view, align);
    use_hook(|| {
        if drags_alone(props.draggable, wants_controls, wants_indicators) {
            warn(
                "Carousel: `draggable` with no `controls` and no `indicators` leaves a drag as the only pointer way to move (WCAG 2.5.7). Turn one on.",
            );
        }
    });

    // Read in render, so a swap re-runs the controlled-index effect.
    let swaps = jump.as_ref().map(CarouselJump::count);

    // A blank name names nothing: it falls back to the localized one and still warns.
    let own_label = props
        .aria_label
        .clone()
        .filter(|label| !label.trim().is_empty());

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
        landed: jump.as_ref().map(CarouselJump::landed),
        r#loop: props.r#loop,
        autoplay: props.autoplay,
        autoplay_delay: props
            .autoplay_delay
            .unwrap_or(theme.carousel.autoplay_delay),
        named: own_label.is_some(),
        quiet,
        track_stop: !quiet && !jump.as_ref().is_some_and(CarouselJump::slide_stop),
        scroll_by,
        free: props.drag_free && !props.r#loop,
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
    let aria_label = own_label.unwrap_or_else(|| labels.label.to_string());

    let root_states: Input<States> = props
        .states
        .clone()
        .unwrap_or_default()
        .with(orientation.state_name(), true)
        .into();
    let variables: Input<Variables> = carousel_variables(
        per_views.is_plain().then(|| per_view.to_string()),
        &props.gap,
        &props.height,
    )
    .into();
    let sx = carousel_sx(&props, &per_views);

    // A focusout is decided a task later: a focusin in between is a move within the
    // root, no new entry (2393, WebView 2664), on every platform that reports the pair.
    let moves = use_signal(|| 0u64);
    let focus = use_focus_within(
        move || vec![root_handle.mounted()],
        move |change| {
            let (mut focused, mut moves) = (state.focused, moves);
            if change.within {
                moves += 1;
                focused.set(true);
                return;
            }
            let seen = *moves.peek();
            spawn(async move {
                page_task().await;
                if *moves.peek() == seen {
                    focused.set(false);
                }
            });
        },
    );

    let root = use_box()
        .framework_sx(&CAROUSEL_ROOT_SX)
        .class(&props.class)
        .sx(&sx)
        .parts(&props.parts)
        .states(&root_states)
        .variables(&variables)
        .prepare()
        .element(&root_handle)
        .attr("role", (!empty).then_some("region"))
        .attr(
            "aria-roledescription",
            (!empty).then_some(labels.roledescription),
        )
        .attr("aria-label", (!empty).then(|| aria_label.clone()))
        // Mouse only: a tap fires the compat mouseenter and would hold autoplay off.
        .event("onpointerenter", move |event: Event<PointerData>| {
            if event.data().pointer_type() == "mouse" {
                let mut hovered = state.hovered;
                hovered.set(true)
            }
        })
        .event("onpointerleave", move |event: Event<PointerData>| {
            if event.data().pointer_type() == "mouse" {
                let mut hovered = state.hovered;
                hovered.set(false)
            }
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
            Box { framework_sx: &CAROUSEL_VIEWPORT_SX, "data-slot": CarouselPart::Viewport.slot(),
                // First in Tab order, as APG's rotation control (todo 548); in the
                // viewport so it stays off the dot row (todo 3035).
                if state.rotates {
                    {carousel_pause_button(view, controls)}
                }
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

/// Whether a drag is the only way the pointer moves the strip: `controls` and `indicators` as drawn.
fn drags_alone(draggable: bool, controls: bool, indicators: bool) -> bool {
    draggable && !controls && !indicators
}

/// `template` with `{n}` one-based (read aloud) and `{m}` the count.
pub(crate) fn numbered(template: &str, index: usize, count: usize) -> String {
    fill(template, &[("n", &(index + 1)), ("m", &count)])
}
