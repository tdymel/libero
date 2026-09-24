//! The zoom, pan, pinch and swipe maths, and the state they share.

use dioxus::prelude::*;

use crate::{
    components::accessibility::Announcer,
    hooks::{DragPoint, ElementHandle},
    localization::{LightboxLabels, fill},
    platform::{Dimensions, ElementApi},
};

/// Pixels one arrow press pans a zoomed picture.
pub(super) const PAN_STEP: f64 = 48.0;
/// Scale change per wheel notch.
pub(super) const WHEEL_FACTOR: f64 = 1.25;
/// Double-click and `z` step from this, doubling up to `max_zoom`.
pub(super) const FIRST_STEP_ZOOM: f64 = 2.0;
/// How far down a swipe has to travel to close.
pub(super) const SWIPE_CLOSE_DISTANCE: f64 = 96.0;
/// How far a press on a zoomed picture may travel and still be a click.
pub(super) const CLICK_SLOP: f64 = 4.0;

/// What a zoom is bounded by: the frame, and the picture as it is shown in it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Fit {
    pub(super) frame: Dimensions,
    pub(super) picture: Dimensions,
}

impl Fit {
    /// The picture as `object-fit: scale-down` shows it. Without a natural size
    /// it fills the frame, which only makes the pan bounds generous.
    pub(super) fn new(frame: Dimensions, natural: Option<Dimensions>) -> Self {
        let picture = match natural {
            Some(natural) => {
                let ratio = (frame.width / natural.width)
                    .min(frame.height / natural.height)
                    .min(1.0);
                Dimensions {
                    width: natural.width * ratio,
                    height: natural.height * ratio,
                }
            }
            None => frame,
        };
        Self { frame, picture }
    }
}

/// The zoom of picture `index`; any other picture reads it as fitted, so a
/// new one never paints zoomed first.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Zoom {
    pub(super) index: usize,
    pub(super) scale: f64,
    /// On-screen offset of the picture's centre, in px.
    pub(super) x: f64,
    pub(super) y: f64,
}

impl Zoom {
    pub(super) fn fitted(index: usize) -> Self {
        Self {
            index,
            scale: 1.0,
            x: 0.0,
            y: 0.0,
        }
    }

    pub(super) fn is_zoomed(self) -> bool {
        self.scale > 1.0
    }

    /// Keeps the picture covering the frame: it may move by its overhang,
    /// `(picture * s - frame) / 2`, and not at all while smaller.
    pub(super) fn clamped(self, fit: Fit) -> Self {
        let bound = |picture: f64, frame: f64| ((picture * self.scale - frame) / 2.0).max(0.0);
        let x_bound = bound(fit.picture.width, fit.frame.width);
        let y_bound = bound(fit.picture.height, fit.frame.height);
        Self {
            x: self.x.clamp(-x_bound, x_bound),
            y: self.y.clamp(-y_bound, y_bound),
            ..self
        }
    }

    /// Rescales about `point`, measured from the frame's centre, so the spot
    /// under the cursor stays under it.
    pub(super) fn scaled(self, scale: f64, point: DragPoint, fit: Fit) -> Self {
        let ratio = scale / self.scale;
        Self {
            scale,
            x: point.x * (1.0 - ratio) + self.x * ratio,
            y: point.y * (1.0 - ratio) + self.y * ratio,
            ..self
        }
        .clamped(fit)
    }

    pub(super) fn panned(self, dx: f64, dy: f64, fit: Fit) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            ..self
        }
        .clamped(fit)
    }

    /// Brings the spot at `point`, measured from the frame's centre, to the
    /// centre: the single-pointer pan (2.5.7).
    pub(super) fn centred_on(self, point: DragPoint, fit: Fit) -> Self {
        self.panned(-point.x, -point.y, fit)
    }

    /// Scale about the centre, then shift: the translation is divided by the
    /// scale because it is applied inside it.
    pub(super) fn transform(self) -> String {
        format!(
            "scale({s}) translate({x}px, {y}px)",
            s = self.scale,
            x = self.x / self.scale,
            y = self.y / self.scale,
        )
    }
}

