//! A headless Blitz harness: one libero component in a windowless
//! `DioxusDocument`, driven through Blitz's own event pipeline.
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

use anyrender::{
    Paint, Scene,
    recording::{GlyphRunCommand, RenderCommand},
};
use anyrender_vello_cpu::VelloCpuImageRenderer;
use blitz_dom::{
    BaseDocument, Document, DocumentConfig, Node,
    node::{ImageData, SpecialElementData},
};
use blitz_traits::{
    NodeId,
    events::{
        BlitzKeyEvent, BlitzPointerEvent, BlitzPointerId, BlitzWheelDelta, BlitzWheelEvent,
        KeyState, MouseEventButton, MouseEventButtons, Point, PointerCoords, PointerDetails,
        UiEvent,
    },
    shell::Viewport,
};
use dioxus::prelude::*;
use dioxus_native_dom::DioxusDocument;
use libero::LiberoProvider;
use style::properties::PropertyId;
use warnings::SignalWarnings;

mod warnings;

pub use blitz_traits::shell::ColorScheme;
pub use dioxus::prelude::{Code, Key, Location, Modifiers};

/// The viewport every test renders into, CSS pixels at scale 1.
pub const VIEWPORT: (u32, u32) = (1024, 768);

// Bounds `settle`, so a render loop fails the test instead of hanging it.
const MAX_POLLS: usize = 200;

// HTML bool attributes, set by presence whatever their value. Not `checked`:
// Blitz reads `checked="false"` as unticked, and `reset` writes it.
const FALSE_FLAGS: &str = "[disabled=false], [hidden=false], [readonly=false], \
    [required=false], [selected=false], [multiple=false], [open=false], \
    [autofocus=false], [inert=false]";

/// Mounts `app` inside a [`LiberoProvider`] and settles the first render.
pub fn mount(app: fn() -> Element) -> Page {
    mount_in(app, ColorScheme::Light)
}

/// [`mount`], with the window theme set to `scheme`.
pub fn mount_in(app: fn() -> Element, scheme: ColorScheme) -> Page {
    let warnings = SignalWarnings::watch();
    let vdom = VirtualDom::new_with_props(Root, RootProps { app: App(app) });
    let mut doc = DioxusDocument::new(
        vdom,
        DocumentConfig {
            viewport: Some(viewport(scheme)),
            html_parser_provider: Some(Arc::new(blitz_html::HtmlProvider)),
            ..Default::default()
        },
    );
    doc.initial_build();
    let mut page = Page {
        doc,
        time: 0.0,
        warnings,
    };
    page.settle();
    page
}

/// A painted colour in `computed`'s notation, so the two compare.
fn css_color(color: peniko::Color) -> String {
    let [r, g, b, a] = color.to_rgba8().to_u8_array();
    if a == 255 {
        format!("rgb({r}, {g}, {b})")
    } else {
        format!("rgba({r}, {g}, {b}, {})", a as f32 / 255.0)
    }
}

/// Blitz's client rect leaves out `transform`; the web's does not. libero's
/// `client_rect` in `platform/backend/blitz.rs` does the same walk.
fn transformed_rect(doc: &BaseDocument, node_id: NodeId) -> Option<(f64, f64, f64, f64)> {
    let node = doc.get_node(node_id)?;
    let size = node.unrounded_layout().size;
    let (width, height) = (f64::from(size.width), f64::from(size.height));
    let mut corners = [(0.0, 0.0), (width, 0.0), (0.0, height), (width, height)];
    let scale = doc.viewport().scale_f64();
    let mut transformed = false;
    let mut current = Some(node);
    while let Some(node) = current {
        let boxed = matches!(
            node.data,
            blitz_dom::NodeData::Element(_) | blitz_dom::NodeData::AnonymousBlock(_)
        );
        if let Some(transform) = boxed.then(|| *node.transform()).flatten() {
            let [a, b, c, d, e, f] = transform.as_coeffs();
            for (x, y) in &mut corners {
                (*x, *y) = (a * *x + c * *y + e / scale, b * *x + d * *y + f / scale);
            }
            transformed = true;
        }
        let location = node.final_layout().location;
        let parent = node.layout_parent.get().and_then(|id| doc.get_node(id));
        let scroll = parent.map(|parent| *parent.scroll_offset());
        for (x, y) in &mut corners {
            *x += f64::from(location.x) - scroll.map_or(0.0, |s| s.x);
            *y += f64::from(location.y) - scroll.map_or(0.0, |s| s.y);
        }
        current = parent;
    }
    if !transformed {
        return None;
    }
    let scroll = doc.viewport_scroll();
    let xs = corners.map(|(x, _)| x - scroll.x);
    let ys = corners.map(|(_, y)| y - scroll.y);
    let min_x = xs.iter().copied().fold(f64::INFINITY, f64::min);
    let min_y = ys.iter().copied().fold(f64::INFINITY, f64::min);
    let max_x = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let max_y = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    Some((min_x, min_y, max_x - min_x, max_y - min_y))
}

