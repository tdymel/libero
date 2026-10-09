use dioxus::prelude::*;

use crate::{
    hooks::{DragPoint, use_direction, use_subscription_slot},
    platform::{
        self, EDGE_SWIPE_MARK, EdgeBand, ScrollSubscription, hold_edge_pan,
        holds_edge_pan_by_pointer,
    },
    sx::{Sx, sx},
};

/// Which way a swipe went on screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}

/// What [`use_swipe`] reports, once per press.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwipeEvent {
    pub direction: SwipeDirection,
    /// Where the pointer went down, in client coordinates.
    pub start: DragPoint,
    /// How far it had moved when the swipe was recognised.
    pub delta: DragPoint,
}

/// What [`use_swipe`] takes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwipeOptions {
    /// How far, in CSS px, the pointer travels along one axis before it is a swipe.
    pub distance: f64,
}

impl Default for SwipeOptions {
    fn default() -> Self {
        Self { distance: 48.0 }
    }
}

/// Handlers to spread onto the swiped element.
#[derive(Clone, Copy)]
pub struct Swipe {
    pub onpointerdown: Callback<PointerEvent>,
    pub onpointermove: Callback<PointerEvent>,
    pub onpointerup: Callback<PointerEvent>,
    pub onpointercancel: Callback<PointerEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Track {
    pointer_id: i32,
    start: DragPoint,
    /// The swipe was reported; the rest of the press is ignored.
    done: bool,
}

/// The track a pointerdown leaves. A primary touch or pen starts over, also over a
/// press that never ended here: its swipe opened a drawer that made this element
/// inert (2170). A second finger makes it a pinch; a mouse is ignored.
fn track_after_down(
    current: Option<Track>,
    primary: bool,
    mouse: bool,
    pointer_id: i32,
    start: DragPoint,
) -> Option<Track> {
    match (primary, mouse) {
        (false, _) => None,
        (true, true) => current,
        (true, false) => Some(Track {
            pointer_id,
            start,
            done: false,
        }),
    }
}

/// The direction `delta` swipes in, once one axis covers `distance` and
/// outweighs the other.
pub(crate) fn swipe_direction(delta: DragPoint, distance: f64) -> Option<SwipeDirection> {
    let (x, y) = (delta.x.abs(), delta.y.abs());
    if x >= distance && x > y {
        Some(if delta.x > 0.0 {
            SwipeDirection::Right
        } else {
            SwipeDirection::Left
        })
    } else if y >= distance && y > x {
        Some(if delta.y > 0.0 {
            SwipeDirection::Down
        } else {
            SwipeDirection::Up
        })
    } else {
        None
    }
}

/// Calls `on_swipe` once a touch or pen moves `options.distance` along one axis.
///
/// It fires during the move, not on release, and once per press. A mouse is
/// ignored ([`use_drag`](super::use_drag) covers it), and a second finger drops
/// the press. The element's `touch-action` decides what reaches it: the browser
/// keeps the axes it may pan and cancels the pointer, so give `pan-y` for
/// sideways swipes on a scrolling page, `none` for all four.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Box;
/// # use libero::hooks::{SwipeEvent, SwipeOptions, use_swipe};
/// # fn app() -> Element {
/// let mut last = use_signal(|| None);
/// let swipe = use_swipe(
///     Callback::new(move |event: SwipeEvent| last.set(Some(event.direction))),
///     SwipeOptions::default(),
/// );
///
/// rsx! {
///     Box {
///         style: "touch-action: none",
///         onpointerdown: move |event| swipe.onpointerdown.call(event),
///         onpointermove: move |event| swipe.onpointermove.call(event),
///         onpointerup: move |event| swipe.onpointerup.call(event),
///         onpointercancel: move |event| swipe.onpointercancel.call(event),
///         "Swiped: {last:?}"
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/hooks/use-swipe>
pub fn use_swipe(on_swipe: Callback<SwipeEvent>, options: SwipeOptions) -> Swipe {
    let SwipeOptions { distance } = options;
    let mut track = use_signal(|| None::<Track>);

    let onpointerdown = use_callback(move |event: PointerEvent| {
        let at = event.client_coordinates();
        let current = *track.peek();
        let next = track_after_down(
            current,
            event.is_primary(),
            event.pointer_type() == "mouse",
            event.pointer_id(),
            DragPoint { x: at.x, y: at.y },
        );
        if next != current {
            track.set(next);
        }
    });
    let onpointermove = use_callback(move |event: PointerEvent| {
        let Some(mut current) = *track.peek() else {
            return;
        };
        if current.done || event.pointer_id() != current.pointer_id {
            return;
        }
        let at = event.client_coordinates();
        let delta = DragPoint {
            x: at.x - current.start.x,
            y: at.y - current.start.y,
        };
        if let Some(direction) = swipe_direction(delta, distance) {
            current.done = true;
            track.set(Some(current));
            on_swipe.call(SwipeEvent {
                direction,
                start: current.start,
                delta,
            });
        }
    });
    let end = use_callback(move |event: PointerEvent| {
        if track
            .peek()
            .is_some_and(|current| current.pointer_id == event.pointer_id())
        {
            track.set(None);
        }
    });

    Swipe {
        onpointerdown,
        onpointermove,
        onpointerup: end,
        onpointercancel: end,
    }
}

/// The inline edge an edge swipe starts from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwipeEdge {
    /// Left, right under RTL: where a navigation drawer sits.
    #[default]
    Start,
    End,
}

