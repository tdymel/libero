use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use blitz_dom::BaseDocument;
use dioxus::core::Runtime;
use dioxus::prelude::*;
use dioxus_native_dom::{NodeHandle, NodeId};

use style::properties::PropertyId;

mod activate;
mod color_scheme;
mod deferred;
mod direction;
mod document;
mod drag;
mod element;
mod focus;
mod focused;
mod form;
mod geometry;
mod keyboard;
mod listener;
mod max_length;
mod outlet;
mod placeholder;
mod portal;
mod press;
mod redraw;
mod refused;
mod removed;
mod resize;
mod scroll;
mod sticky;
mod wheel;

pub(super) use activate::focus_selectors;
use color_scheme::check_scheme;
pub(super) use color_scheme::color_scheme;
use deferred::{Deferred, defer, flush_soon, later, run_deferred, run_or_defer};
pub(super) use deferred::{awaits_layout, when_free, when_laid_out, when_laid_out_retry};
pub(super) use direction::{is_rtl, rtl_target};
pub(super) use document::document;
pub(super) use drag::follow_pointer;
use drag::followed;
use element::{BlitzElement, is_rendered, node_of};
pub(super) use element::{element, element_contains, mounted_by_selector};
pub(super) use focus::{focus_lands_in, press_kept_focus, silent_focus};
pub(super) use focused::{arrow_target, caret_edges, focus_is_in, typing_target, watch_focus};
use form::checked_attr;
pub(super) use form::{activated_submitter, form_values, implicit_submission};
use geometry::{client_extent, client_rect};
pub(super) use keyboard::keyboard;
use keyboard::{KeyCallback, keyed};
pub(super) use listener::Listener;
use listener::refocus_wrapper;
pub(super) use max_length::fit_pasted;
pub(super) use outlet::{Outlet, SheetWatch};
pub(super) use placeholder::sync_soon as placeholder_drawn;
use portal::HIT_AREA_STYLE;
pub(super) use portal::{PortalEntry, PortalRoot};
use press::{
    BLANK_PRESS, HIT, PRESS, activated, focusable_ancestor, forget_press, pressed, tab_from_start,
};
pub(super) use press::{focus_pressed, nested_interactive};
pub(super) use resize::{on_content_change, on_intersection, on_resize, on_viewport_resize};
use scroll::{LAYOUT_TRIES, notify_scroll, show};
pub(super) use scroll::{reveal, scroll, scroll_chain_into_view, scroll_padding_properties};

use super::{INTERACTIVE, origin::Origin};
use crate::{platform::TimerSubscription, tokens::ColorScheme};

/// Stylo's resolved value of a longhand, `""` for an unknown property or an
/// unstyled node.
fn resolved_style_value(doc: &BaseDocument, node_id: NodeId, property: &str) -> String {
    let Ok(property) = PropertyId::parse_enabled_for_all_content(property) else {
        return String::new();
    };
    let (Err(id), Some(style)) = (
        property.as_shorthand(),
        doc.get_node(node_id).and_then(|node| node.primary_styles()),
    ) else {
        return String::new();
    };
    style.computed_value_to_string(id)
}

thread_local! {
    /// Each live `VirtualDom`'s [`Doc`], by its runtime: dioxus-native runs one
    /// per document, and several on one thread (todo 702).
    static DOCS: RefCell<Vec<(Weak<Runtime>, Rc<Doc>)>> = const { RefCell::new(Vec::new()) };


    /// A button is held: a drag's renders wait for its release. See [`redraw`].
    static HELD: Cell<bool> = const { Cell::new(false) };
}

/// One document's backend state, shared by every `LiberoProvider` in it.
struct Doc {
    /// Each mounted [`Outlet`], newest last: the newest's handle is the
    /// document's, the one before takes over when it drops (todo 635).
    outlets: RefCell<Vec<OutletSlot>>,
    /// The first handle seen, the only way to the document until an [`Outlet`]
    /// mounts: `use_modal` asks for focus without an element (191).
    seen: RefCell<Option<NodeHandle>>,
    /// Commands that found the document borrowed, oldest first. See
    /// [`BlitzElement::command`].
    deferred: RefCell<Vec<Deferred>>,
    /// Reads waiting for the next poll's layout. See [`when_laid_out`].
    laid_out: RefCell<Vec<Deferred>>,
    laid_out_wait: RefCell<Option<Box<dyn TimerSubscription>>>,
    /// The [`when_laid_out_retry`] waits still to fire, and their timers.
    retrying: Cell<usize>,
    retry_waits: RefCell<Vec<Box<dyn TimerSubscription>>>,
    /// [`Listener`]'s own element.
    wrapper: RefCell<Option<NodeHandle>>,
    /// [`PortalRoot`]'s viewport-wide box. See [`HIT_AREA_STYLE`].
    hit_area: RefCell<Option<NodeHandle>>,
    /// The scheme Rust was last told, and who to tell when the viewport's
    /// differs. See [`BlitzColorScheme`](color_scheme::BlitzColorScheme).
    scheme: Cell<ColorScheme>,
    scheme_callbacks: Callbacks<dyn Fn(ColorScheme)>,
    /// Re-reads the viewport while anyone listens to the scheme.
    scheme_poll: RefCell<Option<Box<dyn TimerSubscription>>>,
    /// Who to tell that something scrolled. See [`BlitzScroll`](scroll::BlitzScroll).
    scroll_callbacks: Callbacks<dyn Fn()>,
    /// A `scroll_into_view` waiting for its target's first layout. See [`show`].
    show_retry: RefCell<Option<Box<dyn TimerSubscription>>>,
    /// Every [`BlitzKeyboard`](keyboard::BlitzKeyboard) subscription: id, whether it skips text entry,
    /// the subscribing scope, callback. Called by [`Listener`]'s `onkeydown`.
    key_callbacks: RefCell<Vec<KeyCallback>>,
    focus: focus::Watch,
    resize: resize::Watch,
    wheel: wheel::Watch,
    /// A `<style>` changed this poll. See [`SheetWatch`].
    sheets_changed: Cell<bool>,
    /// The document's shell provider is wrapped. See [`redraw`].
    redraws_watched: Cell<bool>,
}

