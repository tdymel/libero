use std::rc::Rc;

use blitz_dom::{
    BaseDocument, QualName, local_name,
    node::{ImageData, SpecialElementData},
    ns,
};
use dioxus::prelude::*;
use dioxus_native_dom::{NodeHandle, NodeId};

use super::FreeDoc;
use super::{
    LAYOUT_TRIES, activate, ancestors, checked_attr, client_extent, client_rect, doc, focus,
    notify_scroll, remember_document, resolved_style_value, run_or_defer, show,
};
use crate::platform::{Dimensions, ElementApi, PlatformError, Read};

/// Blitz backs a mounted element with a `NodeHandle`, which carries the whole
/// document, so it answers for its subtree and (via [`document`]) the document.
pub(in crate::platform::backend) fn element(
    mounted: &Rc<MountedData>,
) -> Option<Box<dyn ElementApi>> {
    let (anchor, node_id) = node_of(mounted)?;
    remember_document(&anchor);
    Some(Box::new(BlitzElement { anchor, node_id }))
}

/// A document query's match: any handle carries the whole document, so one stands in.
struct Found {
    pub(super) anchor: NodeHandle,
    pub(super) node_id: NodeId,
}

impl dioxus::html::RenderedElementBacking for Found {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub(in crate::platform::backend) fn mounted_by_selector(selector: &str) -> Option<Rc<MountedData>> {
    let anchor = doc()?.anchor()?;
    let node_id = anchor.free_doc()?.query_selector(selector).ok()??;
    Some(Rc::new(MountedData::new(Found { anchor, node_id })))
}

/// The node behind `mounted`, and a handle to reach its document by.
pub(super) fn node_of(mounted: &Rc<MountedData>) -> Option<(NodeHandle, NodeId)> {
    if let Some(handle) = mounted.downcast::<NodeHandle>() {
        return Some((handle.clone(), handle.node_id()));
    }
    let found = mounted.downcast::<Found>()?;
    Some((found.anchor.clone(), found.node_id))
}

/// `false` under a `display: none`, or for a `visibility: hidden` node: the
/// web's `focus()` ignores both. A node not styled yet counts as rendered.
pub(super) fn is_rendered(doc: &BaseDocument, node_id: NodeId) -> bool {
    let styles = |id| doc.get_node(id).and_then(|node| node.primary_styles());
    let visible = styles(node_id).is_none_or(|styles| {
        styles.clone_visibility() == style::computed_values::visibility::T::Visible
    });
    visible
        && doc
            .node_chain(node_id)
            .into_iter()
            .all(|id| styles(id).is_none_or(|styles| !styles.clone_display().is_none()))
}

pub(in crate::platform::backend) fn element_contains(
    outer: &Rc<MountedData>,
    inner: &Rc<MountedData>,
) -> bool {
    // A selector-found element too: a Menu's wrapper anchors a popup (todo 2989).
    let (Some((_, target)), Some((inner, node_id))) = (node_of(outer), node_of(inner)) else {
        return false;
    };
    inner
        .free_doc()
        .is_some_and(|doc| ancestors(&doc, node_id).any(|id| id == target))
}

pub(super) struct BlitzElement {
    /// Any handle into the document: a queried node has no `NodeHandle` of its
    /// own. `node_id` is what this addresses.
    pub(super) anchor: NodeHandle,
    pub(super) node_id: NodeId,
}

impl BlitzElement {
    /// A read answers when called, not awaited: the document stays borrowed
    /// while dioxus drains tasks, so a read made in a `spawn` fails.
    fn read<T: 'static>(&self, from: impl FnOnce(&BaseDocument, NodeId) -> Option<T>) -> Read<T> {
        let Some(doc) = self.anchor.free_doc() else {
            return Box::pin(std::future::ready(Err(PlatformError::Unsupported)));
        };
        let answer = from(&doc, self.node_id).ok_or(PlatformError::NotFound);
        Box::pin(std::future::ready(answer))
    }

