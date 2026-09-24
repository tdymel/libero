use std::time::Duration;

use dioxus::prelude::*;

use super::CarouselAlign;
use crate::{
    components::{
        common::{Orientation, use_name_warning},
        layout::ScrollAreaHandle,
    },
    hooks::ElementHandle,
    localization::CarouselLabels,
    platform::{TimerSubscription, prefers_reduced_motion, timer},
};

/// The snappable indices, inclusive: six slides three-up reach 0..=3 at start,
/// 1..=4 centred and 2..=5 end-aligned, since the browser clamps the scroll.
pub(super) fn index_range(count: usize, per_view: f64, align: CarouselAlign) -> (usize, usize) {
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
pub(super) fn snap_position(
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
pub(super) fn outside_viewport(
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
pub(super) fn showing(
    rest: usize,
    strip: usize,
    per_view: f64,
    align: CarouselAlign,
) -> (usize, usize) {
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
pub(super) fn live_copy(
    index: usize,
    count: usize,
    clones: usize,
    shows: impl Fn(usize) -> bool,
) -> usize {
    let real = index + clones;
    let leading = (index + clones >= count).then(|| index + clones - count);
    let trailing = (index < clones).then(|| clones + count + index);
    [Some(real), leading, trailing]
        .into_iter()
        .flatten()
        .find(|&position| shows(position))
        .unwrap_or(real)
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
pub(super) fn index_at(
    offset: f64,
    max: f64,
    count: usize,
    per_view: f64,
    align: CarouselAlign,
) -> usize {
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
pub(super) fn offset_for(
    index: usize,
    max: f64,
    count: usize,
    per_view: f64,
    align: CarouselAlign,
) -> f64 {
    let span = count as f64 - per_view;
    if max <= 0.0 || span <= 0.0 {
        return 0.0;
    }
    ((index as f64 - align_shift(per_view, align)) / span * max).clamp(0.0, max)
}

/// The slides plus the clones at both ends (zero when not looping).
pub(super) fn strip_count(count: usize, clones: usize) -> usize {
    count + 2 * clones
}

/// The real slide a strip position shows: leading clones are the tail, trailing the head.
pub(super) fn real_for(raw: usize, count: usize, clones: usize) -> usize {
    if count == 0 {
        return 0;
    }
    (raw as isize - clones as isize).rem_euclid(count as isize) as usize
}

/// Whether a strip position is a cloned end: the seam has to be crossed.
pub(super) fn is_clone(raw: usize, count: usize, clones: usize) -> bool {
    clones > 0 && (raw < clones || raw >= clones + count)
}

/// Whether a controlled scroll is instant: the first after mount, or one with a
/// slide swap. Only if it moves, or a raised `seam` never comes down.
pub(super) fn instant_scroll(
    first_scroll: bool,
    swapped: bool,
    clones: usize,
    index: usize,
    from: usize,
    first: usize,
) -> bool {
    (first_scroll && (clones > 0 || index != first)) || (swapped && index != from)
}

/// Whether a swap arrived since last asked; reading consumes it, or every later
/// move would be instant too.
pub(super) fn consume_swap(seen: &mut Option<u64>, swaps: Option<u64>) -> bool {
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

/// Everything a move needs, rebuilt each render and copied into handlers: a
/// `use_callback` would keep the first render's `count` and `per_view`.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Nav {
    pub(super) current: Signal<usize>,
    pub(super) settled: Signal<usize>,
    pub(super) seam: Signal<bool>,
    pub(super) track: ScrollAreaHandle,
    pub(super) count: usize,
    pub(super) per_view: f64,
    pub(super) orientation: Orientation,
    pub(super) align: CarouselAlign,
    pub(super) first: usize,
    pub(super) last: usize,
    pub(super) onindexchange: Option<EventHandler<usize>>,
    /// Slides cloned onto each end of a looping strip; zero otherwise.
    pub(super) clones: usize,
}

impl Nav {
    pub(super) fn strip_count(self) -> usize {
        strip_count(self.count, self.clones)
    }

    /// A real slide's position in the strip.
    pub(super) fn raw_for(self, index: usize) -> usize {
        index + self.clones
    }

    pub(super) fn real_for(self, raw: usize) -> usize {
        real_for(raw, self.count, self.clones)
    }

    pub(super) fn is_clone(self, raw: usize) -> bool {
        is_clone(raw, self.count, self.clones)
    }

    /// Clamps to the real slides when looping, else to where scrolling stops.
    pub(super) fn clamp_index(self, index: usize) -> usize {
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

    pub(super) fn go_to(mut self, index: usize) {
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
    pub(super) fn cross_seam(mut self) {
        self.seam.set(true);
    }

    /// The first and last real slide at least half showing while the strip
    /// rests on `index`. A looping strip can wrap: `from` after `to`.
    pub(super) fn showing(self, index: usize) -> (usize, usize) {
        let (first, last) = showing(
            self.raw_for(index),
            self.strip_count(),
            self.per_view,
            self.align,
        );
        (self.real_for(first), self.real_for(last))
    }

    /// The strip position a scroll report (in percent) names.
    pub(super) fn raw_at(self, x: f64, y: f64) -> usize {
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

/// What `Carousel`'s state hook reads, as one reactive effect argument.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct CarouselSetup {
    pub(super) track: ScrollAreaHandle,
    pub(super) count: usize,
    pub(super) per_view: f64,
    pub(super) orientation: Orientation,
    pub(super) align: CarouselAlign,
    pub(super) first: usize,
    pub(super) last: usize,
    pub(super) onindexchange: Option<EventHandler<usize>>,
    /// The caller's `index`, when one drives the carousel from outside.
    pub(super) controlled: Option<usize>,
    /// `CarouselJump`'s swap counter, or `None` without a provider.
    pub(super) swaps: Option<u64>,
    pub(super) r#loop: bool,
    pub(super) autoplay: bool,
    pub(super) autoplay_delay: u32,
    /// Whether the caller named the region, for the name warning.
    pub(super) named: bool,
    /// Every slide fits under [`CarouselQuietWhenFits`]: no status, no track tab stop.
    pub(super) quiet: bool,
}

/// `Carousel`'s live state: `nav`, what pauses autoplay, and the drag.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct CarouselState {
    pub(super) nav: Nav,
    pub(super) paused: Signal<bool>,
    pub(super) hovered: Signal<bool>,
    pub(super) focused: Signal<bool>,
    /// A pointer is down on the pause control, so the focus it brings is no entry.
    pub(super) pressing: Signal<bool>,
    pub(super) dragging: Signal<bool>,
    pub(super) drag_origin: Signal<f64>,
    /// The controlled index the effect last applied. Until it runs, a new one
    /// stands in for `settled` when the slides go `inert` - see `carousel_slides`.
    pub(super) applied: Signal<Option<usize>>,
    /// Autoplay is on and nothing is holding it back.
    pub(super) running: bool,
}

/// Everything the carousel's chrome needs: setup, state, strings and ids.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct CarouselView {
    pub(super) setup: CarouselSetup,
    pub(super) state: CarouselState,
    pub(super) labels: &'static CarouselLabels,
    pub(super) track_id: Signal<String>,
    pub(super) status_id: Signal<String>,
    pub(super) root: ElementHandle,
}

/// The carousel's state and every effect that moves the strip.
pub(super) fn use_carousel_state(setup: CarouselSetup) -> CarouselState {
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
