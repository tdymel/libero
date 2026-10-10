use super::{
    CAROUSEL_ROOT_SX, CarouselAlign,
    controls::{CAROUSEL_CONTROL_SX, CAROUSEL_PAUSE_SX, showing_status},
    indicators::CAROUSEL_INDICATOR_SX,
    slide::CAROUSEL_SLIDE_SX,
    state::*,
    track::CAROUSEL_TRACK_SX,
};
use crate::{
    components::{CarouselPart, common::part_table},
    css::Stylesheet,
    localization::CarouselLabels,
    sx::REDUCED_MOTION,
};

/// The slot names are public: a rename here is a breaking change.
#[test]
fn the_part_table_is_stable() {
    assert_eq!(
        part_table::<CarouselPart>(),
        [
            ("viewport", "& > [data-slot='viewport']"),
            ("track", "& > [data-slot='viewport'] > [data-slot='track']"),
            (
                "slide",
                "& > [data-slot='viewport'] > [data-slot='track'] > * > [data-slot='slide']"
            ),
            (
                "controls",
                "& > [data-slot='viewport'] > [data-slot='controls']"
            ),
            (
                "control",
                "& > [data-slot='viewport'] > [data-slot='controls'] > [data-slot='control']"
            ),
            ("indicators", "& > [data-slot='indicators']"),
            (
                "indicator",
                "& > [data-slot='indicators'] > [data-slot='indicator']"
            ),
            ("pause", "& > [data-slot='pause']"),
        ]
    );
}

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

/// Todos 2829, 2927: five slides three-up centred, looping, mounted on slide 0 show
/// slides 5, 1 and 2; the range 5-2 would count down, so the status says it wraps.
#[test]
fn a_looping_window_across_the_seam_says_it_wraps() {
    let labels = CarouselLabels::ENGLISH;
    let nav_showing = |rest: usize| {
        let (clones, strip) = (3, 11);
        let (first, last) = showing(rest + clones, strip, 3.0, CarouselAlign::Center);
        (real_for(first, 5, clones), real_for(last, 5, clones))
    };

    assert_eq!(nav_showing(0), (4, 1));
    assert_eq!(
        showing_status(&labels, nav_showing(0), 5),
        "Slides 5 to 2 of 5, wrapping around"
    );
    assert_eq!(
        showing_status(&labels, nav_showing(4), 5),
        "Slides 4 to 1 of 5, wrapping around"
    );
    assert_eq!(
        showing_status(&CarouselLabels::GERMAN, nav_showing(0), 5),
        "Folien 5 bis 2 von 5, über das Ende hinaus"
    );
    // Windows that do not wrap keep their range.
    assert_eq!(
        showing_status(&labels, nav_showing(2), 5),
        "Slides 2–4 of 5"
    );
    assert_eq!(showing_status(&labels, (3, 3), 5), "Slide 4 of 5");
}

/// Todos 619, 2860: a slide leaves room for the whole ring, stripe and halo, of content
/// flush with its edge; the global reset's `border-box` keeps it inside the basis.
#[test]
fn a_slide_pads_by_the_focus_ring() {
    let css = Stylesheet::from(&CAROUSEL_SLIDE_SX);
    let css = css.as_str();

    assert!(
        css.contains("padding:var(--lsx-focus-ring-halo-spread)"),
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

/// Todo 2357: five slides looping three-up (three clones a side) wrap onto the
/// adjacent clone, never back across the strip.
#[test]
fn a_looping_step_wraps_onto_the_adjacent_clone() {
    assert_eq!(step_from(4, true, 5, 3, 1, 3), (0, 8));
    assert_eq!(step_from(0, false, 5, 3, 1, 3), (4, 2));
    assert_eq!(step_from(2, true, 5, 3, 1, 3), (3, 6));
    assert_eq!(step_from(2, false, 5, 3, 1, 3), (1, 4));
}

/// A plain strip stops at its reachable window.
#[test]
fn a_plain_step_stops_at_the_window() {
    assert_eq!(step_from(3, true, 6, 0, 0, 3), (3, 3));
    assert_eq!(step_from(1, false, 6, 0, 1, 4), (1, 1));
    assert_eq!(step_from(1, true, 6, 0, 0, 3), (2, 2));
}
