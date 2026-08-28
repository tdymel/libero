use std::{cell::RefCell, rc::Rc};

use blitz_dom::BaseDocument;
use blitz_traits::events::UiEvent;
use dioxus::prelude::MountedData;
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
    /// first handle we ever resolve is kept as an anchor: its node may since
    /// have unmounted, but the document it points into outlives it, and only
    /// document-wide calls are made through it.
    static ANCHOR: RefCell<Option<NodeHandle>> = const { RefCell::new(None) };
}

fn remember_document(handle: &NodeHandle) {
    ANCHOR.with(|anchor| {
        let mut anchor = anchor.borrow_mut();
        if anchor.is_none() {
            *anchor = Some(handle.clone());
        }
    });
}

pub(super) fn document() -> Option<Box<dyn DocumentApi>> {
    ANCHOR.with(|anchor| {
        let anchor = anchor.borrow().clone()?;
        Some(Box::new(BlitzDocument { anchor }) as Box<dyn DocumentApi>)
    })
}

struct BlitzDocument {
    anchor: NodeHandle,
}

impl DocumentApi for BlitzDocument {
    fn viewport(&self) -> Read<Dimensions> {
        let answer = match self.anchor.try_doc() {
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
        let node_id = self.anchor.try_doc()?.get_focussed_node_id()?;
        Some(Box::new(BlitzElement {
            anchor: self.anchor.clone(),
            node_id,
        }))
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

    fn at(&self, node_id: NodeId) -> Box<dyn ElementApi> {
        Box::new(BlitzElement {
            anchor: self.anchor.clone(),
            node_id,
        })
    }
}

impl ElementApi for BlitzElement {
    // Focus and scrolling take the document mutably. Nothing holds a borrow
    // while a handler or a spawned task runs - `DioxusDocument::poll` releases
    // it before polling tasks, and `handle_event` before dispatching - which is
    // why dioxus's own `NodeHandle::set_focus` borrows the same way.
    fn focus(&self) -> Result<(), PlatformError> {
        self.anchor.doc_mut().set_focus_to(self.node_id);
        Ok(())
    }

    fn blur(&self) -> Result<(), PlatformError> {
        let mut doc = self.anchor.doc_mut();
        if doc.get_focussed_node_id() == Some(self.node_id) {
            doc.clear_focus();
        }
        Ok(())
    }

    /// Queued rather than dispatched: everything that reaches a dioxus
    /// `onclick` belongs to the `Document` wrapping this one, which only the
    /// shell holds - and we are usually inside a handler with the document
    /// mid-dispatch anyway. The shell runs it on its next turn, through the
    /// whole pipeline. Like every command here, "queued" is the answer.
    fn click(&self) -> Result<(), PlatformError> {
        self.anchor
            .doc_mut()
            .queue_ui_event(UiEvent::Activate(self.node_id));
        Ok(())
    }

    fn is_focused(&self) -> bool {
        self.anchor
            .try_doc()
            .is_some_and(|doc| doc.get_focussed_node_id() == Some(self.node_id))
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

    /// Blitz only scrolls by a delta, so this reads the current offset first.
    fn scroll_to(&self, x: f64, y: f64) -> Result<(), PlatformError> {
        let mut doc = self.anchor.doc_mut();
        let offset = *doc
            .get_node(self.node_id)
            .ok_or(PlatformError::NotFound)?
            .scroll_offset();
        doc.scroll_node_by(self.node_id, x - offset.x, y - offset.y, |_| {});
        Ok(())
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
