use dioxus::prelude::*;

/// Tracks the DOM lifecycle of an element that animates in/out instead of
/// just appearing/disappearing: `mounted` says whether it should be rendered
/// at all, `visible` drives the "open" data-state that triggers the CSS
/// transition. The element stays mounted for the duration of its exit
/// transition (so it has something to animate from) and is only actually
/// removed once that transition finishes - see `on_transition_end`.
#[derive(Clone, Copy)]
pub(crate) struct Presence {
    mounted: Signal<bool>,
    visible: Signal<bool>,
}

impl Presence {
    pub fn mounted(&self) -> bool {
        (self.mounted)()
    }

    pub fn visible(&self) -> bool {
        (self.visible)()
    }

    /// Attach to the transitioning element's `onmounted`: flips it visible
    /// once it has actually landed in the DOM, so an entering element renders
    /// closed for one frame first and has an opacity/transform value to
    /// transition from instead of just appearing at its end state.
    pub fn on_mounted(&self, open: bool) {
        if open {
            let mut visible = self.visible;
            visible.set(true);
        }
    }

    /// Attach to the transitioning element's `ontransitionend`: removes it
    /// from the DOM once its exit transition has actually finished playing.
    pub fn on_transition_end(&self, open: bool) {
        if !open {
            let mut mounted = self.mounted;
            mounted.set(false);
        }
    }
}

/// Drives a [`Presence`] from a plain `open: bool` prop (as opposed to a
/// signal), handling the `use_reactive` dance that's needed for the effect to
/// actually rerun when that prop changes on an already-mounted component.
pub(crate) fn use_presence(open: bool) -> Presence {
    let mut mounted = use_signal(|| open);
    let mut visible = use_signal(|| false);

    use_effect(use_reactive!(|open| {
        if open {
            if mounted() {
                // Already in the DOM (re-opened before the exit transition
                // finished) - flip immediately, no insertion to wait for.
                visible.set(true);
            } else {
                // `on_mounted` flips `visible` once the element actually
                // lands in the DOM.
                mounted.set(true);
            }
        } else {
            visible.set(false);
        }
    }));

    Presence { mounted, visible }
}
