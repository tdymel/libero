use dioxus::prelude::*;

/// Tracks the mount/visible lifecycle of an animated-open/close element.
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

    pub fn on_mounted(&self, open: bool) {
        if open {
            let mut visible = self.visible;
            visible.set(true);
        }
    }

    pub fn on_transition_end(&self, open: bool) {
        if !open {
            let mut mounted = self.mounted;
            mounted.set(false);
        }
    }
}

pub(crate) fn use_presence(open: bool) -> Presence {
    let mut mounted = use_signal(|| open);
    let mut visible = use_signal(|| false);

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

    Presence { mounted, visible }
}
