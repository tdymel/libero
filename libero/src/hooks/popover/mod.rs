//! Anchored, portaled floating boxes: a dropdown that escapes an
//! `overflow: hidden` ancestor and flips when it runs out of room.

mod options;
mod owners;
mod place;

use dioxus::prelude::*;

pub use options::{Align, Placement, PopoverOptions, PopoverWidth, Side};
pub(crate) use place::{Placed, Rect};

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::{
    hooks::{
        ElementHandle,
        dismiss::{DismissHandle, DismissOptions, use_dismiss},
        element::use_element,
        portal::{PortalSlot, use_portal_slot},
    },
    platform::{ElementApi, ScrollSubscription, document, scroll, when_laid_out},
    theme::CssVar,
};

#[cfg(test)]
pub(crate) use owners::open_popups;
pub(crate) use owners::{OpenPopups, focus_in_popup_of, owner_link, use_open_popups};
pub(crate) use place::place;

/// The room on the box's side, set by [`PopoverHandle::style`]: a box that can
/// grow tall takes `max-height: min(<own>, var(.., <own>))` and scrolls (WCAG 1.4.10).
pub(crate) const AVAILABLE_HEIGHT: CssVar = CssVar::new("--lsx-popover-available-height");

/// A popover's own state: where its box goes, and the portal slot it goes in.
/// Built by [`use_popover`]; `Copy`, so it travels into handlers.
#[derive(Clone, Copy)]
pub struct PopoverHandle {
    anchor: ElementHandle,
    floating: ElementHandle,
    placed: Signal<Option<Placed>>,
    /// The anchor's measured width, which is what `PopoverWidth` follows.
    anchor_width: Signal<Option<f64>>,
    width: PopoverWidth,
    /// The collision padding, which the width cap leaves at both edges.
    padding: f64,
    slot: PortalSlot,
    /// `None` from `use_popover_on`, whose consumers run their own `use_dismiss`.
    dismiss: Option<PopoverDismiss>,
}

/// [`PopoverOptions::dismiss`]'s wiring: `use_dismiss` plus the caller's close.
#[derive(Clone, Copy)]
struct PopoverDismiss {
    handle: DismissHandle,
    onclose: CopyValue<Option<Box<dyn FnMut()>>>,
    /// Open with `dismiss` on.
    active: bool,
}

impl PopoverHandle {
    /// The handle to put on the floating box - `.element(popover.floating())` -
    /// so it can be measured. Nothing is placed until it is attached.
    pub fn floating(&self) -> &ElementHandle {
        &self.floating
    }

    /// Whether the box has been measured and placed. `false` on the pass that
    /// opens it, because measuring needs it rendered first.
    pub fn placed(&self) -> bool {
        self.placed.read().is_some_and(|placed| !placed.provisional)
    }

    /// The top it was placed at, which a stylesheet's `!important` may have overridden.
    pub(crate) fn placed_top(&self) -> Option<f64> {
        self.placed.peek().as_ref().map(|placed| placed.y)
    }

    /// Where the box landed. The preferred side and align until it has been
    /// measured, so a skin drawing an arrow never sees a placement of `None`.
    pub fn placement(&self) -> Placement {
        self.placed
            .read()
            .as_ref()
            .map(|placed| placed.placement)
            .unwrap_or_default()
    }

    /// The box's `style`: `position: fixed` in viewport coordinates, and
    /// `visibility: hidden` (measurable, unlike `display: none`) until placed.
    /// Sets [`AVAILABLE_HEIGHT`], `100vh` until placed, which a box that can grow tall caps itself with.
    ///
    /// Every declaration is emitted on every render: a renderer never removes
    /// one that stops being printed. The CSS width cap keeps it on a phone (todo 357).
    pub fn style(&self) -> String {
        let (x, y, visibility, available) = match *self.placed.read() {
            Some(placed) => (
                placed.x,
                placed.y,
                if placed.provisional {
                    "hidden"
                } else {
                    "visible"
                },
                format!("{}px", placed.available_height),
            ),
            None => (0.0, 0.0, "hidden", String::from("100vh")),
        };
        // `auto` until the anchor is measured: a `width: 0` box would measure
        // the height of its text wrapped into nothing.
        let (width, min_width) = match (self.width, *self.anchor_width.read()) {
            (PopoverWidth::Match, Some(anchor)) => (format!("{anchor}px"), String::from("auto")),
            (PopoverWidth::Min, Some(anchor)) => (String::from("auto"), format!("{anchor}px")),
            _ => (String::from("auto"), String::from("auto")),
        };

        let edges = 2.0 * self.padding;

        format!(
            "position:fixed;left:{x}px;top:{y}px;width:{width};min-width:{min_width};max-width:calc(100vw - {edges}px);visibility:{visibility};{}:{available};",
            AVAILABLE_HEIGHT.name()
        )
    }