/// crates.io Blitz stops marking ancestors at a stale dirty bit, so a clock
/// tick's restyle can be skipped and freeze a transition (Blitz #789, todo 880).
/// libero heals at a flush, a read or a pointer move; a tick has none of those.
fn heal_dirty_bits(doc: &BaseDocument) {
    let mut dirty = Vec::new();
    doc.visit(|id, node| {
        if node.has_dirty_descendants() {
            dirty.push(id);
        }
    });
    for id in dirty {
        let mut parent = doc.get_node(id).and_then(|node| node.parent);
        while let Some(node) = parent.and_then(|id| doc.get_node(id)) {
            node.set_dirty_descendants();
            parent = node.parent;
        }
    }
}

fn viewport(scheme: ColorScheme) -> Viewport {
    Viewport::new(VIEWPORT.0, VIEWPORT.1, 1.0, scheme)
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
    warnings: SignalWarnings,
}

impl Page {
    /// Polls the vdom until it has no work left, then restyles and lays out.
    pub fn settle(&mut self) {
        for _ in 0..MAX_POLLS {
            let busy = self.doc.poll(None);
            self.doc.inner.borrow_mut().resolve(self.time);
            if !busy {
                self.assert_no_false_flags();
                self.assert_no_signal_warnings();
                return;
            }
        }
        panic!("the vdom was still busy after {MAX_POLLS} polls");
    }

    /// dioxus-native writes a `false` bool attribute as `disabled="false"`,
    /// which Blitz reads as set: raw `rsx!` must write `flag.then_some(true)`.
    fn assert_no_false_flags(&self) {
        if let Some(id) = self.query(FALSE_FLAGS) {
            panic!(
                "a bool attribute written as \"false\" on {} in\n{}",
                self.describe(id),
                self.tree()
            );
        }
    }

    /// A `dioxus_signals` warning names a value read outside its owner's
    /// scope, which may be dropped under the reader.
    fn assert_no_signal_warnings(&self) {
        let seen = self.warnings.take();
        if !seen.is_empty() {
            panic!("dioxus_signals warned:\n{}", seen.join("\n"));
        }
    }

