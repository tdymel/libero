use dioxus::prelude::*;
use dioxus_native_dom::NodeHandle;

use super::doc;

/// The portal outlet: a hitless viewport box; [`PortalEntry`] takes the hits.
/// No `z-index`: Blitz paints and hit-tests a z-indexed `fixed` box scrolled.
const PORTAL_ROOT_STYLE: &str = "position:fixed;inset:0;pointer-events:none;";

/// A hitless viewport-wide box: Blitz hit-tests z-indexed boxes only in their
/// untransformed union. During a [`follow_pointer`](super::follow_pointer) drag it takes the hits.
pub(super) const HIT_AREA_STYLE: &str = "position:absolute;inset:0;z-index:1;pointer-events:none;";

#[component]
pub(in crate::platform::backend) fn PortalRoot(children: Element) -> Element {
    let doc = use_hook(doc);
    rsx! {
        div { style: PORTAL_ROOT_STYLE,
            div {
                style: HIT_AREA_STYLE,
                onmounted: move |event| {
                    if let (Some(doc), Some(handle)) = (&doc, event.data().downcast::<NodeHandle>()) {
                        *doc.hit_area.borrow_mut() = Some(handle.clone());
                    }
                },
            }
            {children}
        }
    }
}

/// One portaled entry, taking hits again below [`PORTAL_ROOT_STYLE`].
#[component]
pub(in crate::platform::backend) fn PortalEntry(children: Element) -> Element {
    rsx! {
        div { display: "contents", pointer_events: "auto", {children} }
    }
}
