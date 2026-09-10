//! A headless Blitz harness: one libero component in a windowless
//! `DioxusDocument`, driven through Blitz's own event pipeline.
//!
//! Shaped after `.blitz-fork`'s `blitz-test-harness`, which cannot be a
//! dependency: it builds against its own dioxus, not `.dioxus-active`.
//!
//! ```no_run
//! use dioxus::prelude::*;
//! use native_tests::mount;
//!
//! fn app() -> Element {
//!     rsx! { button { id: "go", "Go" } }
//! }
//!
//! let mut page = mount(app);
//! page.click("#go");
//! assert!(page.is_focused("#go"));
//! ```

use std::{sync::Arc, thread, time::Duration};

use blitz_dom::{BaseDocument, Document, DocumentConfig, Node};
use blitz_traits::{
    NodeId,
    events::{
        BlitzKeyEvent, BlitzPointerEvent, BlitzPointerId, KeyState, MouseEventButton,
        MouseEventButtons, Point, PointerCoords, PointerDetails, UiEvent,
    },
    shell::{ColorScheme, Viewport},
};
use dioxus::prelude::*;
use dioxus_native_dom::DioxusDocument;
use libero::LiberoProvider;

pub use dioxus::prelude::{Code, Key, Location, Modifiers};

/// The viewport every test renders into, CSS pixels at scale 1.
pub const VIEWPORT: (u32, u32) = (1024, 768);

// Bounds `settle`, so a render loop fails the test instead of hanging it.
const MAX_POLLS: usize = 200;

/// Mounts `app` inside a [`LiberoProvider`] and settles the first render.
pub fn mount(app: fn() -> Element) -> Page {
    let vdom = VirtualDom::new_with_props(Root, RootProps { app: App(app) });
    let mut doc = DioxusDocument::new(
        vdom,
        DocumentConfig {
            viewport: Some(Viewport::new(
                VIEWPORT.0,
                VIEWPORT.1,
                1.0,
                ColorScheme::Light,
            )),
            html_parser_provider: Some(Arc::new(blitz_html::HtmlProvider)),
            ..Default::default()
        },
    );
    doc.initial_build();
    let mut page = Page { doc, time: 0.0 };
    page.settle();
    page
}

// A newtype, because `fn` pointers compare by an address codegen may merge.
#[derive(Clone, Copy)]
struct App(fn() -> Element);

impl PartialEq for App {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::fn_addr_eq(self.0, other.0)
    }
}

#[component]
fn Root(app: App) -> Element {
    rsx! {
        LiberoProvider { Mounted { app } }
    }
}

// Its own scope, so `app`'s hooks see the provider's context.
#[component]
fn Mounted(app: App) -> Element {
    (app.0)()
}

/// A mounted document. Every action settles before it returns.
pub struct Page {
    pub doc: DioxusDocument,
    time: f64,
}

impl Page {
    /// Polls the vdom until it has no work left, then restyles and lays out.
    /// Dispatches what the document queued for its shell (`UiEvent::Activate`
    /// from `ElementApi::click`), as the shell's next turn would.
    pub fn settle(&mut self) {
        for _ in 0..MAX_POLLS {
            let busy = self.doc.poll(None);
            self.doc.inner.borrow_mut().resolve(self.time);
            let queued = self.doc.inner.borrow_mut().take_queued_ui_events();
            let dispatched = !queued.is_empty();
            for event in queued {
                self.doc.handle_ui_event(event);
            }
            if !busy && !dispatched {
                return;
            }
        }
        panic!("the vdom was still busy after {MAX_POLLS} polls");
    }

    /// Advances the animation clock, so a CSS transition reaches its end.
    pub fn advance(&mut self, seconds: f64) {
        self.time += seconds;
        self.settle();
    }

    /// Sleeps in real time, then settles: libero's timers are OS threads.
    pub fn wait(&mut self, duration: Duration) {
        thread::sleep(duration);
        self.settle();
    }

