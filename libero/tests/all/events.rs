//! Events dispatched the way a renderer does: the payload is a
//! `PlatformEventData`, not the concrete event type. A listener built without
//! that conversion type-checks and passes an SSR test, but panics on the first
//! real click - see `BoxBuilder::event`.

use dioxus::core::{AttributeValue, ElementId, WriteMutations};
use dioxus::prelude::*;
use libero::{LiberoProvider, components::Box};

/// Every listener name registered during a rebuild, in order.
#[derive(Default)]
struct Listeners(Vec<String>);

impl WriteMutations for Listeners {
    fn add_event_listener(&mut self, name: &str) {
        self.0.push(name.to_string());
    }
    fn push_id(&mut self, _id: ElementId) {}
    fn set_id(&mut self, _id: ElementId) {}
    fn child(&mut self, _index: usize) {}
    fn pop(&mut self) {}
    fn create_element(&mut self, _tag: &str, _ns: Option<&str>) {}
    fn create_text(&mut self, _value: &str) {}
    fn clone(&mut self) {}
    fn append_children(&mut self, _m: usize) {}
    fn replace_with(&mut self, _m: usize) {}
    fn insert_after(&mut self, _m: usize) {}
    fn insert_before(&mut self, _m: usize) {}
    fn set_attribute(&mut self, _n: &str, _ns: Option<&str>, _v: &AttributeValue) {}
    fn set_text(&mut self, _value: &str) {}
    fn remove_event_listener(&mut self, _name: &str) {}
    fn remove(&mut self) {}
}

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
    let mut listeners = Listeners::default();
    dom.rebuild(&mut listeners);

    for name in ["input", "focus"] {
        assert!(
            listeners.0.iter().any(|listener| listener == name),
            "no {name} listener: {:?}",
            listeners.0
        );
    }
}
