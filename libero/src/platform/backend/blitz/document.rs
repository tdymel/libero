use blitz_dom::{QualName, ns};

use super::{BlitzElement, PRESS, anchor, doc, run_or_defer};
use crate::platform::{Dimensions, DocumentApi, ElementApi, PlatformError, Read};

pub(in crate::platform::backend) fn document() -> Option<&'static dyn DocumentApi> {
    anchor().map(|_| &DOCUMENT as &'static dyn DocumentApi)
}

pub(super) struct BlitzDocument;

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
        let doc = anchor.try_doc()?;
        let focused = doc.get_focussed_node_id();
        // Focus unmoved since a press means Blitz has yet to move it there.
        let pressed = PRESS.get().filter(|&(before, target)| {
            before == focused
                && doc
                    .get_node(target)
                    .is_some_and(|node| node.flags.is_in_document())
        });
        // The wrapper holds focus for nothing: `<html>`, as Blitz answers then.
        let wrapper = self::doc().and_then(|doc| doc.wrapper_id());
        let node_id = match pressed {
            Some((_, target)) => target,
            None if focused == wrapper => doc.root_element().id,
            None => focused?,
        };
        drop(doc);
        Some(Box::new(BlitzElement { anchor, node_id }))
    }

    /// On the `<html>` `root_element()` reaches; the mutator's write restyles
    /// it (todo 480).
    fn set_root_attribute(&self, name: &str, value: Option<&str>) -> bool {
        let Some(anchor) = anchor() else {
            return false;
        };
        let name = QualName::new(None, ns!(), name.into());
        let value = value.map(str::to_string);
        run_or_defer(&anchor, move |doc| {
            let root = doc.root_element().id;
            let mut mutator = doc.mutate();
            match value {
                Some(value) => mutator.set_attribute(root, name, &value),
                None => mutator.clear_attribute(root, name),
            }
        });
        true
    }
}