    /// Runs now if it can, else at the end of this poll: tasks run while the
    /// document is borrowed, and `doc_mut()` there panicked (189).
    pub(super) fn command(&self, run: impl FnOnce(&mut BaseDocument) + 'static) {
        run_or_defer(&self.anchor, run);
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
        let wrapper = self::doc().and_then(|doc| doc.wrapper_id());
        let node_id = self.node_id;
        self.command(move |doc| {
            // `active_element` names the wrapper `<html>`; focus there is out
            // of every key's reach.
            let node_id = match wrapper {
                Some(wrapper) if node_id == doc.root_element().id => wrapper,
                _ => node_id,
            };
            // Blitz focuses what isn't rendered too (todo 862).
            if !is_rendered(doc, node_id) {
                return;
            }
            focus::watch_revealing(doc);
            focus::requested(node_id);
            doc.set_focus_to(node_id);
        });
        Ok(())
    }

    fn blur(&self) -> Result<(), PlatformError> {
        let node_id = self.node_id;
        self.command(move |doc| {
            if doc.get_focussed_node_id() == Some(node_id) {
                focus::watch(doc);
                doc.clear_focus();
            }
        });
        Ok(())
    }

    /// At the end of this poll, like every command here.
    fn click(&self) -> Result<(), PlatformError> {
        activate::click(&self.anchor, self.node_id);
        Ok(())
    }

    fn is_focused(&self) -> bool {
        self.anchor
            .free_doc()
            .is_some_and(|doc| doc.get_focussed_node_id() == Some(self.node_id))
    }

    /// `IS_IN_DOCUMENT`, Blitz's own `isConnected`; a missing node is gone. A
    /// borrowed document (tasks draining) answers `true`, the trait's rule.
    fn is_connected(&self) -> bool {
        let Some(doc) = self.anchor.free_doc() else {
            return true;
        };
        doc.get_node(self.node_id)
            .is_some_and(|node| node.flags.is_in_document())
    }

    fn dimensions(&self) -> Read<Dimensions> {
        self.read(|doc, node_id| {
            let (_, _, width, height) = client_rect(doc, node_id)?;
            Some(Dimensions { width, height })
        })
    }

    fn client_offset(&self) -> Read<(f64, f64)> {
        self.read(|doc, node_id| {
            let (x, y, _, _) = client_rect(doc, node_id)?;
            Some((x, y))
        })
    }

    fn client_size(&self) -> Read<Dimensions> {
        self.read(|doc, node_id| {
            let layout = doc.get_node(node_id)?.final_layout();
            let (border, bar) = (layout.border, layout.scrollbar_size);
            Some(Dimensions {
                width: client_extent(layout.size.width, border.left + border.right, bar.width),
                height: client_extent(layout.size.height, border.top + border.bottom, bar.height),
            })
        })
    }

    /// Taffy's `scroll_width` is the overflow, the furthest offset, not the content's
    /// size: the client box is added back, as the web's `scrollWidth` is (todos 652, 3172).
    fn scroll_size(&self) -> Read<Dimensions> {
        self.read(|doc, node_id| {
            let layout = doc.get_node(node_id)?.final_layout();
            let (border, bar) = (layout.border, layout.scrollbar_size);
            Some(Dimensions {
                width: client_extent(layout.size.width, border.left + border.right, bar.width)
                    + f64::from(layout.scroll_width()),
                height: client_extent(layout.size.height, border.top + border.bottom, bar.height)
                    + f64::from(layout.scroll_height()),
            })
        })
    }

    fn scroll_offset(&self) -> Read<(f64, f64)> {
        self.read(|doc, node_id| {
            let offset = doc.get_node(node_id)?.scroll_offset();
            Some((offset.x, offset.y))
        })
    }

    /// An SVG answers its CSS intrinsic size, as the web's `naturalWidth` does
    /// (todo 920); an image still loading answers `NotFound`.
    fn natural_size(&self) -> Read<Dimensions> {
        self.read(|doc, node_id| {
            let element = doc.get_node(node_id)?.element_data()?;
            let SpecialElementData::Image(image) = &element.special_data else {
                return None;
            };
            let (width, height) = match &**image {
                ImageData::Raster(raster) => (raster.width as f32, raster.height as f32),
                ImageData::Svg(svg) => svg.intrinsic_size(),
                ImageData::None => return None,
            };
            Some(Dimensions {
                width: f64::from(width),
                height: f64::from(height),
            })
        })
    }

