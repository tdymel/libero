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

/// How far a mouse or pen moves before a [`use_distance_drag`] drags.
const MOUSE_SLOP: f64 = 4.0;

/// How far from a scrolling box's side edge a drag starts scrolling it, in px.
const EDGE: f64 = 48.0;

/// The most a box scrolls per auto-scroll tick, at its very edge.
const EDGE_STEP: f64 = 16.0;

/// The px a drag at client `x` scrolls a box spanning `start..start + width`
/// by per tick: negative near its left edge, positive near its right, faster
/// the closer, 0 elsewhere.
pub(crate) fn edge_scroll_step(x: f64, start: f64, width: f64) -> f64 {
    let depth = |distance: f64| ((EDGE - distance) / EDGE).clamp(0.0, 1.0);
    let (left, right) = (depth(x - start), depth(start + width - x));
    // At least a px, so a pointer just inside the zone still moves.
    let step = |depth: f64| (EDGE_STEP * depth).max(1.0);
    match (left > 0.0, right > 0.0) {
        (true, _) => -step(left),
        (_, true) => step(right),
        _ => 0.0,
    }
}

/// When a press turns into a drag.
#[derive(Clone, Copy, PartialEq)]
enum Activation {
    /// At once.
    Press,
    /// A touch once it moves sideways; see [`use_sideways_drag`].
    Sideways,
    /// Once it moves a few px; see [`use_distance_drag`].
    Distance,
}

/// [`use_drag`] that drags only once the pointer moved a few px, so a click on
/// the handle stays a click: no `onstart` or `onend` for it.
pub(crate) fn use_distance_drag(options: DragOptions) -> Drag {
    use_drag_inner(options, Activation::Distance, None)
}

/// [`use_drag`] for a horizontal control on a page that scrolls, as a native
/// Android slider: a touch drags once it moves sideways, a tap jumps on release,
/// and a vertical swipe scrolls the page. Give it [`sideways_drag_sx`].
///
/// `grabs` answers, at a touch's press, whether it may drag at all; one that
/// may not only taps, and moving drops it (1059). A mouse always drags.
///
/// A tap calls `onend` right after `onstart`: an `onstart` that measures first
/// must hold the end back until it has run.
pub(crate) fn use_sideways_drag(options: DragOptions, grabs: Callback<(), bool>) -> Drag {
    use_drag_inner(options, Activation::Sideways, Some(grabs))
}

/// `touch-action: pan-y` for a [`use_sideways_drag`] control: the browser
/// keeps a vertical swipe and cancels the pointer.
pub(crate) fn sideways_drag_sx() -> Sx {
    sx().touch_action("pan-y")
}

/// [`use_sideways_drag`] while `sideways`, else [`use_drag`]: for a control
/// whose axis can change between renders.
pub(crate) fn use_drag_with(options: DragOptions, sideways: bool) -> Drag {
    let activation = if sideways {
        Activation::Sideways
    } else {
        Activation::Press
    };
    use_drag_inner(options, activation, None)
}

/// A press not yet moved far enough to be a drag.
#[derive(Clone, Copy)]
struct Pending {
    press: ActiveDrag,
    /// `false`: a tap or nothing.
    grabs: bool,
    /// Set for a [`Activation::Distance`] press: how far it must move.
    slop: Option<f64>,
}

fn use_drag_inner(
    options: DragOptions,
    activation: Activation,
    grabs: Option<Callback<(), bool>>,
) -> Drag {
    let DragOptions {
        capture,
        onstart,
        onmove,
        onend,
    } = options;

    let mut active = use_signal(|| Option::<ActiveDrag>::None);
    let mut dragging = use_signal(|| false);
    let mut pending = use_signal(|| Option::<Pending>::None);

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
            // Uncaptured, a distance press released outside never ends: a new one replaces it.
            || pending.read().is_some_and(|pending| pending.slop.is_none())
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
        let touch = event.pointer_type() == "touch";
        match activation {
            Activation::Sideways if touch => pending.set(Some(Pending {
                press,
                grabs: grabs.is_none_or(|grabs| grabs.call(())),
                slop: None,
            })),
            Activation::Distance => {
                // The press keeps its focus even if it never drags.
                if let Some(within) = capture.mounted() {
                    platform::focus_pressed(&event, &within);
                }
                pending.set(Some(Pending {
                    press,
                    grabs: true,
                    slop: Some(if touch { SIDEWAYS_SLOP } else { MOUSE_SLOP }),
                }));
            }
            _ => {
                begin.call((event, client));
            }
        }
    });

    let onpointermove = use_callback(move |event: Event<PointerData>| {
        let Some(Pending { press, grabs, slop }) = *pending.peek() else {
            track.call(event);
            return;
        };
        if event.pointer_id() != press.pointer_id {
            return;
        }
        let coordinates = event.client_coordinates();
        let (dx, dy) = (coordinates.x - press.start.x, coordinates.y - press.start.y);
        if let Some(slop) = slop {
            if dx.hypot(dy) >= slop {
                pending.set(None);
                if begin.call((event.clone(), press.start)) {
                    track.call(event);
                }
            }
            return;
        }
        // No longer a tap, and not allowed to drag.
        if !grabs {
            if dx.hypot(dy) >= SIDEWAYS_SLOP {
                pending.set(None);
            }
            return;
        }
        // Not yet sideways: wait, the browser cancels a swipe it scrolls.
        if dx.abs() < SIDEWAYS_SLOP || dx.abs() <= dy.abs() {
            return;
        }
        pending.set(None);
        if begin.call((event.clone(), press.start)) {
            track.call(event);
        }
    });

    // A touch released before it moved is a tap: start and end at once. A
    // distance press is a click, reported to nobody.
    let onpointerup = use_callback(move |event: Event<PointerData>| {
        let Some(Pending { press, slop, .. }) = *pending.peek() else {
            end.call(event);
            return;
        };
        if event.pointer_id() != press.pointer_id {
            return;
        }
        pending.set(None);
        if slop.is_none() && begin.call((event.clone(), press.start)) {
            end.call(event);
        }
    });

    // The OS can revoke a pointer mid-drag; without this the drag sticks. A
    // pending touch the page scrolled is dropped unreported.
    let onpointercancel = use_callback(move |event: Event<PointerData>| {
        if pending
            .peek()
            .is_some_and(|pending| pending.press.pointer_id == event.pointer_id())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drag_near_a_side_edge_scrolls_that_way_faster_the_closer() {
        let step = |x: f64| edge_scroll_step(x, 100.0, 600.0);
        assert_eq!(step(400.0), 0.0);
        assert_eq!(step(148.0), 0.0);
        assert_eq!(step(100.0), -EDGE_STEP);
        assert_eq!(step(124.0), -EDGE_STEP / 2.0);
        assert_eq!(step(700.0), EDGE_STEP);
        // Past the edge, the pointer off the box, still the full step.
        assert_eq!(step(760.0), EDGE_STEP);
        assert_eq!(step(147.9), -1.0);
    }
}
