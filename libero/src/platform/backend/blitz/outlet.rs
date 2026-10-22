use std::{cell::Cell, rc::Rc};

use blitz_dom::BaseDocument;
use dioxus::prelude::*;
use dioxus_native_dom::NodeHandle;

use super::{
    OutletSlot, activate, check_scheme, doc, focus, placeholder, redraw, refocus_wrapper, removed,
    resize, run_deferred, run_or_defer, sticky,
};

thread_local! {
    pub(super) static NEXT_OUTLET: Cell<u64> = const { Cell::new(0) };
}

/// Mounted once by `LiberoProvider`: its first mount anchors the document, later
/// ones run deferred commands.
///
/// `onmounted` is the one callback with the document free: deferring bumps
/// `flushes`, the keyed child remounts and runs the queue at the end of the poll.
#[component]
pub(in crate::platform::backend) fn Outlet() -> Element {
    let doc = use_hook(doc);
    let id = use_hook(|| NEXT_OUTLET.replace(NEXT_OUTLET.get() + 1));
    let flushes = use_hook(|| {
        let flushes = Signal::new(0u64);
        if let Some(doc) = &doc {
            doc.outlets.borrow_mut().push(OutletSlot {
                id,
                flushes,
                mounted: None,
                handle: None,
            });
        }
        flushes
    });
    use_drop({
        let doc = doc.clone();
        move || {
            let Some(doc) = doc else {
                return;
            };
            let slot = {
                let mut outlets = doc.outlets.borrow_mut();
                let at = outlets.iter().position(|outlet| outlet.id == id);
                at.map(|at| outlets.remove(at))
            };
            drop(slot);
            // The last one gone: no handle is kept past the document.
            if doc.outlets.borrow().is_empty() {
                doc.seen.take();
            }
        }
    });

    rsx! {
        for flush in [flushes()] {
            div {
                key: "{flush}",
                display: "none",
                // Blitz hits a display:none box at its 0x0 spot: a pointer resting there flips
                // the hover to each remounted div, whose redraw flushes again, forever.
                visibility: "hidden",
                onmounted: {
                    let doc = doc.clone();
                    move |event: MountedEvent| {
                    let Some(doc) = &doc else {
                        return;
                    };
                    let handle = event.data().downcast::<NodeHandle>().cloned();
                    if let Some(outlet) = doc.outlets.borrow_mut().iter_mut().find(|outlet| outlet.id == id) {
                        outlet.mounted = Some(flush);
                        if outlet.handle.is_none() {
                            outlet.handle = handle;
                        }
                    }
                    redraw::quiet(|| {
                        run_deferred(doc);
                        removed::check(doc);
                        focus::check(doc);
                        resize::check(doc);
                        if let Some(anchor) = doc.anchor() {
                            redraw::watch(doc, &mut anchor.doc_mut());
                            activate::sync_marks(&mut anchor.doc_mut());
                            placeholder::sync(&mut anchor.doc_mut());
                            // After layout: a `bottom` box moves before any scroll.
                            sticky::sync_soon();
                            // Last: after every focus move this flush made.
                            if doc.sheets_changed.take() {
                                snapshot_attributes(&mut anchor.doc_mut());
                            }
                        }
                    });
                    // Once, at the provider's mount: see `BlitzColorScheme`. A
                    // first key then reaches the wrapper, as it does after a click.
                    if flush == 0 {
                        check_scheme(doc);
                        refocus_wrapper();
                    }
                }},
            }
        }
    }
}

/// A sheet change in a frame with a state change panics in stylo's state-only
/// snapshot (837): at this poll's end every snapshot takes the attributes too.
#[component]
pub(in crate::platform::backend) fn SheetWatch(version: u64) -> Element {
    // (key, mounted key, version seen): one remount per poll, see `later`.
    let state = use_hook(|| Rc::new(Cell::new((0u64, None::<u64>, version))));
    let (mut key, mounted, seen) = state.get();
    if version != seen && mounted == Some(key) {
        key += 1;
    }
    state.set((key, mounted, version));
    rsx! {
        for key in [key] {
            div {
                key: "{key}",
                display: "none",
                visibility: "hidden",
                onmounted: {
                    let state = state.clone();
                    move |event: MountedEvent| {
                    let (_, _, seen) = state.get();
                    state.set((key, Some(key), seen));
                    let Some(handle) = event.data().downcast::<NodeHandle>().cloned() else {
                        return;
                    };
                    if let Some(doc) = doc() {
                        doc.sheets_changed.set(true);
                    }
                    run_or_defer(&handle, snapshot_attributes);
                }},
            }
        }
    }
}

fn snapshot_attributes(doc: &mut BaseDocument) {
    let mut snapshotted = Vec::new();
    doc.visit(|id, node| {
        if node.has_snapshot() {
            snapshotted.push(id);
        }
    });
    for id in snapshotted {
        doc.snapshot_node(id);
    }
}