    /// Stylo's resolved value, so a length arrives in `px` as on the web.
    fn computed_px(&self, property: &str) -> Read<Option<f64>> {
        self.read(|doc, node_id| {
            doc.get_node(node_id)?;
            let value = resolved_style_value(doc, node_id, property);
            Some(
                value
                    .strip_suffix("px")
                    .and_then(|px| px.trim().parse().ok()),
            )
        })
    }

    /// Blitz only scrolls by a delta, so this reads the current offset first -
    /// when the command runs, which may be after it was asked for.
    fn scroll_to(&self, x: f64, y: f64) -> Result<(), PlatformError> {
        if self
            .anchor
            .free_doc()
            .is_some_and(|doc| doc.get_node(self.node_id).is_none())
        {
            return Err(PlatformError::NotFound);
        }
        let node_id = self.node_id;
        self.command(move |doc| {
            let Some(offset) = doc.get_node(node_id).map(|node| *node.scroll_offset()) else {
                return;
            };
            // A wheel's sign: a positive delta scrolls towards the start.
            doc.scroll_node_by(node_id, offset.x - x, offset.y - y, |_| {});
            notify_scroll();
        });
        Ok(())
    }

    /// The web's walk over Blitz's layout; `smooth` is ignored, as Blitz only
    /// jumps.
    fn scroll_into_view(&self, _smooth: bool) -> Result<(), PlatformError> {
        show(self.anchor.clone(), self.node_id, LAYOUT_TRIES, false);
        Ok(())
    }

    /// Blitz has no `FileList`, and nothing native posts a form anyway.
    fn set_files(&self, _files: &[dioxus::html::FileData]) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    /// Blitz has no form reset: each text control and box goes back to what
    /// dioxus last wrote. A `<select>` never changes on input here.
    fn reset(&self) -> Result<(), PlatformError> {
        let form = self.node_id;
        self.command(move |doc| {
            let Ok(controls) = doc.query_selector_all_in(form, "input, textarea") else {
                return;
            };
            let defaults: Vec<_> = controls
                .into_iter()
                .filter_map(|id| {
                    let node = doc.get_node(id)?;
                    let element = node.element_data()?;
                    if element.checkbox_input_checked().is_some() {
                        let checked = checked_attr(element);
                        return Some((id, local_name!("checked"), checked.to_string()));
                    }
                    element.text_input_data()?;
                    let default = match &*element.name.local {
                        "textarea" => node.text_content(),
                        _ => element
                            .attr(local_name!("value"))
                            .unwrap_or_default()
                            .to_string(),
                    };
                    Some((id, local_name!("value"), default))
                })
                .collect();
            let mut mutator = doc.mutate();
            for (id, name, default) in defaults {
                mutator.set_attribute(id, QualName::new(None, ns!(), name), &default);
            }
        });
        Ok(())
    }

    /// Through the `value` attribute, as [`reset`](Self::reset) writes it:
    /// Blitz's text input follows it.
    fn set_value(&self, value: &str) -> Result<(), PlatformError> {
        let node_id = self.node_id;
        let value = value.to_string();
        self.command(move |doc| {
            let text = doc
                .get_node(node_id)
                .and_then(|node| node.element_data()?.text_input_data())
                .is_some();
            if text {
                let name = QualName::new(None, ns!(), local_name!("value"));
                doc.mutate().set_attribute(node_id, name, &value);
            }
        });
        Ok(())
    }

    fn attribute(&self, name: &str) -> Result<Option<String>, PlatformError> {
        let doc = self.anchor.free_doc().ok_or(PlatformError::Unsupported)?;
        let element = doc
            .get_node(self.node_id)
            .and_then(|node| node.element_data())
            .ok_or(PlatformError::NotFound)?;
        Ok(element
            .attrs()
            .iter()
            .find(|attr| &*attr.name.local == name)
            .map(|attr| attr.value.clone()))
    }

    fn matches(&self, selector: &str) -> Result<bool, PlatformError> {
        let doc = self.anchor.free_doc().ok_or(PlatformError::Unsupported)?;
        let found = doc
            .query_selector_all(selector)
            .map_err(|_| PlatformError::NotFound)?;
        Ok(found.contains(&self.node_id))
    }

