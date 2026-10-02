use std::time::Duration;

use dioxus::prelude::*;

use crate::platform::{prefers_reduced_motion, transition_property};

use super::timers::use_scheduled;

/// How long past the exit's duration the fallback waits to unmount. Errs long:
/// browsers send `transitionend` ~30ms late (measured 2026-09-16).
const EXIT_SLACK: Duration = Duration::from_millis(150);

/// Tracks the mount/visible lifecycle of an animated-open/close element.
#[derive(Clone, Copy)]
pub(crate) struct Presence {
    mounted: Signal<bool>,
    visible: Signal<bool>,
    /// A snapshot: never cache a `Presence` across renders.
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
        if self.open && !*self.visible.peek() {
            let mut visible = self.visible;
            visible.set(true);
        }
    }

    /// Attach to the element's `ontransitionend`. Filtered on `property` (the
    /// shortest one ends first); a match stops propagating, so no nested exit ends an outer one.
    pub fn on_transition_end(&self, event: &Event<TransitionData>) {
        // Unreadable counts as a match: unmount rather than never
        // (`platform::transition_property` lists the backends).
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

/// Keeps closing content mounted until its exit transition on `property`
/// (`"opacity"` for a fade) ends, or `exit` plus [`EXIT_SLACK`]; `None` waits.
///
/// The closed state must hide the content from the a11y tree itself (`inert`, or
/// a delayed `visibility: hidden`). Reduced motion unmounts at once.
pub(crate) fn use_presence(open: bool, property: &'static str, exit: Option<Duration>) -> Presence {
    let mut mounted = use_signal(|| open);
    // Not `false`: the first render is the one a server sends, and
    // mounted-without-visible is the closed markup.
    let mut visible = use_signal(|| open);
    let fallback = use_scheduled(move |_| mounted.set(false));

    // A write re-renders even when unchanged: every presence mounted twice (todo 2031).
    use_effect(use_reactive!(|open, exit| {
        if open {
            fallback.cancel();
            if !mounted() {
                mounted.set(true);
            } else if !*visible.peek() {
                visible.set(true);
            }
            return;
        }

        if *visible.peek() {
            visible.set(false);
        }
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
        fallback.after(delay.as_millis() as u64);
    }));

    Presence {
        mounted,
        visible,
        open,
        property,
    }
}
