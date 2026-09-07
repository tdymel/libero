use dioxus::{html::input_data::MouseButton, prelude::*};

use crate::{
    hooks::ElementHandle,
    platform::ElementApi,
    sx::{Sx, sx},
};

/// A pointer position in client coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragPoint {
    pub x: f64,
    pub y: f64,
}

/// Where the pointer went down. Measure whatever geometry the drag is
/// relative to here - it is the only moment the layout is known to be
/// settled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragStart {
    pub client: DragPoint,
    /// Abandons the drag, either at once - a disabled control refusing it -
    /// or once the handler has measured and learned it is impossible, which
    /// off the web is a round-trip away. Called at once, no move or end is
    /// ever reported for this press.
    pub cancel: Callback<()>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragMove {
    pub start: DragPoint,
    pub client: DragPoint,
}

impl DragMove {
    pub fn delta(&self) -> DragPoint {
        DragPoint {
            x: self.client.x - self.start.x,
            y: self.client.y - self.start.y,
        }
    }
}

pub struct DragOptions {
    /// The element that takes pointer capture and carries the move/up/cancel
    /// handlers - the one owning the geometry, not the grab handle.
    pub capture: ElementHandle,
    /// The geometry a drag needs may take a round-trip to measure, so this
    /// cannot veto by returning - call [`DragStart::cancel`] once it knows.
    pub onstart: Callback<DragStart>,
    pub onmove: Callback<DragMove>,
    pub onend: Callback<()>,
}

/// The one pointer a drag follows, and where it went down.
#[derive(Clone, Copy)]
struct ActiveDrag {
    pointer_id: i32,
    start: DragPoint,
}

/// Handlers to spread onto the two elements a drag involves.
#[derive(Clone, Copy)]
pub struct Drag {
    pub dragging: Signal<bool>,
    /// The grab handle.
    pub onpointerdown: Callback<Event<PointerData>>,
    /// The capture element.
    pub onpointermove: Callback<Event<PointerData>>,
    pub onpointerup: Callback<Event<PointerData>>,
    pub onpointercancel: Callback<Event<PointerData>>,
}

/// Pointer plumbing for a drag: capture, a start coordinate, deltas against
/// it, and one end path off both release and cancel. It knows nothing about
/// axes or units - convert the client-space delta yourself.
///
/// Give the handle [`drag_handle_sx`], or a touch never produces a move.
pub fn use_drag(options: DragOptions) -> Drag {
    let DragOptions {
        capture,
        onstart,
        onmove,
        onend,
    } = options;

    let mut active = use_signal(|| Option::<ActiveDrag>::None);
    let mut dragging = use_signal(|| false);

    let cancel = use_callback(move |()| {
        active.set(None);
        dragging.set(false);
    });

    let onpointerdown = use_callback(move |event: Event<PointerData>| {
        // A right- or middle-click must not drag; an unreported button (some
        // webviews, and touch) still counts as primary.
        if matches!(event.trigger_button(), Some(button) if button != MouseButton::Primary)
            || active.read().is_some()
        {
            return;
        }
        event.prevent_default();
        let coordinates = event.client_coordinates();
        let client = DragPoint {
            x: coordinates.x,
            y: coordinates.y,
        };

        // Active before `onstart`, so a `cancel` it makes synchronously - a
        // disabled control refusing the drag - clears it rather than being
        // overwritten.
        active.set(Some(ActiveDrag {
            pointer_id: event.pointer_id(),
            start: client,
        }));
        dragging.set(true);

        onstart.call(DragStart { client, cancel });
        if active.peek().is_none() {
            return;
        }

        // Best-effort: where the platform has no capture this costs
        // out-of-element tracking, not the drag.
        let _ = capture.set_pointer_capture(event.pointer_id());
    });

    let onpointermove = use_callback(move |event: Event<PointerData>| {
        let Some(drag) = active() else {
            return;
        };
        // A second finger on the same element is not this drag.
        if event.pointer_id() != drag.pointer_id {
            return;
        }
        let coordinates = event.client_coordinates();

        onmove.call(DragMove {
            start: drag.start,
            client: DragPoint {
                x: coordinates.x,
                y: coordinates.y,
            },
        });
    });

    let end = use_callback(move |event: Event<PointerData>| {
        let Some(drag) = active() else {
            return;
        };
        if event.pointer_id() != drag.pointer_id {
            return;
        }
        active.set(None);
        dragging.set(false);
        onend.call(());
    });

    Drag {
        dragging,
        onpointerdown,
        onpointermove,
        onpointerup: end,
        // The OS can revoke a pointer mid-drag; without this the drag sticks.
        onpointercancel: end,
    }
}

/// Without `touch-action: none` the browser claims a touch as a scroll and no
/// `pointermove` ever arrives.
///
/// It carries no colour, so it changes nothing about the handle's focus ring.
/// Worth knowing anyway, because a drag handle is usually a filled knob: a
/// handle with `background("primary.6")` takes a **white** focus ring drawn
/// outside itself, which is 1.11:1 on a light track. Inset the ring into the
/// fill - `focus_visible(sx().outline_offset("-4px"))` - where the colour the
/// fill publishes is the right one. See [`crate::components::Box`].
pub fn drag_handle_sx() -> Sx {
    sx().touch_action("none")
}
