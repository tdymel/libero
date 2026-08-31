use dioxus::prelude::*;

use crate::platform::transition_property;

/// Tracks the mount/visible lifecycle of an animated-open/close element.
#[derive(Clone, Copy)]
pub struct Presence {
    mounted: Signal<bool>,
    visible: Signal<bool>,
    /// The property whose transition ends the exit. See `on_transition_end`.
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
    pub fn on_mounted(&self, open: bool) {
        if open {
            let mut visible = self.visible;
            visible.set(true);
        }
    }

    /// Attach to the element's `ontransitionend`. Unmounts once the exit has
    /// actually finished.
    ///
    /// The property filter is the whole reason this takes the event. An
    /// element mid-close is usually transitioning more than one property, and
    /// `transitionend` fires per property - including for the zero-length ones
    /// used to drop something out of the focus order. Without the filter the
    /// first of them to finish unmounts the content underneath the animation
    /// that is still running.
    ///
    /// Where the platform cannot name the property this unmounts anyway,
    /// rather than never. Off the web nothing transitions, so refusing would
    /// strand the content mounted forever.
    pub fn on_transition_end(&self, open: bool, event: &Event<TransitionData>) {
        if open {
            return;
        }
        if transition_property(event).is_some_and(|property| property != self.property) {
            return;
        }

        let mut mounted = self.mounted;
        mounted.set(false);
    }
}

/// `property` is the CSS property carrying the exit transition -
/// `"grid-template-rows"` for a collapse, `"opacity"` for a fade. It is what
/// `on_transition_end` waits for.
pub fn use_presence(open: bool, property: &'static str) -> Presence {
    let mut mounted = use_signal(|| open);
    // Not `false`: an element that starts open has to *paint* open. Starting
    // hidden makes the first frame the closed one and plays the entry
    // animation against it - on every load and every hydration.
    let mut visible = use_signal(|| open);

    use_effect(use_reactive!(|open| {
        if open {
            if mounted() {
                visible.set(true);
            } else {
                mounted.set(true);
            }
        } else {
            visible.set(false);
        }
    }));

    Presence {
        mounted,
        visible,
        property,
    }
}
