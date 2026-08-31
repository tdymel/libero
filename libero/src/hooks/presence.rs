use dioxus::prelude::*;

use crate::platform::transition_property;

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
    pub fn on_transition_end(&self, event: &Event<TransitionData>) {
        if self.open {
            return;
        }
        // An unreadable property unmounts rather than never - see
        // `platform::transition_property` for which backends that costs.
        if transition_property(event).is_some_and(|property| property != self.property) {
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
pub fn use_presence(open: bool, property: &'static str) -> Presence {
    let mut mounted = use_signal(|| open);
    // Not `false`: the first render is the one a server sends, and
    // mounted-without-visible is the closed markup.
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
        open,
        property,
    }
}
