//! dioxus-native writes `hidden="false"`, which Blitz reads as set; [`sync`]
//! drops it at each flush, as the web's interpreter does (943).

use blitz_dom::{BaseDocument, QualName};
use dioxus_native_dom::NodeId;

// The web interpreter's `isBoolAttr`, less `checked`: dioxus-native clears that one.
const FLAGS: [&str; 25] = [
    "allowfullscreen",
    "allowpaymentrequest",
    "async",
    "autofocus",
    "autoplay",
    "controls",
    "default",
    "defer",
    "disabled",
    "formnovalidate",
    "hidden",
    "ismap",
    "itemscope",
    "loop",
    "multiple",
    "muted",
    "nomodule",
    "novalidate",
    "open",
    "playsinline",
    "readonly",
    "required",
    "reversed",
    "selected",
    "truespeed",
];

/// Removes every bool attribute whose value is `"false"`.
pub(super) fn sync(doc: &mut BaseDocument) {
    let mut found: Vec<(NodeId, QualName)> = Vec::new();
    doc.visit(|id, node| {
        let Some(element) = node.element_data() else {
            return;
        };
        for attr in element.attrs.iter() {
            if &*attr.value == "false" && FLAGS.contains(&&*attr.name.local) {
                found.push((id, attr.name.clone()));
            }
        }
    });
    if found.is_empty() {
        return;
    }
    let mut mutator = doc.mutate();
    for (id, name) in found {
        mutator.clear_attribute(id, name);
    }
}