/// What the current pointer drag on the picture is doing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Gesture {
    Pan {
        origin: DragPoint,
    },
    Swipe {
        delta: DragPoint,
    },
    /// Two fingers `distance` apart at `mid` went down on `from`. `origin` is
    /// `mid` from the frame's centre, once the frame is measured.
    Pinch {
        from: Zoom,
        distance: f64,
        mid: DragPoint,
        origin: Option<(Fit, DragPoint)>,
    },
    /// A pinch lost a finger: the one left pans from where the pinch put it.
    Settling,
}

/// Two fingers' distance and midpoint.
pub(super) fn span(a: DragPoint, b: DragPoint) -> (f64, DragPoint) {
    (
        (a.x - b.x).hypot(a.y - b.y),
        DragPoint {
            x: (a.x + b.x) / 2.0,
            y: (a.y + b.y) / 2.0,
        },
    )
}

/// A pinch at `scale`: about its starting midpoint `origin`, then moved by
/// `shift`, the midpoint's travel since.
pub(super) fn pinched(
    from: Zoom,
    scale: f64,
    origin: DragPoint,
    shift: DragPoint,
    fit: Fit,
) -> Zoom {
    match scale > 1.0 {
        true => from
            .scaled(scale, origin, fit)
            .panned(shift.x, shift.y, fit),
        false => Zoom::fitted(from.index),
    }
}

/// One wheel notch, or one `+` / `-`, from `scale`: out when `closer`.
pub(super) fn wheel_step(scale: f64, closer: bool, max_zoom: f64) -> f64 {
    match closer {
        true => scale / WHEEL_FACTOR,
        false => scale * WHEEL_FACTOR,
    }
    .clamp(1.0, max_zoom)
}

/// One double-click or `z` from `scale`: the next of 2x, 4x, 8x... past it,
/// capped at `max_zoom`, and from `max_zoom` back to fit.
pub(super) fn zoom_step(scale: f64, max_zoom: f64) -> f64 {
    if scale >= max_zoom {
        return 1.0;
    }
    let mut step = FIRST_STEP_ZOOM;
    while step <= scale {
        step *= 2.0;
    }
    step.min(max_zoom)
}

/// Whether a swipe travelled far enough, and mostly downwards, to close.
pub(super) fn swipe_closes(delta: DragPoint) -> bool {
    delta.y > SWIPE_CLOSE_DISTANCE && delta.y > delta.x.abs()
}

/// A second `open_with` lands on its clamped index, fitted: a kept zoom would
/// carry onto the new picture at the same index.
pub(super) fn reopened(zoom: Zoom, index: usize, count: usize) -> (usize, Zoom) {
    let zoom = match zoom.is_zoomed() {
        true => Zoom::fitted(usize::MAX),
        false => zoom,
    };
    (index.min(count.saturating_sub(1)), zoom)
}

/// The untransformed frame's bounds and top-left. Call in the handler, await in
/// the task: Blitz answers a read only while the document is free. The frame's
/// own handle, not a query: a WebView answers none (1068).
pub(super) fn measure_fit(
    frame: ElementHandle,
    picture: ElementHandle,
) -> impl Future<Output = Option<(Fit, f64, f64)>> {
    let (dimensions, origin, natural) = (
        frame.dimensions(),
        frame.client_offset(),
        picture.natural_size(),
    );
    async move {
        let (Ok(dimensions), Ok((left, top))) = (dimensions.await, origin.await) else {
            return None;
        };
        Some((Fit::new(dimensions, natural.await.ok()), left, top))
    }
}

