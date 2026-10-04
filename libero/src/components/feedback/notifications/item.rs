use std::{cell::RefCell, rc::Rc, time::Duration};

use dioxus::prelude::*;

use super::{
    data::{Draw, NotificationId, NotificationLive},
    store::NotificationStore,
};
use crate::{
    components::{
        common::{FOCUSABLE_SELECTOR, HtmlTag, Input, States, focus_ring_sx},
        layout::use_box,
    },
    hooks::use_focus_within,
    platform::{self, ElementApi, TimerSubscription, timer},
    sx::{REDUCED_MOTION, StaticSx, sx},
    theme::{NOTIFICATION_IN, NOTIFICATION_OUT, NOTIFICATION_TRANSITION},
};

static ITEM_SX: StaticSx = StaticSx::new(|| {
    let animation =
        |name: &str, easing: &str| format!("{name} {} {easing}", NOTIFICATION_TRANSITION.value());

    sx().pointer_events("auto")
        .selector("&:focus-visible", focus_ring_sx())
        .animation(animation(NOTIFICATION_IN, "ease-out"))
        .media(REDUCED_MOTION, sx().animation("none"))
        // Declared, not only animated to: it holds until the unmount, and is
        // where reduced motion lands at once.
        .when(
            "leaving",
            sx().opacity("0")
                .visibility("hidden")
                .pointer_events("none")
                .animation(animation(NOTIFICATION_OUT, "ease-in"))
                // A delayed flip, not a keyframe: an animated `visibility` keeps the
                // fade off the compositor and restyles every frame (todo 2159).
                .transition(format!("visibility 0s {}", NOTIFICATION_TRANSITION.value()))
                .media(REDUCED_MOTION, sx().animation("none").transition("none")),
        )
});

/// Compared by pointer: a closure has no other equality.
#[derive(Clone)]
pub(super) struct DrawRef(pub(super) Draw);

impl PartialEq for DrawRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Props, Clone, PartialEq)]
pub(super) struct ItemProps {
    /// Passed, not looked up: a lookup depends on where the item renders.
    pub(super) store: NotificationStore,
    pub(super) id: NotificationId,
    pub(super) draw: DrawRef,
    /// Resolved against the host: `None` stays until closed.
    pub(super) auto_close: Option<u32>,
    pub(super) leaving: bool,
    pub(super) exit_ms: u32,
    pub(super) live: NotificationLive,
}

/// The last focusable before the contained host `host` selects, `item` being
/// inside it.
fn focusable_before(
    item: &Rc<MountedData>,
    host: &str,
) -> Result<Option<std::boxed::Box<dyn ElementApi>>, crate::platform::PlatformError> {
    platform::element(item).previous_focusable(&format!("{FOCUSABLE_SELECTOR}:not({host} *)"))
}