    /// The editor counts bytes; the DOM counts UTF-16 units.
    fn select(&self) -> Result<(), PlatformError> {
        let node_id = self.node_id;
        self.command(move |doc| doc.with_text_input(node_id, |mut driver| driver.select_all()));
        Ok(())
    }

    fn selection_start(&self) -> Option<u32> {
        let doc = self.anchor.free_doc()?;
        let editor = &doc
            .get_node(self.node_id)?
            .element_data()?
            .text_input_data()?
            .editor;
        let start = editor.raw_selection().text_range().start;
        let units = editor.raw_text().get(..start)?.encode_utf16().count();
        u32::try_from(units).ok()
    }

    fn set_caret(&self, at: u32) -> Result<(), PlatformError> {
        let doc = self.anchor.free_doc().ok_or(PlatformError::Unsupported)?;
        let editor = &doc
            .get_node(self.node_id)
            .and_then(|node| node.element_data())
            .and_then(|element| element.text_input_data())
            .ok_or(PlatformError::NotFound)?
            .editor;
        let text = editor.raw_text();
        let mut units = 0;
        let byte = text
            .char_indices()
            .find(|(_, char)| {
                let reached = units >= at as usize;
                units += char.len_utf16();
                reached
            })
            .map_or(text.len(), |(byte, _)| byte);
        drop(doc);
        let node_id = self.node_id;
        self.command(move |doc| {
            doc.with_text_input(node_id, |mut driver| driver.move_to_byte(byte))
        });
        Ok(())
    }

    /// A pre-order walk down to this node: everything it visits first comes
    /// before it, ancestors included, and its own subtree comes after.
    fn previous_focusable(
        &self,
        selector: &str,
    ) -> Result<Option<Box<dyn ElementApi>>, PlatformError> {
        let doc = self.anchor.free_doc().ok_or(PlatformError::Unsupported)?;
        let matches = doc
            .query_selector_all(selector)
            .map_err(|_| PlatformError::NotFound)?;
        let mut before = std::collections::HashSet::new();
        let mut stack = vec![doc.root_element().id];
        let mut found = false;
        while let Some(id) = stack.pop() {
            if id == self.node_id {
                found = true;
                break;
            }
            before.insert(id);
            if let Some(node) = doc.get_node(id) {
                stack.extend(node.children.iter().rev());
            }
        }
        drop(doc);
        if !found {
            return Err(PlatformError::NotFound);
        }
        let node_id = matches.into_iter().rev().find(|id| before.contains(id));
        Ok(node_id.map(|node_id| self.at(node_id)))
    }

    /// Blitz fires no submit event from code.
    fn request_submit(&self) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    /// Blitz has no pointer capture. `use_drag` keeps tracking outside its
    /// element through [`follow_pointer`](super::follow_pointer) instead.
    fn set_pointer_capture(&self, _pointer_id: i32) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    fn query_selector(&self, selector: &str) -> Result<Box<dyn ElementApi>, PlatformError> {
        let doc = self.anchor.free_doc().ok_or(PlatformError::Unsupported)?;
        // A dropped node panics in Blitz's query (#1185).
        doc.get_node(self.node_id).ok_or(PlatformError::NotFound)?;
        let node_id = doc
            .query_selector_in(self.node_id, &activate::focus_selectors(selector))
            .map_err(|_| PlatformError::NotFound)?
            .ok_or(PlatformError::NotFound)?;
        drop(doc);
        Ok(self.at(node_id))
    }

    fn query_selector_all(
        &self,
        selector: &str,
    ) -> Result<Vec<Box<dyn ElementApi>>, PlatformError> {
        let doc = self.anchor.free_doc().ok_or(PlatformError::Unsupported)?;
        doc.get_node(self.node_id).ok_or(PlatformError::NotFound)?;
        let nodes = doc
            .query_selector_all_in(self.node_id, &activate::focus_selectors(selector))
            .map_err(|_| PlatformError::NotFound)?;
        drop(doc);
        Ok(nodes.into_iter().map(|node_id| self.at(node_id)).collect())
    }
}
