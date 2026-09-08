use std::{cell::RefCell, rc::Rc};

use blitz_dom::BaseDocument;
use blitz_traits::events::UiEvent;
use dioxus::prelude::*;
use dioxus_native_dom::{NodeHandle, NodeId};

use crate::platform::{Dimensions, DocumentApi, ElementApi, PlatformError, Read};

/// Blitz backs a mounted element with a `NodeHandle`, which carries the whole
/// document - so an element can answer for its subtree, and (via [`document`])
/// for the document too.
pub(super) fn element(mounted: &Rc<MountedData>) -> Option<Box<dyn ElementApi>> {
    let handle = mounted.downcast::<NodeHandle>()?.clone();
    remember_document(&handle);
    let node_id = handle.node_id();
    Some(Box::new(BlitzElement {
        anchor: handle,
        node_id,
    }))
}

thread_local! {
    /// A `NodeHandle` is the only way to reach the Blitz document, and
    /// `use_modal` has to ask where focus is without owning an element. So the
    /// first handle we ever see is kept as an anchor: its node may since have
    /// unmounted, but the document it points into outlives it, and only
    /// document-wide calls are made through it.
    ///
    /// [`Outlet`] fills it when `LiberoProvider` mounts. Waiting for the first
    /// element handle to be *called* left it empty until then, so the first
    /// popover was never placed and the first modal returned focus nowhere
    /// (todo 191, seen in a native window).
    static ANCHOR: RefCell<Option<NodeHandle>> = const { RefCell::new(None) };

    /// Commands that found the document borrowed, oldest first. See
    /// [`BlitzElement::command`].
    static DEFERRED: RefCell<Vec<Deferred>> = const { RefCell::new(Vec::new()) };

    /// Bumped to remount [`Outlet`]'s flush element. `None` until it renders.
    static FLUSHES: RefCell<Option<Signal<u64>>> = const { RefCell::new(None) };
}

type Deferred = (NodeHandle, Box<dyn FnOnce(&mut BaseDocument)>);

/// Mounted once by `LiberoProvider`. Its first mount is the document anchor;
/// every later one runs the deferred commands.
///
/// A mount is the one place dioxus-native calls back into user code with the
/// document free while no event is being dispatched:
/// `DioxusDocument::poll` fires `onmounted` after it drops the borrow it held
/// across `render_immediate`. So deferring bumps `FLUSHES`, the keyed element
/// below is replaced, and its `onmounted` runs the queue at the end of the
/// same poll (seen in a native window: a modal's focus return, deferred and
/// then run, before the next event).
#[component]
pub(super) fn Outlet() -> Element {
    let flushes = use_hook(|| {
        let flushes = Signal::new(0u64);
        FLUSHES.with(|slot| *slot.borrow_mut() = Some(flushes));
        flushes
    });
    use_drop(|| FLUSHES.with(|slot| *slot.borrow_mut() = None));

    rsx! {
        for flush in [flushes()] {
            div {
                key: "{flush}",
                display: "none",
                onmounted: move |event| {
                    if let Some(handle) = event.data().downcast::<NodeHandle>() {
                        remember_document(handle);
                    }
                    run_deferred();
                },
            }
        }
    }
}

fn run_deferred() {
    let deferred = DEFERRED.with(|queue| std::mem::take(&mut *queue.borrow_mut()));
    for (anchor, command) in deferred {
        command(&mut anchor.doc_mut());
    }
}

fn remember_document(handle: &NodeHandle) {
    ANCHOR.with(|anchor| {
        let mut anchor = anchor.borrow_mut();
        if anchor.is_none() {
            *anchor = Some(handle.clone());
        }
    });
}

pub(super) fn document() -> Option<&'static dyn DocumentApi> {
    anchor().map(|_| &DOCUMENT as &'static dyn DocumentApi)
}

/// The anchor as of right now. Held in a `thread_local` rather than in
/// [`BlitzDocument`], so the document can be a `&'static` like every other
/// capability - and so a handle taken before the first frame is not stale.
fn anchor() -> Option<NodeHandle> {
    ANCHOR.with(|anchor| anchor.borrow().clone())
}

struct BlitzDocument;

static DOCUMENT: BlitzDocument = BlitzDocument;

impl DocumentApi for BlitzDocument {
    fn viewport(&self) -> Read<Dimensions> {
        let anchor = anchor();
        let answer = match anchor.as_ref().and_then(|anchor| anchor.try_doc()) {
            Some(doc) => {
                // `window_size` is physical pixels; layout, and so every
                // rect `client_offset` answers, is in CSS pixels.
                let scale = doc.viewport().scale_f64();
                let (width, height) = doc.viewport().window_size;
                Ok(Dimensions {
                    width: width as f64 / scale,
                    height: height as f64 / scale,
                })
            }
            None => Err(PlatformError::Unsupported),
        };
        Box::pin(std::future::ready(answer))
    }

