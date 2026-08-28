//! Anchored, portaled floating boxes: a dropdown that escapes an
//! `overflow: hidden` ancestor and flips when it runs out of room.

mod options;
mod place;

use dioxus::prelude::*;

pub use options::{Align, Placement, PopoverOptions, PopoverWidth, Side};
pub use place::{Placed, Rect};

use crate::{
    hooks::{
        ElementHandle,
        element::use_element,
        portal::{PortalSlot, use_portal_slot},
    },
    platform::{ElementApi, document},
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

        Some(format!(
            "position:fixed;left:{x}px;top:{y}px;width:{width};min-width:{min_width};visibility:{visibility};"
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
/// Measured once per open, and again whenever `options` changes -
/// [`PopoverOptions::remeasure`] is the knob for an anchor that resizes. A page
/// scrolled while the popover is open drags it off its anchor: tracking that
/// needs a window-level event, which no backend here exposes yet.
pub fn use_popover(anchor: ElementHandle, open: bool, options: PopoverOptions) -> PopoverHandle {
    let floating = use_element();
    let mut placed = use_signal(|| None::<Placed>);
    let mut anchor_width = use_signal(|| None::<f64>);
    let slot = use_portal_slot();

    use_effect(use_reactive!(|(open, options)| {
        // Read first, branch second: this is what subscribes the effect to the
        // box mounting, and an early return would skip it. It also re-runs on a
        // *re*-mount, which is every reopen.
        let mounted = floating.mount_token().is_some();
        if !open || !mounted {
            placed.set(None);
            return;
        }

        let Some(document) = document() else {
            return;
        };
        // Started here, awaited in the task: a read resolves where it is
        // called, and under Blitz the document is locked for as long as dioxus
        // drains tasks (see `ElementApi::dimensions`).
        let anchor_size = anchor.dimensions();
        let anchor_offset = anchor.client_offset();
        let floating_size = floating.dimensions();
        let viewport = document.viewport();

        spawn(async move {
            let (Ok(anchor_size), Ok((x, y)), Ok(floating_size), Ok(viewport)) = (
                anchor_size.await,
                anchor_offset.await,
                floating_size.await,
                viewport.await,
            ) else {
                return;
            };

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
        slot,
    }
}