    /// Portals the box, `None` takes it away. Already rendered, so its reads
    /// happen in the caller's scope, not the outlet's.
    pub fn show(&self, content: Option<Element>) {
        self.slot.show(content);
    }

    /// What Escape and a press outside call, with [`PopoverOptions::dismiss`]
    /// on. Call it on every render; the latest closure wins.
    pub fn on_dismiss(&self, onclose: impl FnMut() + 'static) {
        if let Some(dismiss) = self.dismiss {
            let mut slot = dismiss.onclose;
            slot.set(Some(Box::new(onclose)));
        }
    }

    /// Spread on the trigger: the press-outside check, and Escape where the
    /// document cannot be listened to, while open with `dismiss` on. On a
    /// WebView, also the link that lets [`Hotkey::within`](crate::hooks::Hotkey::within)
    /// count focus in the box as inside the trigger's scope.
    pub fn anchor_events(&self) -> Vec<Attribute> {
        let mut events = self.anchor.attributes();
        if let Some(dismiss) = self.dismiss.filter(|dismiss| dismiss.active) {
            events.extend(dismiss.handle.anchor_events());
            events.push(dismiss.handle.focusout_listener());
        }
        events
    }

    /// Spread on the box, as [`anchor_events`](Self::anchor_events) on the
    /// trigger; the WebView link needs both.
    pub fn floating_events(&self) -> Vec<Attribute> {
        let mut events = owners::owner_link(&self.anchor);
        if let Some(dismiss) = self.dismiss {
            events.extend(dismiss.handle.floating_events());
        }
        events
    }
}

/// Anchors a portaled box to `anchor`, flipping and shifting it to stay in the
/// viewport.
///
/// Measured per open, on `options` changes and on scroll (web only: elsewhere
/// it drifts). It owns no open state; [`PopoverOptions::dismiss`] adds Escape
/// and a press outside, and Escape hands focus to `anchor`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::{PopoverOptions, use_element, use_popover};
/// # fn app() -> Element {
/// let anchor = use_element();
/// let mut opened = use_signal(|| false);
/// let popover = use_popover(anchor, opened(), PopoverOptions::new(4.0, 8.0).dismiss(true));
/// popover.on_dismiss(move || opened.set(false));
/// popover.show(opened().then(|| rsx! {
///     div {
///         onmounted: popover.floating().mount(),
///         style: popover.style(),
///         tabindex: "-1",
///         ..popover.floating_events(),
///         "Content"
///     }
/// }));
///
/// rsx! {
///     button {
///         onmounted: anchor.mount(),
///         onclick: move |_| opened.toggle(),
///         ..popover.anchor_events(),
///         "Open"
///     }
/// }
/// # }
/// ```
pub fn use_popover(anchor: ElementHandle, open: bool, options: PopoverOptions) -> PopoverHandle {
    let mut popover = use_popover_on(anchor, use_element(), open, options);

    let onclose = use_hook(|| CopyValue::new(None::<Box<dyn FnMut()>>));
    let close = use_callback(move |()| {
        let mut onclose = onclose;
        if let Some(close) = onclose.write().as_mut() {
            close();
        }
    });
    let active = open && options.dismiss;
    let handle = use_dismiss(
        anchor,
        popover.floating,
        active,
        popover.placed.peek().is_some(),
        Some(close),
        DismissOptions {
            escape: options.dismiss,
            outside: options.dismiss,
            ..Default::default()
        },
    );
    // Focus goes back to the trigger, whatever held it when the box opened.
    let focus_return = handle.focus_return();
    use_effect(use_reactive!(|(active,)| {
        if active {
            focus_return.remember_element(anchor);
        }
    }));

    popover.dismiss = options.dismiss.then_some(PopoverDismiss {
        handle,
        onclose,
        active,
    });
    popover
}

/// How often an open waits for its box's first layout before it places a 0x0.
const UNLAID_TRIES: u8 = 3;

