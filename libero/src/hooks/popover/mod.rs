//! Anchored, portaled floating boxes: a dropdown that escapes an
//! `overflow: hidden` ancestor and flips when it runs out of room.

mod options;
mod place;

use dioxus::prelude::*;

pub use options::{Align, Placement, PopoverOptions, PopoverWidth, Side};
pub use place::{Placed, Rect};

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::{
    hooks::{
        ElementHandle,
        element::use_element,
        portal::{PortalSlot, use_portal_slot},
    },
    platform::{ElementApi, ScrollSubscription, document, scroll, when_laid_out},
};

use place::place;

/// A popover's own state: where its box goes, and the portal slot it goes in.
///
/// Built by [`use_popover`], and `Copy`, so it travels into the handlers and
/// closures that render the box.
#[derive(Clone, Copy)]
pub struct PopoverHandle {
    floating: ElementHandle,
    placed: Signal<Option<Placed>>,
    /// The anchor's measured width, which is what `PopoverWidth` follows.
    anchor_width: Signal<Option<f64>>,
    width: PopoverWidth,
    /// The collision padding, which the width cap leaves at both edges.
    padding: f64,
    slot: PortalSlot,
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
        self.placed.read().is_some()
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

    /// The box's `style`, for `use_box().style(..)`.
    ///
    /// `position: fixed` in viewport coordinates: the outlet sits at the
    /// document root, but any ancestor `transform` still creates a containing
    /// block, and `client_offset` already answers in the viewport's frame.
    ///
    /// Before the first measurement the box is laid out but
    /// `visibility: hidden` - **not** `display: none`, which has no dimensions
    /// to measure.
    ///
    /// **Every declaration is emitted on every render, never omitted.** A
    /// renderer applies these one property at a time, so a declaration that
    /// simply stops being printed is never taken off the element: dropping
    /// `visibility` once the box was placed left it hidden at the right
    /// coordinates.
    ///
    /// The width is capped at the viewport less the collision padding at both
    /// edges, so a long row ellipsises instead of pushing the box off a phone's
    /// screen (todo 357). In CSS rather than from the measurement: the cap then
    /// holds on the pass that is measured, and for content that changes while
    /// the box is open.
    pub fn style(&self) -> Option<String> {
        let (x, y, visibility) = match *self.placed.read() {
            Some(placed) => (placed.x, placed.y, "visible"),
            None => (0.0, 0.0, "hidden"),
        };
        // `auto` until the anchor has been measured: a `width: 0` box laid out
        // for its own measurement would report the height of its text wrapped
        // into nothing.
        let (width, min_width) = match (self.width, *self.anchor_width.read()) {
            (PopoverWidth::Match, Some(anchor)) => (format!("{anchor}px"), String::from("auto")),
            (PopoverWidth::Min, Some(anchor)) => (String::from("auto"), format!("{anchor}px")),
            _ => (String::from("auto"), String::from("auto")),
        };

        let edges = 2.0 * self.padding;

        Some(format!(
            "position:fixed;left:{x}px;top:{y}px;width:{width};min-width:{min_width};max-width:calc(100vw - {edges}px);visibility:{visibility};"
        ))
    }

    /// Portals the box. `None` takes it away, which is how a closed popover
    /// stops rendering.
    ///
    /// Already-rendered, like [`use_portal`](super::use_portal): the reads
    /// inside it have to happen in the caller's scope, not the outlet's.
    pub fn show(&self, content: Option<Element>) {
        self.slot.show(content);
    }
}

/// Anchors a portaled box to `anchor`, flipping and shifting it to stay in the
/// viewport.
///
/// Measured once per open, again whenever `options` changes -
/// [`PopoverOptions::remeasure`] is the knob for an anchor that resizes - and
/// again on every scroll, so the box follows its anchor instead of being
/// dragged off it. Scroll tracking needs [`platform::scroll`](crate::platform::scroll),
/// which only the web answers; elsewhere an open popover still drifts.
///
/// It keeps following even when the anchor leaves the viewport: `place()` flips
/// and shifts as usual, so the box ends up clamped at the edge rather than
/// hidden or closed. Closing is the caller's business - this hook owns no open
/// state.
pub fn use_popover(anchor: ElementHandle, open: bool, options: PopoverOptions) -> PopoverHandle {
    use_popover_on(anchor, use_element(), open, options)
}

