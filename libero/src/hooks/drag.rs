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
    use_drag_with(options, false)
}

/// How far, in CSS px, a touch on a [`use_sideways_drag`] control moves
/// sideways before it drags. Android's touch slop is 8dp.
const SIDEWAYS_SLOP: f64 = 8.0;

/// [`use_drag`] for a horizontal control on a page that scrolls, as a native
/// Android slider: a touch drags once it moves sideways, a tap jumps on release,
/// and a vertical swipe scrolls the page. Give it [`sideways_drag_sx`].
///
/// A tap calls `onend` right after `onstart`: an `onstart` that measures first
/// must hold the end back until it has run.
pub(crate) fn use_sideways_drag(options: DragOptions) -> Drag {
    use_drag_with(options, true)
}

/// `touch-action: pan-y` for a [`use_sideways_drag`] control: the browser
/// keeps a vertical swipe and cancels the pointer.
pub(crate) fn sideways_drag_sx() -> Sx {
    sx().touch_action("pan-y")
}

fn use_drag_with(options: DragOptions, sideways: bool) -> Drag {
    let DragOptions {
        capture,
        onstart,
        onmove,
        onend,
    } = options;

    let mut active = use_signal(|| Option::<ActiveDrag>::None);
    let mut dragging = use_signal(|| false);
    // A sideways touch not yet moved far enough to be a drag.
    let mut pending = use_signal(|| Option::<ActiveDrag>::None);

    let cancel = use_callback(move |()| {
        active.set(None);
        dragging.set(false);
    });

    let track = use_callback(move |event: Event<PointerData>| {
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

    // Starts the drag `event`'s pointer went down for at `client`; `false` if
    // `onstart` cancelled it.
    let begin = use_callback(move |(event, client): (Event<PointerData>, DragPoint)| {
        // Active before `onstart`, so a synchronous `cancel` clears it rather
        // than being overwritten.
        active.set(Some(ActiveDrag {
            pointer_id: event.pointer_id(),
            start: client,
        }));
        dragging.set(true);

        onstart.call(DragStart { client, cancel });
        if active.peek().is_none() {
            return false;
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
            platform::follow_pointer(&event, &within, track, end);
        }
        true
    });

    let onpointerdown = use_callback(move |event: Event<PointerData>| {
        // A right- or middle-click must not drag; an unreported button (some
        // webviews, and touch) still counts as primary.
        if matches!(event.trigger_button(), Some(button) if button != MouseButton::Primary)
            || active.read().is_some()
            || pending.read().is_some()
        {
            return;
        }
        event.prevent_default();
        let coordinates = event.client_coordinates();
        let client = DragPoint {
            x: coordinates.x,
            y: coordinates.y,
        };
        let press = ActiveDrag {
            pointer_id: event.pointer_id(),
            start: client,
        };
        match sideways && event.pointer_type() == "touch" {
            true => pending.set(Some(press)),
            false => {
                begin.call((event, client));
            }
        }
    });

    let onpointermove = use_callback(move |event: Event<PointerData>| {
        let Some(press) = *pending.peek() else {
            track.call(event);
            return;
        };
        if event.pointer_id() != press.pointer_id {
            return;
        }
        let coordinates = event.client_coordinates();
        let (dx, dy) = (coordinates.x - press.start.x, coordinates.y - press.start.y);
        // Not yet sideways: wait, the browser cancels a swipe it scrolls.
        if dx.abs() < SIDEWAYS_SLOP || dx.abs() <= dy.abs() {
            return;
        }
        pending.set(None);
        if begin.call((event.clone(), press.start)) {
            track.call(event);
        }
    });

    // A touch released before it moved is a tap: start and end at once.
    let onpointerup = use_callback(move |event: Event<PointerData>| {
        let Some(press) = *pending.peek() else {
            end.call(event);
            return;
        };
        if event.pointer_id() != press.pointer_id {
            return;
        }
        pending.set(None);
        if begin.call((event.clone(), press.start)) {
            end.call(event);
        }
    });

    // The OS can revoke a pointer mid-drag; without this the drag sticks. A
    // pending touch the page scrolled is dropped unreported.
    let onpointercancel = use_callback(move |event: Event<PointerData>| {
        if pending
            .peek()
            .is_some_and(|press| press.pointer_id == event.pointer_id())
        {
            pending.set(None);
            return;
        }
        end.call(event);
    });

    Drag {
        dragging,
        onpointerdown,
        onpointermove,
        onpointerup,
        onpointercancel,
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
