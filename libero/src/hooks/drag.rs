use dioxus::{html::input_data::MouseButton, prelude::*};

use crate::{
    components::dom_api,
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
    /// Id of the element that takes pointer capture and carries the
    /// move/up/cancel handlers - the one owning the geometry, not the handle.
    pub capture: Signal<String>,
    /// Return `false` to abort before the drag starts.
    pub on_start: Callback<DragStart, bool>,
    pub on_move: Callback<DragMove>,
    pub on_end: Callback<()>,
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
        on_start,
        on_move,
        on_end,
    } = options;

    let mut active = use_signal(|| Option::<ActiveDrag>::None);
    let mut dragging = use_signal(|| false);

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

        if !on_start.call(DragStart { client }) {
            return;
        }

        // Best-effort: failing costs out-of-element tracking, not the drag.
        if let Ok(element) = dom_api().query_selector(&format!("#{}", capture())) {
            let _ = element.set_pointer_capture(event.pointer_id());
        }

        active.set(Some(ActiveDrag {
            pointer_id: event.pointer_id(),
            start: client,
        }));
        dragging.set(true);
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

        on_move.call(DragMove {
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
        on_end.call(());
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
pub fn drag_handle_sx() -> Sx {
    sx().touch_action("none")
}