    /// Advances the animation clock, so a CSS transition reaches its end.
    pub fn advance(&mut self, seconds: f64) {
        self.time += seconds;
        heal_dirty_bits(&self.doc.inner.borrow());
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

    /// Switches the window theme, as the shell does on a theme-change event.
    pub fn set_color_scheme(&mut self, scheme: ColorScheme) {
        self.doc.inner.borrow_mut().set_viewport(viewport(scheme));
        self.settle();
    }

    /// Resizes the window, as the shell does on a resize event. `painted_*`
    /// still rasterise [`VIEWPORT`].
    pub fn resize(&mut self, width: u32, height: u32) {
        {
            let mut doc = self.doc.inner.borrow_mut();
            let mut viewport = doc.viewport().clone();
            viewport.window_size = (width, height);
            doc.set_viewport(viewport);
        }
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
        self.click_at(x, y);
    }

    /// Pointer down and up at a viewport point, e.g. on a backdrop.
    pub fn click_at(&mut self, x: f32, y: f32) {
        self.dispatch(UiEvent::PointerDown(self.pointer(x, y, true)));
        self.dispatch(UiEvent::PointerUp(self.pointer(x, y, false)));
    }

    /// [`click`](Self::click), but the vdom runs dry before the next layout, as
    /// a window's shell may poll several times between two frames.
    pub fn click_before_layout(&mut self, selector: &str) {
        let (x, y) = self.centre(selector);
        self.dispatch(UiEvent::PointerDown(self.pointer(x, y, true)));
        self.dispatch_before_layout(UiEvent::PointerUp(self.pointer(x, y, false)));
    }

    /// [`press`](Self::press), polled dry before the next layout.
    pub fn press_before_layout(&mut self, key: Key) {
        let modifiers = Modifiers::empty();
        self.doc.handle_ui_event(UiEvent::KeyDown(key_event(
            key.clone(),
            KeyState::Pressed,
            modifiers,
        )));
        self.dispatch_before_layout(UiEvent::KeyUp(key_event(
            key,
            KeyState::Released,
            modifiers,
        )));
    }

    /// [`dispatch`](Self::dispatch), polled dry before the next layout.
    pub fn dispatch_before_layout(&mut self, event: UiEvent) {
        self.doc.handle_ui_event(event);
        for _ in 0..MAX_POLLS {
            if !self.doc.poll(None) {
                break;
            }
        }
        self.settle();
    }

    /// Presses at the first match's centre, moves by `(dx, dy)` in eight steps,
    /// releases there.
    pub fn drag(&mut self, selector: &str, dx: f32, dy: f32) {
        const STEPS: u8 = 8;
        let (x, y) = self.centre(selector);
        self.dispatch(UiEvent::PointerDown(self.pointer(x, y, true)));
        for step in 1..=STEPS {
            let t = f32::from(step) / f32::from(STEPS);
            self.dispatch(UiEvent::PointerMove(self.pointer(
                x + dx * t,
                y + dy * t,
                true,
            )));
        }
        self.dispatch(UiEvent::PointerUp(self.pointer(x + dx, y + dy, false)));
    }

    /// The steps of a hand-driven drag, each settled: press, move with the
    /// button held, release.
    pub fn press_at(&mut self, x: f32, y: f32) {
        self.dispatch(UiEvent::PointerDown(self.pointer(x, y, true)));
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.dispatch(UiEvent::PointerMove(self.pointer(x, y, true)));
    }

    pub fn release_at(&mut self, x: f32, y: f32) {
        self.dispatch(UiEvent::PointerUp(self.pointer(x, y, false)));
    }

    /// Moves the pointer, no button held, to the first match's centre.
    pub fn hover(&mut self, selector: &str) {
        let (x, y) = self.centre(selector);
        self.dispatch(UiEvent::PointerMove(self.pointer(x, y, false)));
    }

    /// Turns the wheel over the first match's centre: a positive `dy` scrolls
    /// down by that many pixels. Blitz scrolls what the pointer hovers, so
    /// `hover` it first, once: a second move there dropped the next wheel.
    pub fn wheel(&mut self, selector: &str, dy: f64) {
        let (x, y) = self.centre(selector);
        self.wheel_at(x, y, dy);
    }

    /// [`wheel`](Self::wheel) at a viewport point, whatever is there now.
    pub fn wheel_at(&mut self, x: f32, y: f32, dy: f64) {
        let coords = self.pointer(x, y, false).coords;
        self.dispatch(UiEvent::Wheel(BlitzWheelEvent {
            // Blitz's sign is a finger's: a negative delta scrolls down.
            delta: BlitzWheelDelta::Pixels(0.0, -dy),
            coords,
            buttons: MouseEventButtons::empty(),
            mods: Modifiers::empty(),
            element: Point { x: 0.0, y: 0.0 },
        }));
    }

    /// [`wheel`](Self::wheel) sideways: a positive `dx` scrolls right.
    pub fn wheel_x(&mut self, selector: &str, dx: f64) {
        let (x, y) = self.centre(selector);
        let coords = self.pointer(x, y, false).coords;
        self.dispatch(UiEvent::Wheel(BlitzWheelEvent {
            delta: BlitzWheelDelta::Pixels(-dx, 0.0),
            coords,
            buttons: MouseEventButtons::empty(),
            mods: Modifiers::empty(),
            element: Point { x: 0.0, y: 0.0 },
        }));
    }

    /// A node's vertical scroll offset.
    pub fn scroll_top(&self, selector: &str) -> f64 {
        let id = self.node(selector);
        self.doc
            .inner
            .borrow()
            .get_node(id)
            .map_or(0.0, |node| node.scroll_offset().y)
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

    /// The text of every laid-out run inside the matches that broke onto more
    /// than one line: the painted text, not the box (todo 627).
    pub fn wrapped_text(&self, selector: &str) -> Vec<String> {
        let doc = self.doc.inner.borrow();
        let mut stack = self.query_all(selector);
        let mut wrapped = Vec::new();
        while let Some(id) = stack.pop() {
            let node = doc.get_node(id).expect("a matched node");
            let inline = node
                .data
                .downcast_element()
                .and_then(|data| data.inline_layout_data.as_ref());
            if let Some(inline) = inline.filter(|inline| inline.layout.len() > 1) {
                let lines = inline.layout.len();
                wrapped.push(format!("{:?} in {lines} lines", inline.text));
            }
            // Layout children too: a flex item's text sits in an anonymous block.
            let layout = node.layout_children.borrow();
            let children = layout.as_deref().unwrap_or(&node.children);
            stack.extend(children.iter().copied());
        }
        wrapped
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
        self.computed_of(self.node(selector), property)
    }

    pub fn computed_of(&self, id: NodeId, property: &str) -> String {
        let doc = self.doc.inner.borrow();
        let Ok(property) = PropertyId::parse_enabled_for_all_content(property) else {
            return String::new();
        };
        let (Err(id), Some(style)) = (
            property.as_shorthand(),
            doc.get_node(id).and_then(|node| node.primary_styles()),
        ) else {
            return String::new();
        };
        style.computed_value_to_string(id)
    }

    /// The stroke Blitz will paint the first match's (an `svg`) first stroked
    /// path with, as `rgb(..)`. Not `computed`: Blitz bakes it at box build.
    pub fn painted_stroke(&self, selector: &str) -> String {
        self.painted_stroke_of(self.node(selector))
    }

    /// [`painted_stroke`](Self::painted_stroke) of a node.
    pub fn painted_stroke_of(&self, id: NodeId) -> String {
        let selector = self.describe(id);
        let doc = self.doc.inner.borrow();
        let element = doc
            .get_node(id)
            .and_then(|node| node.element_data())
            .unwrap();
        let SpecialElementData::Image(image) = &element.special_data else {
            panic!("{selector:?} was not built as an image");
        };
        let ImageData::Svg(svg) = &**image else {
            panic!("{selector:?} was not built as an svg");
        };
        let tree = format!("{:?}", svg.tree.root());
        let start = tree.find("Color { red: ").expect("no stroke colour") + "Color { red: ".len();
        let rest = &tree[start..];
        let channels: Vec<&str> = rest[..rest.find(" }").unwrap()]
            .split(", ")
            .map(|channel| channel.rsplit(": ").next().unwrap())
            .collect();
        format!("rgb({})", channels.join(", "))
    }

    /// The transform Blitz paints the first match with, as affine coefficients.
    /// Not `computed`: Blitz caches it at layout.
    pub fn painted_transform(&self, selector: &str) -> Option<[f64; 6]> {
        let id = self.node(selector);
        let doc = self.doc.inner.borrow();
        doc.get_node(id)?
            .transform()
            .map(|affine| affine.as_coeffs())
    }

    /// The scene Blitz would draw now, recorded rather than rasterised.
    fn scene(&self) -> Scene {
        let mut scene = Scene::new();
        let mut doc = self.doc.inner.borrow_mut();
        blitz_paint::paint_scene(&mut scene, &mut doc, 1.0, VIEWPORT.0, VIEWPORT.1, 0, 0);
        scene
    }

    /// The colour of the pixel at `(x, y)`, as `rgb(..)`: the scene rasterised
    /// by the shell's CPU renderer, where a recorded command may still draw
    /// nothing (a zero-blur shadow).
    pub fn painted_pixel(&self, x: u32, y: u32) -> String {
        let [r, g, b, a] = self.painted_pixels(&[(x, y)])[0];
        css_color(peniko::Color::from_rgba8(r, g, b, a))
    }

    /// [`painted_pixel`](Self::painted_pixel) for many points, rasterised once.
    pub fn painted_pixels(&self, points: &[(u32, u32)]) -> Vec<[u8; 4]> {
        let (width, height) = VIEWPORT;
        let mut doc = self.doc.inner.borrow_mut();
        let buffer = anyrender::render_to_buffer::<VelloCpuImageRenderer, _>(
            |scene| blitz_paint::paint_scene(scene, &mut doc, 1.0, width, height, 0, 0),
            width,
            height,
        );
        points
            .iter()
            .map(|&(x, y)| {
                let at = ((y * width + x) * 4) as usize;
                [0, 1, 2, 3].map(|i| buffer[at + i])
            })
            .collect()
    }

    /// The whole viewport rasterised as a binary PPM, for looking at a failure.
    pub fn save_ppm(&self, path: &str) {
        let (width, height) = VIEWPORT;
        let mut doc = self.doc.inner.borrow_mut();
        let buffer = anyrender::render_to_buffer::<VelloCpuImageRenderer, _>(
            |scene| blitz_paint::paint_scene(scene, &mut doc, 1.0, width, height, 0, 0),
            width,
            height,
        );
        let mut out = format!("P6\n{width} {height}\n255\n").into_bytes();
        out.extend(buffer.chunks(4).flat_map(|px| [px[0], px[1], px[2]]));
        std::fs::write(path, out).expect("writable path");
    }

    /// The colour Blitz paints the first match's text with, as `rgb(..)`. Not
    /// `computed`: an anonymous block's text keeps the style of its box build.
    pub fn painted_text(&self, selector: &str) -> String {
        let (left, top, width, height) = self.rect(selector);
        let rect = kurbo::Rect::new(left, top, left + width, top + height);
        self.scene()
            .commands
            .iter()
            .find_map(|command| match command {
                RenderCommand::GlyphRun(GlyphRunCommand {
                    transform,
                    brush: Paint::Solid(color),
                    glyphs,
                    ..
                }) if glyphs.iter().any(|glyph| {
                    // Just above the baseline, inside the line box.
                    let at = kurbo::Point::new(glyph.x as f64, glyph.y as f64 - 1.0);
                    rect.contains(*transform * at)
                }) =>
                {
                    Some(css_color(*color))
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("no text painted in {selector:?}"))
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

    /// The first match's `getBoundingClientRect()`: `(x, y, width, height)`.
    pub fn rect(&self, selector: &str) -> (f64, f64, f64, f64) {
        let id = self.node(selector);
        let doc = self.doc.inner.borrow();
        if let Some(rect) = transformed_rect(&doc, id) {
            return rect;
        }
        let rect = doc
            .get_client_bounding_rect(id)
            .unwrap_or_else(|| panic!("{selector:?} has no layout box"));
        (rect.x, rect.y, rect.width, rect.height)
    }

    /// Whether a pointer at the first match's centre hits it or a descendant.
    pub fn hits(&self, selector: &str) -> bool {
        let (x, y) = self.centre(selector);
        self.hits_at(selector, x, y)
    }

    /// Whether a pointer at `(x, y)` hits the first match or a descendant.
    pub fn hits_at(&self, selector: &str, x: f32, y: f32) -> bool {
        let target = self.node(selector);
        let (left, top) = self.viewport_scroll();
        let doc = self.doc.inner.borrow();
        let mut hit = doc.hit(x + left, y + top).map(|hit| hit.node_id);
        while let Some(id) = hit {
            if id == target {
                return true;
            }
            hit = doc.get_node(id).and_then(|node| node.parent);
        }
        false
    }

    fn centre(&self, selector: &str) -> (f32, f32) {
        let (x, y, width, height) = self.rect(selector);
        ((x + width / 2.0) as f32, (y + height / 2.0) as f32)
    }

    /// How far the window itself has scrolled.
    pub fn viewport_scroll(&self) -> (f32, f32) {
        let scroll = self.doc.inner.borrow().viewport_scroll();
        (scroll.x as f32, scroll.y as f32)
    }

    /// A pointer at a viewport point. Blitz hit-tests the page point, so the
    /// window's scroll is added, as the shell does (todo 655).
    fn pointer(&self, x: f32, y: f32, down: bool) -> BlitzPointerEvent {
        let (left, top) = self.viewport_scroll();
        let mut event = pointer(x, y, down);
        event.coords.page_x += left;
        event.coords.page_y += top;
        event
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