pub(super) fn NotificationItem(props: ItemProps) -> Element {
    let store = props.store;
    let id = props.id;

    use_hook(|| {
        if let Some(entry) = store.entries.peek().iter().find(|entry| entry.id == id) {
            entry.shown.set(true);
        }
    });

    // Dropping a subscription cancels it. Held here, so an unmount takes its timer.
    let subscription =
        use_hook(|| Rc::new(RefCell::new(None::<std::boxed::Box<dyn TimerSubscription>>)));

    let leaving = props.leaving;
    let auto_close = props.auto_close;
    let exit_ms = props.exit_ms;
    let armed = subscription.clone();
    use_effect(use_reactive!(|(leaving, auto_close, exit_ms)| {
        // Read in the effect, not the render: a pause re-arms timers without a redraw.
        let paused = store.paused();
        let mut armed = armed.borrow_mut();
        *armed = None;

        // The callbacks only write signals: on the web they run with no runtime.
        let (delay, then): (u32, fn(NotificationStore, NotificationId)) = match auto_close {
            _ if leaving => (exit_ms, |store, id| store.remove(id)),
            // Resumes with the full time, not the remainder.
            Some(_) if paused => return,
            Some(ms) => (ms, |store, id| store.hide(id)),
            None => return,
        };
        if let Some(api) = timer() {
            *armed = Some(api.after(
                Duration::from_millis(delay.into()),
                std::boxed::Box::new(move || then(store, id)),
            ));
        }
    }));

    // Its own element, held here: the store's copy goes with a contained host.
    let own = use_hook(|| Rc::new(RefCell::new(None::<Rc<MountedData>>)));
    let dropped = own.clone();
    // What precedes a contained host, found as focus enters. Blitz holds its
    // document through the unmount, so the drop cannot ask then.
    let before_host = use_hook(|| Rc::new(RefCell::new(None::<std::boxed::Box<dyn ElementApi>>)));
    let remembered = before_host.clone();
    use_drop(move || {
        subscription.borrow_mut().take();
        // Removed under the pointer or focus, it never sees `mouseleave`/`focusout`.
        let (mut hovered, mut focused) = (store.hovered, store.focused);
        if *hovered.peek() == Some(id) {
            hovered.set(None);
        }
        if *focused.peek() == Some(id) {
            focused.set(None);
            // Now, not spawned: a contained host going too takes the scope.
            let return_to = store.return_to.try_peek().ok().and_then(|to| to.clone());
            let return_to = return_to.filter(|to| to.is_connected());
            // An entry that outlives its item: the host itself is going.
            let host_going = store.host_selector().filter(|_| {
                store
                    .entries
                    .try_peek()
                    .map_or(true, |entries| entries.iter().any(|entry| entry.id == id))
            });
            let in_host = store
                .return_in_host
                .try_peek()
                .map_or(true, |in_host| *in_host);
            let before = host_going
                .filter(|_| in_host || return_to.is_none())
                .and_then(|host| {
                    let own = dropped.borrow().clone()?;
                    match focusable_before(&own, &host) {
                        Ok(before) => before,
                        Err(_) => remembered.borrow_mut().take(),
                    }
                });
            // Nothing before the host, or no way to ask: where focus came from.
            match (before, return_to) {
                (Some(before), _) => {
                    let _ = before.focus();
                }
                (None, Some(target)) => {
                    let _ = target.focus();
                }
                (None, None) => {}
            }
        }
        let mut elements = store.elements;
        elements.write().remove(&id);
    });
    let item = move || store.elements.peek().get(&id).cloned();
    let focus = use_focus_within(
        move || vec![item()],
        move |change| {
            let mut focused = store.focused;
            if !change.within {
                if *focused.peek() == Some(id) {
                    focused.set(None);
                }
                return;
            }
            if let (Some(host), Some(mounted)) = (store.host_selector(), item()) {
                *before_host.borrow_mut() = focusable_before(&mounted, &host).ok().flatten();
            }
            if !*store.handing_off.peek()
                && let Some(from) = change.entered_from("[data-notification]")
            {
                // The host as the boundary answers `None` for a `from` inside it.
                let in_host = store
                    .host_selector()
                    .is_some_and(|host| from.is_some() && change.entered_from(&host).is_none());
                let (mut return_in_host, mut return_to) = (store.return_in_host, store.return_to);
                return_in_host.set(in_host);
                return_to.set(from.map(Rc::from));
            }
            focused.set(Some(id))
        },
    );

    let states: Input<States> = States::default().with("leaving", leaving).into();
    let mut hovered = store.hovered;
    let mut elements = store.elements;

    use_box()
        .framework_sx(&ITEM_SX)
        .states(&states)
        .prepare()
        .attr("data-notification", true)
        // The hotkey's target when nothing inside takes focus.
        .attr("tabindex", "-1")
        .event("onmounted", move |event: Event<MountedData>| {
            own.replace(Some(event.data()));
            elements.write().insert(id, event.data());
        })
        .event("onmouseenter", move |_: Event<MouseData>| {
            hovered.set(Some(id))
        })
        .event("onmouseleave", move |_: Event<MouseData>| {
            if *hovered.peek() == Some(id) {
                hovered.set(None);
            }
        })
        .event("onfocusin", focus.focusin(0))
        .event("onfocusout", focus.focusout(0))
        .render(
            HtmlTag::Li,
            Vec::new(),
            (props.draw.0)(auto_close.is_none()),
        )
}