/// What [`use_edge_swipe`] takes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeSwipeOptions {
    pub edge: SwipeEdge,
    /// Where the band a swipe must start in begins, CSS px in from the
    /// viewport's edge: past Android's system back zone, so both keep working.
    pub inset: f64,
    /// How wide the band is, in CSS px.
    pub width: f64,
    /// How far the pointer travels inward before it counts.
    pub distance: f64,
}

/// Past Android 16's back zone at its highest sensitivity, 30dp x 1.33 = 40dp
/// (API 36 emulator, 2108), plus 4dp. A CSS px is a dp in a WebView.
const BACK_ZONE: f64 = 44.0;

impl Default for EdgeSwipeOptions {
    fn default() -> Self {
        Self {
            edge: SwipeEdge::Start,
            inset: BACK_ZONE,
            width: 48.0,
            distance: SwipeOptions::default().distance,
        }
    }
}

/// Whether a swipe from client `x` towards `direction` opens from `edge`, for a
/// viewport `viewport` px wide (`None`: unknown yet, needed only for the right edge).
pub(crate) fn edge_swipe_opens(
    x: f64,
    direction: SwipeDirection,
    viewport: Option<f64>,
    options: &EdgeSwipeOptions,
    rtl: bool,
) -> Option<bool> {
    let inside = in_band(x, viewport, options, rtl)?;
    let inward = match (options.edge == SwipeEdge::Start) != rtl {
        true => SwipeDirection::Right,
        false => SwipeDirection::Left,
    };
    Some(direction == inward && inside)
}

/// Whether client `x` lies in the band, `None` for the right edge while `viewport` is unknown.
fn in_band(x: f64, viewport: Option<f64>, options: &EdgeSwipeOptions, rtl: bool) -> Option<bool> {
    let from_edge = match (options.edge == SwipeEdge::Start) != rtl {
        true => x,
        false => viewport? - x,
    };
    Some((options.inset..=options.inset + options.width).contains(&from_edge))
}

/// Blitz starts a touch pan once the finger is more than 2px from the press.
const PAN_SLOP: f64 = 2.0;

/// Whether a touch from the band holds the pan: `None` within [`PAN_SLOP`], then
/// whether its first move went inward more than across, as the web's hold decides.
fn holds_pan(delta: DragPoint, left: bool) -> Option<bool> {
    if delta.x.abs().max(delta.y.abs()) <= PAN_SLOP {
        return None;
    }
    let inward = if left { delta.x } else { -delta.x };
    Some(inward > delta.y.abs())
}

/// A band touch [`use_edge_swipe`] holds the pan for, where [`holds_edge_pan_by_pointer`].
#[derive(Clone, Copy, Debug, PartialEq)]
struct Held {
    pointer_id: i32,
    start: DragPoint,
    holding: Option<bool>,
}

/// `touch-action: pan-y pinch-zoom` for the element an edge swipe is spread on:
/// the page still scrolls and zooms, a sideways move reaches the hook. It stops
/// the browser's own sideways pan of that box, and of inner scrollers for a swipe
/// that starts in the band, so spread it on the main column, not on a horizontal scroller.
pub fn edge_swipe_sx() -> Sx {
    // The mark tells the hook's touch listener the touch started inside.
    sx().touch_action("pan-y pinch-zoom")
        .with(EDGE_SWIPE_MARK, "1")
}