/// [`use_popover`] on a caller-made floating handle, for a box another scope
/// reads (the root `Menu` owns every level's). Skips the `dismiss` wiring.
pub(crate) fn use_popover_on(
    anchor: ElementHandle,
    floating: ElementHandle,
    open: bool,
    options: PopoverOptions,
) -> PopoverHandle {
    let mut placed = use_signal(|| None::<Placed>);
    let mut anchor_width = use_signal(|| None::<f64>);
    let slot = use_portal_slot();
    owners::use_popup_owner(anchor, floating, open);

    // Bumped from the scroll callback, outside every scope: it only invalidates,
    // the effect below measures, where a `Read` may be created.
    let scroll_tick = use_signal(|| 0u64);
    // Alive only while open, so closed dropdowns listen to nothing.
    let subscription: Rc<RefCell<Option<Box<dyn ScrollSubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    // How often this open waited for the box's first layout.
    let waited: Rc<Cell<u8>> = use_hook(|| Rc::new(Cell::new(0)));
    // Whether this open measured the box again at its capped height.
    let capped: Rc<Cell<bool>> = use_hook(|| Rc::new(Cell::new(false)));

    use_drop({
        let subscription = subscription.clone();
        move || {
            subscription.borrow_mut().take();
        }
    });

    let listening = subscription.clone();
    use_effect(use_reactive!(|(open, options)| {
        // Reading it is what re-runs this on a scroll.
        let _ = scroll_tick();
        // Read before any return: it subscribes the effect to every (re)mount.
        let mounted = floating.mount_token().is_some();
        if !open || !mounted {
            listening.borrow_mut().take();
            waited.set(0);
            capped.set(false);
            // A `set` redraws even when unchanged: every opening ran this before
            // its box mounted and redrew the whole consumer for nothing.
            if placed.peek().is_some() {
                placed.set(None);
            }
            return;
        }

        let Some(document) = document() else {
            return;
        };

        // Own statement, so the borrow ends before the `borrow_mut` below.
        let unsubscribed = listening.borrow().is_none();
        if let Some(api) = unsubscribed.then(scroll).flatten() {
            *listening.borrow_mut() = Some(api.on_scroll(Box::new(move || {
                // A `Fn` callback lends no `&mut`; a `Copy` of the signal is the same value.
                let mut tick = scroll_tick;
                // `peek`, not a read: a callback must subscribe nothing.
                let next = tick.peek().wrapping_add(1);
                tick.set(next);
            })));
        }

        // Started here, awaited in the task: Blitz locks the document while
        // tasks drain (see `ElementApi::dimensions`).
        let anchor_size = anchor.dimensions();
        let anchor_offset = anchor.client_offset();
        let floating_size = floating.dimensions();
        let viewport = document.viewport();
        let rtl = anchor.is_rtl();
        let waited = waited.clone();
        let capped = capped.clone();

        spawn(async move {
            let (Ok(anchor_size), Ok((x, y)), Ok(floating_size), Ok(viewport)) = (
                anchor_size.await,
                anchor_offset.await,
                floating_size.await,
                viewport.await,
            ) else {
                return;
            };
            // Not laid out yet (native shell): a 0x0 box placed at the anchor's
            // edge overflows, so wait for layout, a few times at most (todo 896).
            let tries = waited.get();
            if floating_size.width == 0.0 && floating_size.height == 0.0 && tries < UNLAID_TRIES {
                waited.set(tries + 1);
                when_laid_out(move || {
                    let mut tick = scroll_tick;
                    let next = tick.peek().wrapping_add(1);
                    tick.set(next);
                });
                return;
            }

            let rect = Rect {
                x,
                y,
                width: anchor_size.width,
                height: anchor_size.height,
            };
            // The width style lands after this measure: place on the next pass, once per change.
            let widened = *anchor_width.peek() != Some(anchor_size.width);
            if widened {
                anchor_width.set(Some(anchor_size.width));
                if options.width != PopoverWidth::Auto {
                    let mut tick = scroll_tick;
                    let next = tick.peek().wrapping_add(1);
                    tick.set(next);
                    return;
                }
            }
            // A scroll that moved nothing (a listbox's own) must not redraw the consumer.
            let mut next = place(rect, floating_size, viewport, &options, rtl);
            // Taller than the room: a box capping itself at it shrinks, and above
            // its anchor its top follows the new height. Once per open.
            if floating_size.height > next.available_height + 0.5 && !capped.get() {
                capped.set(true);
                next.provisional = true;
                let mut tick = scroll_tick;
                let again = tick.peek().wrapping_add(1);
                tick.set(again);
            }
            let next = Some(next);
            if *placed.peek() != next {
                placed.set(next);
            }
        });
    }));

    PopoverHandle {
        anchor,
        floating,
        placed,
        anchor_width,
        width: options.width,
        padding: options.padding,
        slot,
        dismiss: None,
    }
}