    fn active_element(&self) -> Option<Box<dyn ElementApi>> {
        let anchor = anchor()?;
        let node_id = anchor.try_doc()?.get_focussed_node_id()?;
        Some(Box::new(BlitzElement { anchor, node_id }))
    }

    /// Not natively, for now - so the theme switch rebuilds its sheet here
    /// instead. **Conservative, not impossible**, and the comment that said
    /// the root is unreachable was wrong: dioxus-native does build a real
    /// `html`/`head`/`body`/`main` tree and `BaseDocument::root_element()`
    /// reaches the `<html>` that `:root` matches. What is unverified is
    /// whether mutating an attribute there marks the node dirty for a
    /// restyle, so todo 69 phase 4 should try it before keeping this
    /// fallback. `@media (prefers-color-scheme: dark)` *is* honoured
    /// natively either way - stylo evaluates it off the window theme - so
    /// only an explicit override needs this. See
    /// [[codebase/blitz-platform-gaps]].
    fn set_root_attribute(&self, _name: &str, _value: Option<&str>) -> bool {
        false
    }
}

struct BlitzElement {
    /// Any handle into the same document; `node_id` is what this addresses.
    /// Nodes found by a query have no `NodeHandle` of their own - its fields
    /// are private to dioxus - so one is carried along to reach the document.
    anchor: NodeHandle,
    node_id: NodeId,
}

impl BlitzElement {
    /// **A read answers when it is called, not when it is awaited.** Blitz
    /// hands its event driver the document as a mutable borrow and holds it
    /// across the whole dispatch - which includes dioxus draining every task
    /// spawned from a handler, so a read *created* inside a `spawn` finds the
    /// document locked and no amount of yielding escapes that window (measured:
    /// 2000 self-wakes, still borrowed). During the handler itself it is free.
    ///
    /// So callers must build the future where the handler is and await it
    /// wherever; the returned `Read` is already resolved here, exactly as on
    /// the web.
    fn read<T: 'static>(&self, from: impl FnOnce(&BaseDocument, NodeId) -> Option<T>) -> Read<T> {
        let Some(doc) = self.anchor.try_doc() else {
            return Box::pin(std::future::ready(Err(PlatformError::Unsupported)));
        };
        let answer = from(&doc, self.node_id).ok_or(PlatformError::NotFound);
        Box::pin(std::future::ready(answer))
    }

    /// **A command runs now if it can, and at the end of this poll if not.**
    /// Every dioxus task - anything `spawn`ed, and every timer callback - is
    /// polled inside `render_immediate`, which `DioxusDocument::poll` calls
    /// while it holds the document mutably (`dioxus_document.rs:233-236` in
    /// native-dom). A command from there cannot take the document, and
    /// `doc_mut()` panicked: `FocusReturn::restore` closing any modal did
    /// exactly that (todo 189, seen in a native window). An event handler finds
    /// it free.
    ///
    /// `try_doc` is how to ask - there is no `try_doc_mut` - and it fails only
    /// while the document is borrowed mutably, which is the case that matters:
    /// nothing in libero holds a shared borrow across a command.
    fn command(&self, run: impl FnOnce(&mut BaseDocument) + 'static) {
        if self.anchor.try_doc().is_some() {
            run(&mut self.anchor.doc_mut());
            return;
        }
        DEFERRED.with(|queue| {
            queue
                .borrow_mut()
                .push((self.anchor.clone(), Box::new(run)))
        });
        FLUSHES.with(|slot| {
            if let Some(mut flushes) = *slot.borrow() {
                let next = flushes.peek().wrapping_add(1);
                flushes.set(next);
            }
        });
    }

    fn at(&self, node_id: NodeId) -> Box<dyn ElementApi> {
        Box::new(BlitzElement {
            anchor: self.anchor.clone(),
            node_id,
        })
    }
}

impl ElementApi for BlitzElement {
    fn focus(&self) -> Result<(), PlatformError> {
        let node_id = self.node_id;
        self.command(move |doc| {
            doc.set_focus_to(node_id);
        });
        Ok(())
    }

    fn blur(&self) -> Result<(), PlatformError> {
        let node_id = self.node_id;
        self.command(move |doc| {
            if doc.get_focussed_node_id() == Some(node_id) {
                doc.clear_focus();
            }
        });
        Ok(())
    }

    /// Queued rather than dispatched: everything that reaches a dioxus
    /// `onclick` belongs to the `Document` wrapping this one, which only the
    /// shell holds - and we are usually inside a handler with the document
    /// mid-dispatch anyway. The shell runs it on its next turn, through the
    /// whole pipeline. Like every command here, "queued" is the answer.
    fn click(&self) -> Result<(), PlatformError> {
        let node_id = self.node_id;
        self.command(move |doc| doc.queue_ui_event(UiEvent::Activate(node_id)));
        Ok(())
    }

