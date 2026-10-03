//! Events dispatched the way a renderer does: the payload is a
//! `PlatformEventData`, not the concrete event type. A listener built without
//! that conversion type-checks and passes an SSR test, but panics on the first
//! real click - see `BoxBuilder::event`.

use crate::dispatch::FindClickListener;
use dioxus::prelude::*;
use libero::{LiberoProvider, components::Box};

/// What lets a field declare no `onfocus`, `onblur` or `onkeydown` and still
/// accept one: `base_props!` extends `GlobalAttributes`, which carries
/// listeners and not only attributes. `Button` declares `onclick` for its own
/// ripple reasons, so nothing else pins this down.
#[test]
fn global_attributes_carry_event_listeners_through_the_spread() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Box { oninput: move |_| {}, onfocus: move |_| {}, "x" }
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);

    for name in ["input", "focus"] {
        assert!(
            !find.registered_for(name).is_empty(),
            "no {name} listener: {:?}",
            find.registered
        );
    }
}
