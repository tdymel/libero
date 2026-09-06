use std::time::Duration;

use dioxus::prelude::*;

use crate::platform::{TimerSubscription, prefers_reduced_motion, timer, transition_property};

/// How long past the exit's own duration the fallback waits before it unmounts.
/// Late is harmless - the closed state is already out of the accessibility
/// tree - and early is the defect the property filter exists to prevent, so
/// this errs long. A browser delivers `transitionend` about 30ms after the
/// duration (630ms for a 600ms exit, measured 2026-09-16).
const EXIT_SLACK: Duration = Duration::from_millis(150);

/// Tracks the mount/visible lifecycle of an animated-open/close element.
#[derive(Clone, Copy)]
pub struct Presence {
    mounted: Signal<bool>,
    visible: Signal<bool>,
    /// A snapshot, not a signal: rebuilt every render, so never cache a
    /// `Presence` across renders - a stashed one answers with a frozen `open`.
    open: bool,
    property: &'static str,
}

impl Presence {
    pub fn mounted(&self) -> bool {
        (self.mounted)()
    }

    pub fn visible(&self) -> bool {
        (self.visible)()
    }

    /// Attach to the element's `onmounted`.
    pub fn on_mounted(&self) {
        if self.open {
            let mut visible = self.visible;
            visible.set(true);
        }
    }

    /// Attach to the element's `ontransitionend`.
    ///
    /// Filtered on `property`, because `transitionend` fires once per property
    /// and the shortest one finishes first: without this a 100ms opacity
    /// unmounts the content under a 600ms height still animating.
    ///
    /// A matching event **stops propagating here**. `transitionend` bubbles,
    /// so a nested presence element's exit would otherwise reach this one's
    /// ancestors and end their exit too - an inner `Collapse` unmounting its
    /// parent's content mid-close. Stopping it at the innermost handler means
    /// each one only ever sees its own element's end, on every renderer,
    /// without reading the event's target.
    pub fn on_transition_end(&self, event: &Event<TransitionData>) {
        // An unreadable property counts as a match, so it unmounts rather
        // than never - see `platform::transition_property` for which backends
        // that costs.
        if transition_property(event).is_some_and(|property| property != self.property) {
            return;
        }
        // Whether opening or closing: an ancestor mid-close must not take an
        // opening end for its own either.
        event.stop_propagation();
        if self.open {
            return;
        }

        let mut mounted = self.mounted;
        mounted.set(false);
    }
}

/// `property` is the CSS property carrying the **exit** transition -
/// `"grid-template-rows"` for a collapse, `"opacity"` for a fade.
///
/// **The closed state must also hide the content from the a11y tree** - a
/// `visibility: hidden` step (delayed by the duration, so it lands as the exit
/// ends) or `inert`. Between the close and the unmount the content is still
/// mounted, still focusable and still announced; a filtered exit makes that
/// window as long as the animation rather than as short as its quickest
/// property.
///
/// **An ancestor's own `ontransitionend` no longer sees this element's end
/// events for `property`**: [`Presence::on_transition_end`] stops them, which
/// is what keeps a nested exit from ending an outer one. Other properties
/// still bubble.
///
/// **Under `prefers-reduced-motion: reduce` a close unmounts at once**, because
/// the exit is expected to be switched off there and no `transitionend` would
/// ever arrive. A consumer that still animates under reduced motion loses its
/// exit.
///
/// `exit` is the exit's duration, the fallback for an exit that never reports
/// its end. `transitionend` does not fire for a zero duration, for
/// `transition: none`, or on a renderer that runs no transitions, and nothing
/// else would ever latch `mounted` back to `false`. With `exit` known the hook
/// unmounts at whichever comes first: the event, or `exit` plus 150ms of
/// slack. A zero `exit` unmounts at once. `None` waits for the event
/// alone.
pub fn use_presence(open: bool, property: &'static str, exit: Option<Duration>) -> Presence {
    let mut mounted = use_signal(|| open);
    // Not `false`: the first render is the one a server sends, and
    // mounted-without-visible is the closed markup.
    let mut visible = use_signal(|| open);
    // Dropping it cancels, so replacing or clearing it is the whole
    // cancellation story, and the scope's own drop covers an unmount.
    let mut fallback = use_hook(|| CopyValue::new(None::<Box<dyn TimerSubscription>>));

    use_effect(use_reactive!(|open, exit| {
        if open {
            fallback.set(None);
            if mounted() {
                visible.set(true);
            } else {
                mounted.set(true);
            }
            return;
        }

        visible.set(false);
        // `peek`: this arm must not re-run when its own latch lands.
        if !*mounted.peek() {
            return;
        }
        let delay = match exit {
            _ if prefers_reduced_motion() => Duration::ZERO,
            Some(exit) if exit.is_zero() => Duration::ZERO,
            Some(exit) => exit + EXIT_SLACK,
            None => return,
        };
        if delay.is_zero() {
            mounted.set(false);
            return;
        }
        // The callback runs outside every scope; `mounted` is owned by this
        // one, and the subscription dies with it, so it never outlives it.
        let latch = move || {
            let mut mounted = mounted;
            mounted.set(false);
        };
        fallback.set(timer().map(|timer| timer.after(delay, Box::new(latch))));
    }));

    Presence {
        mounted,
        visible,
        open,
        property,
    }
}