/// A viewport point as an offset from the centre of a frame at `left`/`top`.
pub(super) fn from_frame_centre(client: DragPoint, bounds: Fit, left: f64, top: f64) -> DragPoint {
    DragPoint {
        x: client.x - left - bounds.frame.width / 2.0,
        y: client.y - top - bounds.frame.height / 2.0,
    }
}

/// Rescales picture `index` to `next(current scale)`, about `client` or the
/// centre, once the frame is measured.
pub(super) fn zoom_about(
    frame: ElementHandle,
    picture: ElementHandle,
    zooming: Zooming,
    index: usize,
    client: Option<DragPoint>,
    next: impl Fn(f64) -> f64 + 'static,
) {
    let (mut zoom, mut fit) = (zooming.zoom, zooming.fit);
    let measured = measure_fit(frame, picture);
    spawn(async move {
        let Some((bounds, left, top)) = measured.await else {
            return;
        };
        fit.set(Some(bounds));
        let point = match client {
            Some(client) => from_frame_centre(client, bounds, left, top),
            None => DragPoint { x: 0.0, y: 0.0 },
        };
        let from = match *zoom.peek() {
            held if held.index == index => held,
            _ => Zoom::fitted(index),
        };
        let scale = next(from.scale);
        let to = match scale > 1.0 {
            true => from.scaled(scale, point, bounds),
            false => Zoom::fitted(index),
        };
        zoom.set(to);
        if to.scale != from.scale {
            zooming.announce(to.scale);
        }
    });
}

/// A click on zoomed picture `index` brings the spot clicked to the centre.
pub(super) fn centre_on(
    frame: ElementHandle,
    picture: ElementHandle,
    zooming: Zooming,
    index: usize,
    client: DragPoint,
) {
    let (mut zoom, mut fit) = (zooming.zoom, zooming.fit);
    let measured = measure_fit(frame, picture);
    spawn(async move {
        let Some((bounds, left, top)) = measured.await else {
            return;
        };
        fit.set(Some(bounds));
        let from = *zoom.peek();
        if from.index == index && from.is_zoomed() {
            zoom.set(from.centred_on(from_frame_centre(client, bounds, left, top), bounds));
        }
    });
}

/// After the stage resized: new bounds for the zoomed picture, and its pan
/// clamped to them, so it still covers its frame.
pub(super) fn refit(frame: ElementHandle, picture: ElementHandle, zooming: Zooming) {
    let (mut zoom, mut fit, held) = (zooming.zoom, zooming.fit, zooming.held());
    if !held.is_zoomed() {
        return;
    }
    let measured = measure_fit(frame, picture);
    spawn(async move {
        let Some((bounds, ..)) = measured.await else {
            return;
        };
        // Only the zoom this measured: a move or a zoom out meanwhile wins.
        let now = *zoom.peek();
        if now.index != held.index || !now.is_zoomed() {
            return;
        }
        fit.set(Some(bounds));
        if now.clamped(bounds) != now {
            zoom.set(now.clamped(bounds));
        }
    });
}

/// One picture's handles: the frame measures, the picture holds the drag.
#[derive(Clone, Copy)]
pub(super) struct Slide {
    pub(super) frame: ElementHandle,
    pub(super) picture: ElementHandle,
}

/// The zoom and the gesture, as one `Copy` value every input shares.
#[derive(Clone, Copy)]
pub(super) struct Zooming {
    pub(super) index: Signal<usize>,
    pub(super) zoom: Signal<Zoom>,
    /// Measured whenever a zoom starts, so a zoomed picture always has bounds
    /// to pan within - and a key can decide synchronously whether it pans.
    pub(super) fit: Signal<Option<Fit>>,
    pub(super) gesture: Signal<Option<Gesture>>,
    /// The press on the picture travelled past [`CLICK_SLOP`]: its click is
    /// the end of a pan, not a click.
    pub(super) dragged: Signal<bool>,
    /// The touches down on the picture, for a pinch: the drag follows one.
    pub(super) touches: Signal<Vec<(i32, DragPoint)>>,
    pub(super) max_zoom: f64,
    pub(super) announcer: Announcer,
    pub(super) labels: LightboxLabels,
}