    /// Sends one raw event through Blitz's pipeline and settles.
    pub fn dispatch(&mut self, event: UiEvent) {
        self.doc.handle_ui_event(event);
        self.settle();
    }

    /// Key down and up on whatever holds focus.
    pub fn press(&mut self, key: Key) {
        self.press_with(key, Modifiers::empty());
    }

    pub fn press_with(&mut self, key: Key, modifiers: Modifiers) {
        self.dispatch(UiEvent::KeyDown(key_event(
            key.clone(),
            KeyState::Pressed,
            modifiers,
        )));
        self.dispatch(UiEvent::KeyUp(key_event(
            key,
            KeyState::Released,
            modifiers,
        )));
    }

    pub fn tab(&mut self) {
        self.press(Key::Tab);
    }

    pub fn shift_tab(&mut self) {
        self.press_with(Key::Tab, Modifiers::SHIFT);
    }

    /// Pointer down and up at the centre of the first match.
    pub fn click(&mut self, selector: &str) {
        let (x, y) = self.centre(selector);
        self.dispatch(UiEvent::PointerDown(pointer(x, y, true)));
        self.dispatch(UiEvent::PointerUp(pointer(x, y, false)));
    }

    /// Presses at the first match's centre, moves by `(dx, dy)` in eight steps,
    /// releases there.
    pub fn drag(&mut self, selector: &str, dx: f32, dy: f32) {
        const STEPS: u8 = 8;
        let (x, y) = self.centre(selector);
        self.dispatch(UiEvent::PointerDown(pointer(x, y, true)));
        for step in 1..=STEPS {
            let t = f32::from(step) / f32::from(STEPS);
            self.dispatch(UiEvent::PointerMove(pointer(x + dx * t, y + dy * t, true)));
        }
        self.dispatch(UiEvent::PointerUp(pointer(x + dx, y + dy, false)));
    }

    /// Moves the pointer, no button held, to the first match's centre.
    pub fn hover(&mut self, selector: &str) {
        let (x, y) = self.centre(selector);
        self.dispatch(UiEvent::PointerMove(pointer(x, y, false)));
    }

    /// Focuses the first match directly, the way `element.focus()` does.
    /// Blitz fires no focus or blur event for it.
    pub fn focus(&mut self, selector: &str) {
        let id = self.node(selector);
        self.doc.inner.borrow_mut().set_focus_to(id);
        self.settle();
    }

    pub fn query(&self, selector: &str) -> Option<NodeId> {
        self.doc
            .inner
            .borrow()
            .query_selector(selector)
            .unwrap_or_else(|_| panic!("Blitz cannot parse the selector {selector:?}"))
    }

    pub fn query_all(&self, selector: &str) -> Vec<NodeId> {
        self.doc
            .inner
            .borrow()
            .query_selector_all(selector)
            .unwrap_or_else(|_| panic!("Blitz cannot parse the selector {selector:?}"))
            .to_vec()
    }

    /// The first match. Panics with the tree when there is none.
    pub fn node(&self, selector: &str) -> NodeId {
        self.query(selector)
            .unwrap_or_else(|| panic!("nothing matches {selector:?} in\n{}", self.tree()))
    }

    pub fn exists(&self, selector: &str) -> bool {
        self.query(selector).is_some()
    }

    /// A click on nothing focusable leaves Blitz's focus on `<html>`, not `None`.
    pub fn focused(&self) -> Option<NodeId> {
        self.doc.inner.borrow().get_focussed_node_id()
    }

    /// What holds focus, for assertion messages.
    pub fn focus_owner(&self) -> String {
        self.focused()
            .map(|id| self.describe(id))
            .unwrap_or_else(|| "nothing".into())
    }

    pub fn is_focused(&self, selector: &str) -> bool {
        self.focused()
            .is_some_and(|id| self.query_all(selector).contains(&id))
    }

    /// An attribute of the first match.
    pub fn attr(&self, selector: &str, name: &str) -> Option<String> {
        self.attr_of(self.node(selector), name)
    }