/// Calls `on_swipe` when a touch or pen swipes inward from a band near the
/// viewport's start edge (end under RTL with [`SwipeEdge::Start`]): the way to
/// pull a navigation drawer open on a phone.
///
/// The band starts `options.inset` px in, not at the edge, so the system back
/// gesture there keeps working and needs no exclusion. Spread the handlers and
/// [`edge_swipe_sx`] on a box covering the page; no strip covers the content,
/// so taps go through. It adds a way in, not the only one: keep a button that
/// opens the same drawer (WCAG 2.5.1). RTL follows [`use_direction`]. A swipe
/// during a running fling, whose pointer the browser cancels, still counts.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Box;
/// # use libero::hooks::{EdgeSwipeOptions, edge_swipe_sx, use_edge_swipe};
/// # fn app() -> Element {
/// let mut open = use_signal(|| false);
/// let swipe = use_edge_swipe(
///     Callback::new(move |()| open.set(true)),
///     EdgeSwipeOptions::default(),
/// );
///
/// rsx! {
///     Box {
///         sx: edge_swipe_sx(),
///         onpointerdown: move |event| swipe.onpointerdown.call(event),
///         onpointermove: move |event| swipe.onpointermove.call(event),
///         onpointerup: move |event| swipe.onpointerup.call(event),
///         onpointercancel: move |event| swipe.onpointercancel.call(event),
///         "Page"
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/hooks/use-edge-swipe>
pub fn use_edge_swipe(on_swipe: Callback, options: EdgeSwipeOptions) -> Swipe {
    let direction = use_direction();
    let mut viewport = use_signal(|| None::<f64>);
    // An inner scroller (code block, tab list) would take the band's sideways pan (2170).
    let holding = use_subscription_slot::<dyn ScrollSubscription>();
    let rtl = direction.is_rtl();
    let decide = use_callback(move |event: SwipeEvent| {
        edge_swipe_opens(
            event.start.x,
            event.direction,
            *viewport.peek(),
            &options,
            direction.is_rtl(),
        )
    });
    // A swipe that beat the viewport measurement waits for it.
    let mut waiting = use_signal(|| None::<SwipeEvent>);
    // By the pointer, or after its cancel by the touch the browser took from it.
    let on_track = use_callback(move |event: SwipeEvent| match decide.call(event) {
        Some(true) => on_swipe.call(()),
        Some(false) => {}
        None => waiting.set(Some(event)),
    });
    use_effect(use_reactive!(|(options, rtl)| {
        holding.clear();
        holding.set(hold_edge_pan(
            EdgeBand {
                inset: options.inset,
                width: options.width,
                left: (options.edge == SwipeEdge::Start) != rtl,
                distance: options.distance,
            },
            Box::new(move |[x, y, dx, dy]| {
                let delta = DragPoint { x: dx, y: dy };
                if let Some(direction) = swipe_direction(delta, options.distance) {
                    on_track.call(SwipeEvent {
                        direction,
                        start: DragPoint { x, y },
                        delta,
                    });
                }
            }),
        ));
    }));
    let swipe = use_swipe(
        on_track,
        SwipeOptions {
            distance: options.distance,
        },
    );

    let mut held = use_signal(|| None::<Held>);
    let onpointermove = use_callback(move |event: PointerEvent| {
        let current = *held.peek();
        if let Some(mut current) = current
            && current.pointer_id == event.pointer_id()
        {
            if current.holding.is_none() {
                let at = event.client_coordinates();
                let delta = DragPoint {
                    x: at.x - current.start.x,
                    y: at.y - current.start.y,
                };
                current.holding = holds_pan(delta, (options.edge == SwipeEdge::Start) != rtl);
                if current.holding.is_some() {
                    held.set(Some(current));
                }
            }
            if current.holding == Some(true) {
                event.prevent_default();
            }
        }
        swipe.onpointermove.call(event);
    });
    let end = move |handler: Callback<PointerEvent>| {
        use_callback(move |event: PointerEvent| {
            if held.peek().is_some() {
                held.set(None);
            }
            handler.call(event);
        })
    };
    let onpointerup = end(swipe.onpointerup);
    let onpointercancel = end(swipe.onpointercancel);

    let onpointerdown = use_callback(move |event: PointerEvent| {
        if waiting.peek().is_some() {
            waiting.set(None);
        }
        if holds_edge_pan_by_pointer() {
            let at = event.client_coordinates();
            let next = (event.is_primary()
                && event.pointer_type() != "mouse"
                && in_band(at.x, *viewport.peek(), &options, rtl) == Some(true))
            .then_some(Held {
                pointer_id: event.pointer_id(),
                start: DragPoint { x: at.x, y: at.y },
                holding: None,
            });
            if next != *held.peek() {
                held.set(next);
            }
        }
        swipe.onpointerdown.call(event);
        let Some(document) = platform::document() else {
            return;
        };
        // Measured at each press: a rotation or a resize changes it.
        let size = document.viewport();
        spawn(async move {
            let Ok(size) = size.await else {
                return;
            };
            viewport.set(Some(size.width));
            let held = *waiting.peek();
            if let Some(event) = held {
                waiting.set(None);
                if decide.call(event) == Some(true) {
                    on_swipe.call(());
                }
            }
        });
    });

    Swipe {
        onpointerdown,
        onpointermove,
        onpointerup,
        onpointercancel,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f64, y: f64) -> DragPoint {
        DragPoint { x, y }
    }

    #[test]
    fn a_swipe_needs_the_distance_on_its_dominant_axis() {
        assert_eq!(swipe_direction(at(47.0, 0.0), 48.0), None);
        assert_eq!(
            swipe_direction(at(48.0, 10.0), 48.0),
            Some(SwipeDirection::Right)
        );
        assert_eq!(
            swipe_direction(at(-60.0, 59.0), 48.0),
            Some(SwipeDirection::Left)
        );
        assert_eq!(
            swipe_direction(at(20.0, -50.0), 48.0),
            Some(SwipeDirection::Up)
        );
        assert_eq!(
            swipe_direction(at(0.0, 90.0), 48.0),
            Some(SwipeDirection::Down)
        );
        // A diagonal is neither.
        assert_eq!(swipe_direction(at(60.0, 60.0), 48.0), None);
    }

    #[test]
    fn a_press_starts_over_a_track_that_never_ended() {
        let stale = Some(Track {
            pointer_id: 3,
            start: at(70.0, 400.0),
            done: true,
        });
        let fresh = Track {
            pointer_id: 4,
            start: at(60.0, 300.0),
            done: false,
        };
        assert_eq!(
            track_after_down(stale, true, false, 4, at(60.0, 300.0)),
            Some(fresh)
        );
        // A second finger drops the press; a mouse leaves it.
        assert_eq!(
            track_after_down(Some(fresh), false, false, 5, at(0.0, 0.0)),
            None
        );
        assert_eq!(
            track_after_down(Some(fresh), true, true, 1, at(0.0, 0.0)),
            Some(fresh)
        );
        assert_eq!(track_after_down(None, true, true, 1, at(0.0, 0.0)), None);
    }

    #[test]
    fn a_band_touch_holds_the_pan_only_on_an_inward_first_move() {
        assert_eq!(holds_pan(at(2.0, -1.0), true), None);
        assert_eq!(holds_pan(at(15.0, 1.0), true), Some(true));
        // Up the page, outward, or mirrored at the right edge.
        assert_eq!(holds_pan(at(0.0, -15.0), true), Some(false));
        assert_eq!(holds_pan(at(-15.0, 0.0), true), Some(false));
        assert_eq!(holds_pan(at(-15.0, 0.0), false), Some(true));
    }

    fn opens(x: f64, direction: SwipeDirection, edge: SwipeEdge, rtl: bool) -> Option<bool> {
        let options = EdgeSwipeOptions {
            edge,
            inset: 50.0,
            width: 48.0,
            ..EdgeSwipeOptions::default()
        };
        edge_swipe_opens(x, direction, Some(400.0), &options, rtl)
    }

    #[test]
    fn the_band_sits_past_the_back_zone_at_the_start_edge() {
        use SwipeDirection::{Left, Right};
        assert_eq!(opens(60.0, Right, SwipeEdge::Start, false), Some(true));
        assert_eq!(opens(98.0, Right, SwipeEdge::Start, false), Some(true));
        // Inside the system's back zone, past the band, the wrong way.
        assert_eq!(opens(40.0, Right, SwipeEdge::Start, false), Some(false));
        assert_eq!(opens(120.0, Right, SwipeEdge::Start, false), Some(false));
        assert_eq!(opens(60.0, Left, SwipeEdge::Start, false), Some(false));
    }

    #[test]
    fn rtl_and_the_end_edge_mirror_the_band() {
        use SwipeDirection::{Left, Right};
        assert_eq!(opens(340.0, Left, SwipeEdge::Start, true), Some(true));
        assert_eq!(opens(60.0, Right, SwipeEdge::Start, true), Some(false));
        assert_eq!(opens(340.0, Left, SwipeEdge::End, false), Some(true));
        assert_eq!(opens(60.0, Right, SwipeEdge::End, true), Some(true));
        assert_eq!(opens(380.0, Left, SwipeEdge::End, false), Some(false));
    }

    #[test]
    fn the_right_edge_waits_for_the_viewport() {
        let options = EdgeSwipeOptions::default();
        assert_eq!(
            edge_swipe_opens(340.0, SwipeDirection::Left, None, &options, true),
            None
        );
        assert_eq!(
            edge_swipe_opens(70.0, SwipeDirection::Right, None, &options, false),
            Some(true)
        );
    }
}
