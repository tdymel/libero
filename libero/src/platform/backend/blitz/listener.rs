use std::{cell::Cell, rc::Rc};

use dioxus::core::AttributeValue;
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;
use dioxus_native_dom::{NodeHandle, NodeId};

use super::{
    BLANK_PRESS, HELD, HIT, PRESS, activate, defer, doc, focus, followed, forget_press, keyed,
    max_length, notify_scroll, placeholder, pressed, refused, removed, resize, tab_from_start,
    wheel,
};

/// Wraps the app, so a press's target is known until its click bubbled and keys
/// reach [`BlitzKeyboard`]. Blitz runs click handlers before moving focus.
#[component]
pub(in crate::platform::backend) fn Listener(children: Element) -> Element {
    let doc = use_hook(doc);
    let own = use_hook(|| Rc::new(Cell::new(None::<NodeId>)));
    use_drop({
        let (doc, own) = (doc.clone(), own.clone());
        // A nested provider's wrapper may have been replaced by the outer one.
        move || {
            if let Some(doc) = doc
                && own.get().is_some()
                && doc.wrapper_id() == own.get()
            {
                doc.wrapper.take();
            }
        }
    });
    // Raw, so a double click's second keeps its data for the `dblclick` it fires.
    let onclick = AttributeValue::listener(|raw: Event<PlatformEventData>| {
        let event = raw.map(|data| MouseData::from(data));
        // A kept press on nothing focusable keeps focus too, as on the web.
        let (kept, press) = (focus::press_kept_focus(), PRESS.get());
        let moving = press.filter(|_| !kept);
        let taken = press.is_some() || kept || activate::press_cancelled();
        // A prevented click skips Blitz's focus move; the web moved it at the press (3883).
        let prevented = !event.default_action_enabled();
        if taken && (activate::takes_over(&event, &raw, HIT.get()) || prevented) {
            if let Some((before, target)) = moving {
                activate::focus_pressed(before, target);
            }
        } else if let Some((before, target)) = moving {
            activate::clicked(before, target);
        }
        forget_press();
        focus::clicked();
        if BLANK_PRESS.take() {
            refocus_wrapper();
        }
        resize::pressed();
        placeholder::sync_soon();
    });
    // Raw too, so a nested provider's wrapper knows the press it already counted.
    let onpointerdown = AttributeValue::listener(|raw: Event<PlatformEventData>| {
        let event = raw.map(|data| PointerData::from(data));
        HELD.set(true);
        activate::pointer_down(&event, &raw.data);
        refused::pressed(&event);
        pressed(&event);
    });
    let raw = vec![
        Attribute::new("onclick", onclick, None, false),
        Attribute::new("onpointerdown", onpointerdown, None, false),
    ];
    rsx! {
        div {
            display: "contents",
            tabindex: "-1",
            onmounted: move |event| {
                if let (Some(doc), Some(handle)) = (&doc, event.data().downcast::<NodeHandle>()) {
                    own.set(Some(handle.node_id()));
                    *doc.wrapper.borrow_mut() = Some(handle.clone());
                }
            },
            // Bubble phase, `onpointerdown` in `raw`: this dioxus has no capture listeners.
            onmousedown: |event| focus::mouse_pressed(&event),
            onkeydown: |event| {
                forget_press();
                BLANK_PRESS.set(false);
                focus::forget_kept();
                focus::keyed(&event);
                max_length::key_down(&event);
                keyed(&event);
                tab_from_start(&event);
                removed::keyed(&event);
                refused::tab(&event);
                activate::key_down(&event);
            },
            onkeyup: |event| {
                activate::key_up(&event);
                resize::pressed();
                placeholder::sync_soon();
            },
            oninput: |_| {
                max_length::input();
                placeholder::sync_now();
            },
            onpointermove: move |event| {
                // A release off the app is never seen here; the next move is.
                if event.held_buttons().is_empty() {
                    HELD.set(false);
                }
                followed(&event, false);
            },
            onpointerup: move |event| {
                HELD.set(false);
                followed(&event, true);
                focus::released();
                refused::release(&event);
                resize::pressed();
            },
            // A wheel bubbles where its scroll does not: see `BlitzScroll`.
            onwheel: |event| {
                wheel::wheeled(&event);
                notify_scroll();
            },
            ..raw,
            {children}
        }
    }
}

/// Blitz focuses `<html>` after a click on nothing focusable, where no key would
/// reach [`BlitzKeyboard`](super::keyboard::BlitzKeyboard): taken back at the end of the poll.
pub(super) fn refocus_wrapper() {
    let Some(wrapper) = doc().and_then(|doc| doc.wrapper.borrow().clone()) else {
        return;
    };
    let node_id = wrapper.node_id();
    defer(&wrapper, move |doc| {
        let root = doc.root_element().id;
        if doc
            .get_focussed_node_id()
            .is_none_or(|focused| focused == root)
        {
            doc.set_focus_to(node_id);
        }
    });
}