    pub fn attr_of(&self, id: NodeId, name: &str) -> Option<String> {
        let doc = self.doc.inner.borrow();
        let node = doc.get_node(id)?;
        node.attrs()?
            .iter()
            .find(|attr| *attr.name.local == *name)
            .map(|attr| attr.value.clone())
    }

    pub fn text(&self, selector: &str) -> String {
        let id = self.node(selector);
        self.doc
            .inner
            .borrow()
            .get_node(id)
            .map(Node::text_content)
            .unwrap_or_default()
    }

    /// `getComputedStyle(match).getPropertyValue(property)`: the computed
    /// value, or the used one for layout-dependent properties.
    pub fn computed(&self, selector: &str, property: &str) -> String {
        let id = self.node(selector);
        self.doc.inner.borrow().resolved_style_value(id, property)
    }

    /// `<tag #id [attr=value ...]>` of a node, for assertion messages.
    pub fn describe(&self, id: NodeId) -> String {
        let doc = self.doc.inner.borrow();
        doc.get_node(id)
            .map(describe)
            .unwrap_or_else(|| format!("#{id} (gone)"))
    }

    /// The element tree, one per line, for a failing assertion's message.
    pub fn tree(&self) -> String {
        let doc = self.doc.inner.borrow();
        let mut out = String::new();
        write_tree(&mut out, &doc, doc.root_node(), 0);
        out
    }

    fn centre(&self, selector: &str) -> (f32, f32) {
        let id = self.node(selector);
        let rect = self
            .doc
            .inner
            .borrow()
            .get_client_bounding_rect(id)
            .unwrap_or_else(|| panic!("{selector:?} has no layout box"));
        (
            (rect.x + rect.width / 2.0) as f32,
            (rect.y + rect.height / 2.0) as f32,
        )
    }
}

fn describe(node: &Node) -> String {
    let Some(element) = node.element_data() else {
        return format!("{:?}", node.text_content());
    };
    let attrs: String = element
        .attrs
        .iter()
        .filter(|attr| {
            let name = &*attr.name.local;
            // Listeners show up as attributes valued `<rust func>`.
            !matches!(name, "style" | "class" | "data-dioxus-id") && attr.value != "<rust func>"
        })
        .map(|attr| format!(" {}={:?}", attr.name.local, attr.value))
        .collect();
    format!("<{}{attrs}>", element.name.local)
}

fn write_tree(out: &mut String, doc: &BaseDocument, node: &Node, depth: usize) {
    let skip = node
        .element_data()
        .is_some_and(|element| matches!(&*element.name.local, "style" | "head"));
    if skip {
        return;
    }
    if node.element_data().is_some() {
        out.push_str(&"  ".repeat(depth));
        out.push_str(&describe(node));
        out.push('\n');
    }
    for child in &node.children {
        if let Some(child) = doc.get_node(*child) {
            write_tree(out, doc, child, depth + 1);
        }
    }
}

fn key_event(key: Key, state: KeyState, modifiers: Modifiers) -> BlitzKeyEvent {
    let text = match (&key, state) {
        (Key::Character(text), KeyState::Pressed) => Some(text.as_str().into()),
        _ => None,
    };
    BlitzKeyEvent {
        key,
        code: Code::Unidentified,
        modifiers,
        location: Location::Standard,
        is_auto_repeating: false,
        is_composing: false,
        state,
        text,
    }
}

fn pointer(x: f32, y: f32, down: bool) -> BlitzPointerEvent {
    BlitzPointerEvent {
        id: BlitzPointerId::Mouse,
        is_primary: true,
        coords: PointerCoords {
            page_x: x,
            page_y: y,
            screen_x: x,
            screen_y: y,
            client_x: x,
            client_y: y,
        },
        button: MouseEventButton::Main,
        buttons: if down {
            MouseEventButtons::Primary
        } else {
            MouseEventButtons::empty()
        },
        mods: Modifiers::empty(),
        details: PointerDetails::default(),
        element: Point { x: 0.0, y: 0.0 },
        active_pointers: Default::default(),
    }
}
