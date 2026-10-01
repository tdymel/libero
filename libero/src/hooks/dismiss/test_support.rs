//! The harness `tests.rs` drives dismiss layers with: a mutation recorder, Escape
//! presses and render helpers.

use std::rc::Rc;

use dioxus::core::{AttributeValue, ElementId, WriteMutations};
use dioxus::html::PlatformEventData;

use super::*;
use crate::test_converter::FakeEscape;

/// Every element that registered a `keydown` listener, in creation order
/// (outermost first).
#[derive(Default)]
pub(super) struct FindKeydownListeners {
    pub(super) last: Option<ElementId>,
    pub(super) keydown: Vec<ElementId>,
    pub(super) focusout: Vec<ElementId>,
    pub(super) mounted: Vec<ElementId>,
    /// `id="trigger"`, so a press can target it even without a listener of its own.
    pub(super) trigger: Option<ElementId>,
    /// `id="dropdown"`, the field-dropdown stand-in.
    pub(super) dropdown: Option<ElementId>,
    /// The first `role="combobox"`: a real `Select`'s trigger.
    pub(super) combobox: Option<ElementId>,
}

impl WriteMutations for FindKeydownListeners {
    fn push_id(&mut self, id: ElementId) {
        self.last = Some(id);
    }
    fn set_id(&mut self, id: ElementId) {
        self.last = Some(id);
    }
    fn add_event_listener(&mut self, name: &str) {
        match (name, self.last) {
            ("keydown", Some(id)) => self.keydown.push(id),
            ("focusout", Some(id)) => self.focusout.push(id),
            ("mounted", Some(id)) => self.mounted.push(id),
            _ => {}
        }
    }
    fn child(&mut self, _index: usize) {}
    fn pop(&mut self) {}
    fn create_element(&mut self, _tag: &str, _ns: Option<&str>) {}
    fn create_text(&mut self, _value: &str) {}
    fn clone(&mut self) {}
    fn append_children(&mut self, _m: usize) {}
    fn replace_with(&mut self, _m: usize) {}
    fn insert_after(&mut self, _m: usize) {}
    fn insert_before(&mut self, _m: usize) {}
    fn set_attribute(&mut self, n: &str, _ns: Option<&str>, v: &AttributeValue) {
        if n == "id"
            && let AttributeValue::Text(value) = v
        {
            match value.as_str() {
                "trigger" => self.trigger = self.last,
                "dropdown" => self.dropdown = self.last,
                _ => {}
            }
        }
        if n == "role"
            && self.combobox.is_none()
            && let AttributeValue::Text(value) = v
            && value == "combobox"
        {
            self.combobox = self.last;
        }
    }
    fn set_text(&mut self, _value: &str) {}
    fn remove_event_listener(&mut self, _name: &str) {}
    fn remove(&mut self) {}
}

pub(super) fn escape() -> Rc<dyn std::any::Any> {
    Rc::new(PlatformEventData::new(Box::new(FakeEscape::default())))
}

/// Just the app's marker line, cut at the next tag, so a failure stays readable.
pub(super) fn state(dom: &VirtualDom) -> String {
    let html = dioxus_ssr::render(dom);
    // The space keeps a `--lsx-*-on-state:` declaration from matching.
    let at = html.rfind("state: ").expect("the state marker");
    let marker = &html[at..];
    marker[..marker.find('<').unwrap_or(marker.len())].to_string()
}

/// Drains tasks and re-renders: a close goes task, signal, then the effect that
/// takes the layer off the stack.
pub(super) fn settle(dom: &mut VirtualDom) {
    for _ in 0..4 {
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
}

pub(super) fn press(dom: &mut VirtualDom, target: ElementId) {
    dom.runtime()
        .handle_event("keydown", Event::new(escape(), true), target);
    settle(dom);
}

pub(super) fn press_as(dom: &mut VirtualDom, target: ElementId, press: FakeEscape) {
    let data: Rc<dyn std::any::Any> = Rc::new(PlatformEventData::new(Box::new(press)));
    dom.runtime()
        .handle_event("keydown", Event::new(data, true), target);
    settle(dom);
}

pub(super) const HELD: FakeEscape = FakeEscape {
    repeat: true,
    composing: false,
    arrow_down: false,
};
pub(super) const COMPOSING: FakeEscape = FakeEscape {
    repeat: false,
    composing: true,
    arrow_down: false,
};
pub(super) const ARROW_DOWN: FakeEscape = FakeEscape {
    repeat: false,
    composing: false,
    arrow_down: true,
};

/// How many `<div>`s are open at `at`: tells siblings from nested elements.
pub(super) fn div_depth_at(html: &str, at: usize) -> i32 {
    let mut depth = 0;
    let mut rest = &html[..at];
    while let Some(next) = rest.find("<div") {
        depth += 1;
        rest = &rest[next + 4..];
    }
    let mut closes = 0;
    let mut rest = &html[..at];
    while let Some(next) = rest.find("</div>") {
        closes += 1;
        rest = &rest[next + 6..];
    }
    depth - closes
}
