use dioxus::{html::input_data::MouseButton, prelude::*};

use crate::{
    hooks::ElementHandle,
    platform::{self, ElementApi},
    sx::{Sx, sx},
};

/// A pointer position in client coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragPoint {
    pub x: f64,
    pub y: f64,
}

/// Where the pointer went down. Measure the drag's geometry here: the only
/// moment the layout is known to be settled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragStart {
    pub client: DragPoint,
    /// Abandons the drag, at once or after a measuring round-trip. Called at
    /// once, no move or end is reported for this press.
    pub cancel: Callback<()>,
}

/// A pointer move during a drag.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragMove {
    pub start: DragPoint,
    pub client: DragPoint,
}

impl DragMove {
    /// How far the pointer moved since the drag started.
    pub fn delta(&self) -> DragPoint {
        DragPoint {
            x: self.client.x - self.start.x,
            y: self.client.y - self.start.y,
        }
    }
}

/// What [`use_drag`] takes.
pub struct DragOptions {
    /// Takes pointer capture and the move/up/cancel handlers: the element
    /// owning the geometry, not the grab handle.
    pub capture: ElementHandle,
    /// Vetoes by calling [`DragStart::cancel`], not by returning: measuring
    /// may take a round-trip.
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

/// Pointer plumbing for a drag: capture, a start point, client-space deltas
/// against it, and one end path off release and cancel.
///
/// Give the handle [`drag_handle_sx`], or a touch never produces a move. On
/// the web the hook focuses the pressed tab stop; elsewhere focus in `onstart`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Box;
/// # use libero::hooks::{DragMove, DragOptions, drag_handle_sx, use_drag, use_element};
/// # fn app() -> Element {
/// let track = use_element();
/// let mut x = use_signal(|| 0.0);
/// let drag = use_drag(DragOptions {
///     capture: track,
///     onstart: Callback::new(|_| {}),
///     onmove: Callback::new(move |step: DragMove| x.set(step.delta().x)),
///     onend: Callback::new(|()| {}),
/// });
///
/// rsx! {
///     Box {
///         onmounted: track.mount(),
///         onpointermove: move |event| drag.onpointermove.call(event),
///         onpointerup: move |event| drag.onpointerup.call(event),
///         onpointercancel: move |event| drag.onpointercancel.call(event),
///         Box {
///             onpointerdown: move |event| drag.onpointerdown.call(event),
///             sx: drag_handle_sx(),
///             style: "translate: {x}px",
///             "Drag me"
///         }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/hooks/use-drag>
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

        // Active before `onstart`, so a synchronous `cancel` clears it rather
        // than being overwritten.
        active.set(Some(ActiveDrag {
            pointer_id: event.pointer_id(),
            start: client,
        }));
        dragging.set(true);

        onstart.call(DragStart { client, cancel });
        if active.peek().is_none() {
            return;
        }

        // `prevent_default` also cancelled the focus the press would bring.
        if let Some(within) = capture.mounted() {
            platform::focus_pressed(&event, &within);
        }

        // Blitz has no capture and follows the pointer instead; where neither
        // works, this costs out-of-element tracking, not the drag.
        if capture.set_pointer_capture(event.pointer_id()).is_err()
            && let Some(within) = capture.mounted()
        {
            platform::follow_pointer(&event, &within, onpointermove, end);
        }
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

/// `touch-action: none` for a drag handle: without it a touch scrolls and no
/// `pointermove` arrives.
///
/// A filled knob's focus ring turns white on white outside it: inset it with
/// `focus_visible(sx().outline_offset("-4px"))`.
pub fn drag_handle_sx() -> Sx {
    sx().touch_action("none")
}
