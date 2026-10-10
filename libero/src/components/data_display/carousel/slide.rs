use dioxus::prelude::*;

use super::{
    CarouselPart, numbered,
    state::{CarouselView, Nav, live_copy, outside_viewport},
};
use crate::{
    components::{
        common::{Input, Part, States, states},
        layout::Box,
    },
    platform::ElementApi,
    sx::{StaticSx, sx},
    theme::{CAROUSEL_GAP, CAROUSEL_PER_VIEW, CAROUSEL_RADIUS, FOCUS_RING_HALO_SPREAD},
};

pub(super) static CAROUSEL_SLIDE_SX: StaticSx = StaticSx::new(|| {
    sx()
        // `min-width: 0` or a slide refuses to shrink below its content.
        .min_width("0")
        .min_height("0")
        .border_radius(CAROUSEL_RADIUS.value())
        .overflow("hidden")
        // The slide and the track clip flush at its edges: room for an outset ring and its
        // halo on focusable content at any depth (todos 618, 619, 2860).
        .padding(FOCUS_RING_HALO_SPREAD.value())
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

/// The strip: the cloned tail, the slides, the cloned head (clones only when looping).
pub(super) fn carousel_slides(
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
                    roledescription: labels.slide_roledescription,
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
    roledescription: &'static str,
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
            "data-slot": CarouselPart::Slide.slot(),
            role: live.then_some("group"),
            aria_roledescription: live.then_some(roledescription),
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
