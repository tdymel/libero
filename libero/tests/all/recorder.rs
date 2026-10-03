//! The shared recorder: reads the mutations a render emits, which is the only
//! way to address an element from outside the dom.

use dioxus::core::{AttributeValue, ElementId, WriteMutations};
use std::collections::{HashMap, HashSet};

/// Records the element the `click` listener landed on, which is the only way
/// to address it from a test - and every `mousedown` one, in order. Each
/// element's listeners, dynamic attributes and own text are kept for
/// [`FindClickListener::element`], and every listener and attribute write in order.
#[derive(Default)]
pub struct FindClickListener {
    pub last: Option<ElementId>,
    /// The element a dynamic attribute or a text lands on; `None` while a
    /// template is built, whose static attributes have no element yet.
    pub owner: Option<ElementId>,
    pub pushed: Option<ElementId>,
    pub attributes: HashMap<ElementId, HashMap<String, String>>,
    pub listeners: HashMap<ElementId, HashSet<String>>,
    pub click: Option<ElementId>,
    pub first_click: Option<ElementId>,
    pub clicks: Vec<ElementId>,
    pub input: Option<ElementId>,
    pub change: Option<ElementId>,
    pub keydown: Vec<ElementId>,
    pub mousedown: Vec<ElementId>,
    pub blur: Vec<ElementId>,
    pub transitionend: Vec<ElementId>,
    pub submit: Option<ElementId>,
    pub reset: Option<ElementId>,
    /// A field frame: its padding-press listeners act only on the web.
    pub frame: Option<ElementId>,
    /// Every listener registered, on the element it landed on, in order.
    pub registered: Vec<(ElementId, String)>,
    /// Every attribute write, its value as `Debug`, in order.
    pub writes: Vec<(String, String)>,
}

impl FindClickListener {
    /// The one element with a `listener` listener whose `attribute` is
    /// `value`, or whose own text is, for `"text"`. Panics unless exactly one,
    /// so a renamed label fails here and not as a refusal that passes.
    pub fn element(&self, listener: &str, attribute: &str, value: &str) -> ElementId {
        let found = self.matching(listener, attribute, value);
        match found[..] {
            [id] => id,
            _ => panic!(
                "{} elements with {attribute}={value:?} and a {listener} listener",
                found.len()
            ),
        }
    }

    /// Every element with a `listener` listener whose `attribute` is `value`.
    pub fn matching(&self, listener: &str, attribute: &str, value: &str) -> Vec<ElementId> {
        self.listeners
            .iter()
            .filter(|(_, names)| names.contains(listener))
            .map(|(id, _)| *id)
            .filter(|id| {
                self.attributes
                    .get(id)
                    .and_then(|attributes| attributes.get(attribute))
                    .is_some_and(|found| found == value)
            })
            .collect()
    }

    /// The elements a `name` listener registered on, in order.
    pub fn registered_for(&self, name: &str) -> Vec<ElementId> {
        self.registered
            .iter()
            .filter(|(_, registered)| registered == name)
            .map(|(id, _)| *id)
            .collect()
    }
}

impl WriteMutations for FindClickListener {
    fn push_id(&mut self, id: ElementId) {
        self.last = Some(id);
        self.owner = Some(id);
        self.pushed = Some(id);
    }
    fn set_id(&mut self, id: ElementId) {
        self.last = Some(id);
        self.owner = Some(id);
        self.pushed = None;
    }
    fn add_event_listener(&mut self, name: &str) {
        if let Some(id) = self.last {
            self.registered.push((id, name.to_string()));
        }
        if let Some(id) = self.owner {
            self.listeners
                .entry(id)
                .or_default()
                .insert(name.to_string());
        }
        if self.last.is_some() && self.last == self.frame {
            return;
        }
        if name == "click" {
            self.click = self.last;
            self.first_click = self.first_click.or(self.last);
            self.clicks.extend(self.last);
        }
        if name == "input" {
            self.input = self.last;
        }
        if name == "change" {
            self.change = self.last;
        }
        if name == "keydown" {
            self.keydown.extend(self.last);
        }
        if name == "mousedown" {
            self.mousedown.extend(self.last);
        }
        if name == "blur" {
            self.blur.extend(self.last);
        }
        if name == "transitionend" {
            self.transitionend.extend(self.last);
        }
        if name == "submit" {
            self.submit = self.last;
        }
        if name == "reset" {
            self.reset = self.last;
        }
    }
    fn child(&mut self, _index: usize) {}
    fn pop(&mut self) {
        self.pushed = None;
    }
    fn create_element(&mut self, _tag: &str, _ns: Option<&str>) {
        self.owner = None;
        self.pushed = None;
    }
    // A text is created right after its parent's `push_id`, then appended to it.
    fn create_text(&mut self, value: &str) {
        if let Some(id) = self.pushed {
            let attributes = self.attributes.entry(id).or_default();
            attributes.entry("text".into()).or_default().push_str(value);
        }
    }
    fn clone(&mut self) {
        self.owner = None;
        self.pushed = None;
    }
    fn append_children(&mut self, _m: usize) {}
    fn replace_with(&mut self, _m: usize) {}
    fn insert_after(&mut self, _m: usize) {}
    fn insert_before(&mut self, _m: usize) {}
    fn set_attribute(&mut self, name: &str, _ns: Option<&str>, value: &AttributeValue) {
        self.writes.push((name.to_string(), format!("{value:?}")));
        if name == "data-frame" {
            self.frame = self.last;
        }
        if let (Some(id), AttributeValue::Text(value)) = (self.owner, value) {
            let attributes = self.attributes.entry(id).or_default();
            attributes.insert(name.to_string(), value.clone());
        }
    }
    fn set_text(&mut self, _value: &str) {}
    fn remove_event_listener(&mut self, _name: &str) {}
    fn remove(&mut self) {}
}