    fn is_focused(&self) -> bool {
        self.anchor
            .try_doc()
            .is_some_and(|doc| doc.get_focussed_node_id() == Some(self.node_id))
    }

    /// Blitz tracks this itself: `NodeFlags::IS_IN_DOCUMENT` is set across a
    /// whole subtree when it is inserted and cleared across it again when it
    /// is removed (`blitz-dom`'s `Mutator::process_added_subtree` /
    /// `process_removed_subtree`). So this is a flag read, not a walk up the
    /// parent chain, and it is exactly the DOM's `isConnected`.
    ///
    /// A removed node is also dropped from the slab, so a missing node is gone
    /// too - both answers are covered.
    ///
    /// No document in reach answers `true`, the trait's rule for a renderer
    /// that cannot tell. That case is real here: `try_doc` fails while dioxus
    /// is draining tasks, and an optimistic answer keeps focus return doing
    /// what it did before the predicate existed.
    fn is_connected(&self) -> bool {
        let Some(doc) = self.anchor.try_doc() else {
            return true;
        };
        doc.get_node(self.node_id)
            .is_some_and(|node| node.flags.is_in_document())
    }

    fn dimensions(&self) -> Read<Dimensions> {
        self.read(|doc, node_id| {
            let rect = doc.get_client_bounding_rect(node_id)?;
            Some(Dimensions {
                width: rect.width,
                height: rect.height,
            })
        })
    }

    fn client_offset(&self) -> Read<(f64, f64)> {
        self.read(|doc, node_id| {
            let rect = doc.get_client_bounding_rect(node_id)?;
            Some((rect.x, rect.y))
        })
    }

    fn scroll_size(&self) -> Read<Dimensions> {
        self.read(|doc, node_id| {
            let layout = doc.get_node(node_id)?.final_layout();
            Some(Dimensions {
                width: layout.scroll_width() as f64,
                height: layout.scroll_height() as f64,
            })
        })
    }

    fn scroll_offset(&self) -> Read<(f64, f64)> {
        self.read(|doc, node_id| {
            let offset = doc.get_node(node_id)?.scroll_offset();
            Some((offset.x, offset.y))
        })
    }

    /// Only a raster picture carries its size; a decoded SVG answers
    /// `NotFound`, as does an image still loading.
    fn natural_size(&self) -> Read<Dimensions> {
        self.read(|doc, node_id| {
            let image = doc.get_node(node_id)?.element_data()?.raster_image_data()?;
            Some(Dimensions {
                width: image.width as f64,
                height: image.height as f64,
            })
        })
    }

    /// Blitz only scrolls by a delta, so this reads the current offset first -
    /// when the command runs, which may be after it was asked for.
    fn scroll_to(&self, x: f64, y: f64) -> Result<(), PlatformError> {
        if self
            .anchor
            .try_doc()
            .is_some_and(|doc| doc.get_node(self.node_id).is_none())
        {
            return Err(PlatformError::NotFound);
        }
        let node_id = self.node_id;
        self.command(move |doc| {
            let Some(offset) = doc.get_node(node_id).map(|node| *node.scroll_offset()) else {
                return;
            };
            doc.scroll_node_by(node_id, x - offset.x, y - offset.y, |_| {});
        });
        Ok(())
    }

    /// Not written: finding the scrolling ancestor needs Blitz's overflow
    /// styles, and nothing native has asked for it.
    fn scroll_into_view(&self, _smooth: bool) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    /// Blitz has no `FileList`, and nothing native posts a form anyway.
    fn set_files(&self, _files: &[dioxus::html::FileData]) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    /// Blitz implements no form reset.
    fn reset(&self) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    /// Blitz fires no submit event from code.
    fn request_submit(&self) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    /// Blitz has no pointer capture. A native drag keeps tracking anyway while
    /// the handlers sit on a container the pointer stays inside.
    fn set_pointer_capture(&self, _pointer_id: i32) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    fn query_selector(&self, selector: &str) -> Result<Box<dyn ElementApi>, PlatformError> {
        let doc = self.anchor.try_doc().ok_or(PlatformError::Unsupported)?;
        let node_id = doc
            .query_selector_in(self.node_id, selector)
            .map_err(|_| PlatformError::NotFound)?
            .ok_or(PlatformError::NotFound)?;
        drop(doc);
        Ok(self.at(node_id))
    }

    fn query_selector_all(
        &self,
        selector: &str,
    ) -> Result<Vec<Box<dyn ElementApi>>, PlatformError> {
        let doc = self.anchor.try_doc().ok_or(PlatformError::Unsupported)?;
        let nodes = doc
            .query_selector_all_in(self.node_id, selector)
            .map_err(|_| PlatformError::NotFound)?;
        drop(doc);
        Ok(nodes.into_iter().map(|node_id| self.at(node_id)).collect())
    }
}
