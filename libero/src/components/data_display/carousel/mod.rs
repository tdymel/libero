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
            parts_enum, variables,
        },
        layout::{Box, use_box, use_scroll_area},
    },
    hooks::{use_element, use_focus_within, use_id, use_localization, use_theme},
    localization::fill,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{CAROUSEL_GAP, CAROUSEL_PER_VIEW, CssVar, Size, SizeCss},
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
        .parts(&props.parts)
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
            if state.rotates {
                {carousel_pause_button(view, controls)}
            }
            Box { framework_sx: &CAROUSEL_VIEWPORT_SX, "data-slot": CarouselPart::Viewport.slot(),
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

/// `template` with `{n}` one-based (read aloud) and `{m}` the count.
pub(crate) fn numbered(template: &str, index: usize, count: usize) -> String {
    fill(template, &[("n", &(index + 1)), ("m", &count)])
}
