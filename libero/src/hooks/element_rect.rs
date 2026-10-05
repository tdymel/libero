//! An element's box in viewport coordinates, kept current on scroll and resize:
//! `use_popover`'s measure, without a floating box.

use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    hooks::{ElementHandle, Rect, use_subscription_slot},
    platform::{
        Dimensions, ElementApi, ScrollSubscription, document, next_task, on_viewport_resize,
        scroll, when_laid_out,
    },
    utils::bump,
};

/// How many laid-out turns a measure waits for an unmounted or 0x0 element.
const UNLAID_TRIES: u8 = 3;

/// What [`use_element_rect`] knows of its element.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ElementRect {
    /// Not measured yet, or not active.
    Pending,
    /// No element, or still unmounted after [`UNLAID_TRIES`] laid-out turns.
    Missing,
    At {
        rect: Rect,
        viewport: Dimensions,
    },
}

/// Measures `target` while `active`, again on every scroll and viewport resize.
/// Scrolls are heard where the renderer reports them (Blitz: libero's own and the wheel).
pub(crate) fn use_element_rect(
    target: Option<ElementHandle>,
    active: bool,
) -> ReadSignal<ElementRect> {
    let state = use_signal(|| ElementRect::Pending);
    // Bumped from platform callbacks, outside every scope; the effect measures.
    let tick = use_signal(|| 0u64);
    let listening = use_subscription_slot::<dyn ScrollSubscription>();
    let resizing = use_subscription_slot::<dyn ScrollSubscription>();
    let waited: Rc<Cell<u8>> = use_hook(|| Rc::new(Cell::new(0)));
    let measured: Rc<Cell<Option<ElementHandle>>> = use_hook(|| Rc::new(Cell::new(None)));

    use_effect(use_reactive!(|(target, active)| {
        let _ = tick();
        // A new target gets its own tries, not the ones the last one spent.
        if measured.replace(target) != target {
            waited.set(0);
        }
        // Read before any return: a (re)mount re-runs this.
        let mounted = target.is_some_and(|target| target.mount_token().is_some());
        let set = move |next: ElementRect| {
            let mut state = state;
            if *state.peek() != next {
                state.set(next);
            }
        };
        let Some(target) = target.filter(|_| active) else {
            listening.clear();
            resizing.clear();
            waited.set(0);
            set(match active {
                true => ElementRect::Missing,
                false => ElementRect::Pending,
            });
            return;
        };
        if !mounted {
            let tries = waited.get();
            if tries < UNLAID_TRIES {
                waited.set(tries + 1);
                // The web lays out at once: a bump inside this run would not run it again.
                spawn(async move {
                    next_task().await;
                    when_laid_out(move || bump(tick));
                });
            } else {
                set(ElementRect::Missing);
            }
            return;
        }
        let Some(document) = document() else {
            return;
        };
        if let Some(api) = (!listening.is_some()).then(scroll).flatten() {
            listening.set(Some(api.on_scroll(Box::new(move || bump(tick)))));
        }
        if !resizing.is_some() {
            resizing.set(on_viewport_resize(Box::new(move || bump(tick))));
        }

        // Started here, awaited in the task: Blitz locks the document while tasks drain.
        let size = target.dimensions();
        let offset = target.client_offset();
        let viewport = document.viewport();
        let waited = waited.clone();
        spawn(async move {
            let (Ok(size), Ok((x, y)), Ok(viewport)) = (size.await, offset.await, viewport.await)
            else {
                return;
            };
            // Mounted this poll on a native shell: 0x0 until laid out.
            let tries = waited.get();
            if size.width == 0.0 && size.height == 0.0 && tries < UNLAID_TRIES {
                waited.set(tries + 1);
                when_laid_out(move || bump(tick));
                return;
            }
            let rect = Rect {
                x,
                y,
                width: size.width,
                height: size.height,
            };
            set(ElementRect::At { rect, viewport });
        });
    }));

    state.into()
}