/// [`use_popover`] on a floating handle the caller made, for a box some scope
/// other than this one has to read. A handle is owned by the scope that made
/// it, and dioxus warns when a scope that is not its descendant reads it: a
/// `Menu` level's box is read by every level above it, so the root `Menu`
/// owns them all.
pub(crate) fn use_popover_on(
    anchor: ElementHandle,
    floating: ElementHandle,
    open: bool,
    options: PopoverOptions,
) -> PopoverHandle {
    let mut placed = use_signal(|| None::<Placed>);
    let mut anchor_width = use_signal(|| None::<f64>);
    let slot = use_portal_slot();

    // Bumped from the scroll callback, which runs outside every dioxus scope -
    // so the signal is owned by the root and dropped by hand, the same
    // obligation anything portaled has ([[codebase/use-popover]]). The callback
    // deliberately does *not* measure: it only invalidates, and the effect
    // below does the work, in the runtime, where a `Read` may be created.
    let scroll_tick = use_hook(|| Signal::new_in_scope(0u64, ScopeId::ROOT));
    // Alive exactly while the popover is open. Dropping it removes the
    // listener, so a page full of closed dropdowns listens to nothing.
    let subscription: Rc<RefCell<Option<Box<dyn ScrollSubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    // Whether this open already waited once for the box's first layout.
    let waited: Rc<Cell<bool>> = use_hook(|| Rc::new(Cell::new(false)));

    use_drop({
        let subscription = subscription.clone();
        move || {
            subscription.borrow_mut().take();
            scroll_tick.manually_drop();
        }
    });

    let listening = subscription.clone();
    use_effect(use_reactive!(|(open, options)| {
        // Reading it is what re-runs this on a scroll.
        let _ = scroll_tick();
        // Read first, branch second: this is what subscribes the effect to the
        // box mounting, and an early return would skip it. It also re-runs on a
        // *re*-mount, which is every reopen.
        let mounted = floating.mount_token().is_some();
        if !open || !mounted {
            listening.borrow_mut().take();
            waited.set(false);
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

        // Read the borrow out in its own statement, so it is released before
        // the `borrow_mut` below.
        let unsubscribed = listening.borrow().is_none();
        if let Some(api) = unsubscribed.then(scroll).flatten() {
            *listening.borrow_mut() = Some(api.on_scroll(Box::new(move || {
                // A `Fn` callback cannot hand out `&mut` to what it captured,
                // and `set` needs one. `Signal` is `Copy`, so a copy per call
                // addresses the very same value.
                let mut tick = scroll_tick;
                // `peek`, not a read: a callback must subscribe nothing.
                let next = tick.peek().wrapping_add(1);
                tick.set(next);
            })));
        }

        // Started here, awaited in the task: a read resolves where it is
        // called, and under Blitz the document is locked for as long as dioxus
        // drains tasks (see `ElementApi::dimensions`).
        let anchor_size = anchor.dimensions();
        let anchor_offset = anchor.client_offset();
        let floating_size = floating.dimensions();
        let viewport = document.viewport();
        let waited = waited.clone();

        spawn(async move {
            let (Ok(anchor_size), Ok((x, y)), Ok(floating_size), Ok(viewport)) = (
                anchor_size.await,
                anchor_offset.await,
                floating_size.await,
                viewport.await,
            ) else {
                return;
            };
            // Not laid out yet: a native shell can run this before its next
            // layout, and a 0x0 box placed at the anchor's edge overflows it.
            if floating_size.width == 0.0 && floating_size.height == 0.0 && !waited.replace(true) {
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
            anchor_width.set(Some(anchor_size.width));
            placed.set(Some(place(rect, floating_size, viewport, &options)));
        });
    }));

    PopoverHandle {
        floating,
        placed,
        anchor_width,
        width: options.width,
        padding: options.padding,
        slot,
    }
}