impl Zooming {
    /// Says the new scale: a zoom is otherwise silent to a screen reader.
    pub(super) fn announce(self, scale: f64) {
        self.announcer.say(match scale > 1.0 {
            true => fill(self.labels.zoomed, &[("n", &(scale * 100.0).round())]),
            false => self.labels.fitted.to_string(),
        });
    }

    /// The zoom held for the picture showing, or a fresh fit when the held one
    /// belongs to another index.
    pub(super) fn held(self) -> Zoom {
        match *self.zoom.peek() {
            held if held.index == *self.index.peek() => held,
            _ => Zoom::fitted(*self.index.peek()),
        }
    }

    /// Double-click and `z` step through [`zoom_step`].
    pub(super) fn step_scale(self, scale: f64) -> f64 {
        zoom_step(scale, self.max_zoom)
    }

    /// A second finger went down: the pinch takes over from any pan or swipe.
    pub(super) fn start_pinch(self, slide: Slide) {
        let (mut gesture, mut fit, mut dragged) = (self.gesture, self.fit, self.dragged);
        let touches = self.touches.peek();
        let [(_, a), (_, b), ..] = touches.as_slice() else {
            return;
        };
        let (distance, mid) = span(*a, *b);
        let from = self.held();
        gesture.set(Some(Gesture::Pinch {
            from,
            distance,
            mid,
            origin: None,
        }));
        // Its fingers' lift is no click to centre on.
        dragged.set(true);
        let measured = measure_fit(slide.frame, slide.picture);
        spawn(async move {
            let Some((bounds, left, top)) = measured.await else {
                return;
            };
            fit.set(Some(bounds));
            let waiting = matches!(*gesture.peek(), Some(Gesture::Pinch { origin: None, .. }));
            if waiting {
                let origin = Some((bounds, from_frame_centre(mid, bounds, left, top)));
                gesture.set(Some(Gesture::Pinch {
                    from,
                    distance,
                    mid,
                    origin,
                }));
            }
        });
    }

    /// Scales by the fingers' spread and pans by their midpoint's travel.
    pub(super) fn pinch_moved(self) {
        let mut zoom = self.zoom;
        let Some(Gesture::Pinch {
            from,
            distance,
            mid,
            origin: Some((fit, origin)),
        }) = *self.gesture.peek()
        else {
            return;
        };
        let touches = self.touches.peek();
        let [(_, a), (_, b), ..] = touches.as_slice() else {
            return;
        };
        let (now, at) = span(*a, *b);
        if distance < 1.0 {
            return;
        }
        let scale = (from.scale * now / distance).clamp(1.0, self.max_zoom);
        let shift = DragPoint {
            x: at.x - mid.x,
            y: at.y - mid.y,
        };
        let to = pinched(from, scale, origin, shift, fit);
        if *zoom.peek() != to {
            zoom.set(to);
        }
    }

    /// A touch lifted or was cancelled: a pinch left with one finger settles.
    pub(super) fn lift(self, pointer_id: i32) {
        let (mut touches, mut gesture) = (self.touches, self.gesture);
        if !touches.peek().iter().any(|(id, _)| *id == pointer_id) {
            return;
        }
        touches.write().retain(|(id, _)| *id != pointer_id);
        let left = touches.peek().len();
        let held = *gesture.peek();
        match held {
            Some(Gesture::Pinch { from, .. }) if left < 2 => {
                gesture.set((left > 0).then_some(Gesture::Settling));
                let to = self.held();
                if to.scale != from.scale {
                    self.announce(to.scale);
                }
            }
            Some(Gesture::Settling) if left == 0 => gesture.set(None),
            _ => {}
        }
    }
}