/// A mounted [`Outlet`]: `flushes` is bumped to remount its flush element,
/// `mounted` is the flush element that last mounted.
struct OutletSlot {
    id: u64,
    flushes: Signal<u64>,
    mounted: Option<u64>,
    handle: Option<NodeHandle>,
}

impl Doc {
    fn new() -> Self {
        Self {
            outlets: RefCell::new(Vec::new()),
            seen: RefCell::new(None),
            deferred: RefCell::new(Vec::new()),
            laid_out: RefCell::new(Vec::new()),
            laid_out_wait: RefCell::new(None),
            retrying: Cell::new(0),
            retry_waits: RefCell::new(Vec::new()),
            wrapper: RefCell::new(None),
            hit_area: RefCell::new(None),
            scheme: Cell::new(ColorScheme::Light),
            scheme_callbacks: Callbacks::new(),
            scheme_poll: RefCell::new(None),
            scroll_callbacks: Callbacks::new(),
            show_retry: RefCell::new(None),
            key_callbacks: RefCell::new(Vec::new()),
            focus: focus::Watch::default(),
            resize: resize::Watch::default(),
            wheel: wheel::Watch::default(),
            sheets_changed: Cell::new(false),
            redraws_watched: Cell::new(false),
        }
    }

    fn anchor(&self) -> Option<NodeHandle> {
        let outlets = self.outlets.borrow();
        outlets
            .iter()
            .rev()
            .find_map(|outlet| outlet.handle.clone())
            .or_else(|| self.seen.borrow().clone())
    }

    fn wrapper_id(&self) -> Option<NodeId> {
        self.wrapper.borrow().as_ref().map(NodeHandle::node_id)
    }
}

/// The running `VirtualDom`'s [`Doc`], made on first ask. Timers run in their
/// runtime's root scope, so every caller finds its own.
fn doc() -> Option<Rc<Doc>> {
    let runtime = Runtime::try_current()?;
    let (doc, gone) = DOCS.with_borrow_mut(|docs| {
        let (live, gone): (Vec<_>, Vec<_>) = std::mem::take(docs)
            .into_iter()
            .partition(|(owner, _)| owner.strong_count() > 0);
        *docs = live;
        let doc = match docs
            .iter()
            .find(|(owner, _)| std::ptr::eq(owner.as_ptr(), Rc::as_ptr(&runtime)))
        {
            Some((_, doc)) => doc.clone(),
            None => {
                let doc = Rc::new(Doc::new());
                docs.push((Rc::downgrade(&runtime), doc.clone()));
                doc
            }
        };
        (doc, gone)
    });
    // Outside the borrow: a dropped doc's callbacks may reach for it.
    drop(gone);
    Some(doc)
}

/// Subscribers, each under the id its subscription drops, run in the scope
/// that subscribed.
struct Callbacks<F: ?Sized> {
    list: RefCell<Vec<(u64, Origin, Rc<F>)>>,
    next: Cell<u64>,
}

impl<F: ?Sized> Callbacks<F> {
    const fn new() -> Self {
        Self {
            list: RefCell::new(Vec::new()),
            next: Cell::new(0),
        }
    }

    fn add(&self, callback: Rc<F>) -> u64 {
        let id = self.next.replace(self.next.get() + 1);
        self.list.borrow_mut().push((id, Origin::here(), callback));
        id
    }

    /// Whether any are left.
    fn remove(&self, id: u64) -> bool {
        let mut list = self.list.borrow_mut();
        list.retain(|(other, ..)| *other != id);
        !list.is_empty()
    }

    /// Calls each, on a copy, so a callback may subscribe or unsubscribe.
    fn each(&self, call: impl Fn(&F)) {
        let list: Vec<_> = self
            .list
            .borrow()
            .iter()
            .map(|(_, origin, callback)| (*origin, callback.clone()))
            .collect();
        for (origin, callback) in list {
            origin.run(|| call(&callback));
        }
    }
}

/// The hit node and its ancestors, nearest first.
fn ancestors(doc: &BaseDocument, node_id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
    std::iter::successors(Some(node_id), |&id| doc.get_node(id)?.parent)
}

fn remember_document(handle: &NodeHandle) {
    if let Some(doc) = doc() {
        doc.seen.borrow_mut().get_or_insert_with(|| handle.clone());
    }
}

/// The running document's anchor, looked up per call so [`BlitzDocument`](document::BlitzDocument) can
/// be `&'static` and no early handle goes stale.
fn anchor() -> Option<NodeHandle> {
    doc()?.anchor()
}
