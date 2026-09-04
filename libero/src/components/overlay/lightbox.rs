use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        Box, Carousel, Dialog, Input, States, Variables,
        common::{focus_ring_sx, states, variables},
    },
    hooks::{
        DragMove, DragOptions, DragPoint, DragStart, ElementHandle, LightboxOpening,
        LightboxOptions, drag_handle_sx, use_drag, use_element, use_id, use_modal_close, use_theme,
    },
    platform::{Dimensions, ElementApi},
    sx::{StaticSx, sx},
    theme::{
        CarouselDefaults, CssVar, LIGHTBOX_STAGE_HEIGHT, LIGHTBOX_THUMBNAIL_SIZE,
        LIGHTBOX_THUMBNAILS_GAP, LIGHTBOX_WIDTH,
    },
};

/// Pixels one arrow press pans a zoomed picture.
const PAN_STEP: f64 = 48.0;
/// Scale change per wheel notch.
const WHEEL_FACTOR: f64 = 1.25;
/// Double-click and `z` zoom to this, or to `max_zoom` if that is lower.
const TOGGLE_ZOOM: f64 = 2.0;
/// How far down a swipe has to travel to close.
const SWIPE_CLOSE_DISTANCE: f64 = 96.0;

/// The current picture's transform, written per instance. Private: nothing
/// themes a zoom.
const LIGHTBOX_TRANSFORM: CssVar = CssVar::new("--lsx-lightbox-transform");
/// Thumbnails actually shown at once, so a short strip is only as wide as its
/// thumbnails and centres.
const LIGHTBOX_THUMBNAILS_SHOWN: CssVar = CssVar::new("--lsx-lightbox-thumbnails-shown");

static LIGHTBOX_DIALOG_SX: StaticSx = StaticSx::new(|| {
    sx().width("100%")
        .max_width(LIGHTBOX_WIDTH.value())
        .padding("sm")
});

// Every slide is this one box, so a picture is fitted into the stage rather
// than sizing it, and the pan bounds are the same for all of them. The ring
// sits here, inset, because the picture itself is scaled while zoomed and an
// outline on it would be scaled and clipped with it.
static LIGHTBOX_FRAME_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative")
        .height(LIGHTBOX_STAGE_HEIGHT.value())
        .overflow("hidden")
        .has_focus_visible(focus_ring_sx().outline_offset("-2px"))
});

static LIGHTBOX_IMAGE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .height("100%")
        .object_fit("contain")
        // Replaces `Box`'s own ring: the frame draws this one.
        .focus_visible(sx().outline("none"))
        .transform(LIGHTBOX_TRANSFORM.value_or("none"))
        .transition("transform 150ms ease")
        .media("(prefers-reduced-motion: reduce)", sx().transition("none"))
        .when("zoomable", sx().cursor("zoom-in"))
        // The carousel still scrolls sideways natively; a vertical move is
        // left to the swipe-down.
        .when("swipe", sx().touch_action("pan-x"))
        .when("zoomed", drag_handle_sx().cursor("grab"))
        // A drag is the pointer's own position: easing towards it lags.
        .when("dragging", sx().cursor("grabbing").transition("none"))
});

static LIGHTBOX_CAPTION_SX: StaticSx =
    StaticSx::new(|| sx().margin("0").margin_top("sm").text_align("center"));

static LIGHTBOX_THUMBNAILS_SX: StaticSx = StaticSx::new(|| {
    sx().margin_top("sm")
        .margin_left("auto")
        .margin_right("auto")
        .max_width(format!(
            "calc({shown} * {size} + ({shown} - 1) * {gap})",
            shown = LIGHTBOX_THUMBNAILS_SHOWN.value(),
            size = LIGHTBOX_THUMBNAIL_SIZE.value(),
            gap = LIGHTBOX_THUMBNAILS_GAP.value(),
        ))
});

// The padding is the current thumbnail's frame: its background shows round the
// picture. A second channel beside `aria-current`, with no opacity on the
// others, which would dim their focus ring too.
static LIGHTBOX_THUMBNAIL_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .aspect_ratio("1")
        .padding("2px")
        .border_width("0")
        .background("transparent")
        .cursor("pointer")
        .when("current", sx().background("primary.6"))
        // Inset: the carousel slide around it clips.
        .focus_visible(focus_ring_sx().outline_offset("-2px"))
});

static LIGHTBOX_THUMBNAIL_IMAGE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .height("100%")
        .object_fit("cover")
});

/// The zoom of one picture. Carries the index it belongs to, so moving to
/// another picture resets it without a write: a zoom for any other index reads
/// as fitted. Mantine resets during render for the same reason - the new
/// picture must never paint zoomed first.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Zoom {
    index: usize,
    scale: f64,
    /// On-screen offset of the picture's centre, in px.
    x: f64,
    y: f64,
}

impl Zoom {
    fn fitted(index: usize) -> Self {
        Self {
            index,
            scale: 1.0,
            x: 0.0,
            y: 0.0,
        }
    }

    fn is_zoomed(self) -> bool {
        self.scale > 1.0
    }

    /// Keeps the picture covering the frame: at scale `s` it overhangs by
    /// `length * (s - 1) / 2` on each side, and that is how far it may move.
    fn clamped(self, size: Dimensions) -> Self {
        let bound = |length: f64| (length * (self.scale - 1.0) / 2.0).max(0.0);
        let (x_bound, y_bound) = (bound(size.width), bound(size.height));
        Self {
            x: self.x.clamp(-x_bound, x_bound),
            y: self.y.clamp(-y_bound, y_bound),
            ..self
        }
    }

    /// Rescales about `point`, measured from the frame's centre, so the spot
    /// under the cursor stays under it.
    fn scaled(self, scale: f64, point: DragPoint, size: Dimensions) -> Self {
        let ratio = scale / self.scale;
        Self {
            scale,
            x: point.x * (1.0 - ratio) + self.x * ratio,
            y: point.y * (1.0 - ratio) + self.y * ratio,
            ..self
        }
        .clamped(size)
    }

    fn panned(self, dx: f64, dy: f64, size: Dimensions) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            ..self
        }
        .clamped(size)
    }

    /// Scale about the centre, then shift: the translation is divided by the
    /// scale because it is applied inside it.
    fn transform(self) -> String {
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
enum Gesture {
    Pan { origin: DragPoint },
    Swipe { delta: DragPoint },
}

/// Whether a swipe travelled far enough, and mostly downwards, to close.
fn swipe_closes(delta: DragPoint) -> bool {
    delta.y > SWIPE_CLOSE_DISTANCE && delta.y > delta.x.abs()
}

fn frame_selector(index: usize) -> String {
    format!("[data-lightbox-frame=\"{index}\"]")
}

fn image_id(base: &str, index: usize) -> String {
    format!("{base}-image-{index}")
}

fn thumbnail_id(base: &str, index: usize) -> String {
    format!("{base}-thumbnail-{index}")
}

/// Rescales picture `index` to `next(current scale)`, about `client` or the
/// centre. The frame is measured first - both for the pan bounds and for where
/// the cursor sits in it - so the zoom lands once the read does.
fn zoom_about(
    stage: ElementHandle,
    mut zoom: Signal<Zoom>,
    mut size: Signal<Option<Dimensions>>,
    index: usize,
    client: Option<DragPoint>,
    next: impl Fn(f64) -> f64 + 'static,
) {
    let Ok(frame) = stage.query_selector(&frame_selector(index)) else {
        return;
    };
    let (dimensions, origin) = (frame.dimensions(), frame.client_offset());
    spawn(async move {
        let (Ok(dimensions), Ok((left, top))) = (dimensions.await, origin.await) else {
            return;
        };
        size.set(Some(dimensions));
        let point = match client {
            Some(client) => DragPoint {
                x: client.x - left - dimensions.width / 2.0,
                y: client.y - top - dimensions.height / 2.0,
            },
            None => DragPoint { x: 0.0, y: 0.0 },
        };
        let from = match *zoom.peek() {
            held if held.index == index => held,
            _ => Zoom::fitted(index),
        };
        let scale = next(from.scale);
        zoom.set(match scale > 1.0 {
            true => from.scaled(scale, point, dimensions),
            false => Zoom::fitted(index),
        });
    });
}

/// The viewer [`crate::hooks::use_lightbox`] opens. Only rendered inside that
/// hook's modal.
#[component]
pub(crate) fn Lightbox(opening: LightboxOpening, options: LightboxOptions) -> Element {
    let theme = use_theme();
    let close = use_modal_close();
    let stage = use_element();
    let base_id = use_id();
    let caption_id = use_id();

    let items = opening.items.clone();
    let count = items.len();
    let last = count.saturating_sub(1);
    let mut index = use_signal(|| opening.index.min(last));
    // A second `open_with` while this one shows replaces the gallery in place.
    use_effect(use_reactive!(|opening| {
        let target = opening.index.min(opening.items.len().saturating_sub(1));
        if *index.peek() != target {
            index.set(target);
        }
    }));

    let mut zoom = use_signal(|| Zoom::fitted(usize::MAX));
    // Measured whenever a zoom starts, so a zoomed picture always has bounds
    // to pan within - and a key can decide synchronously whether it pans.
    let size = use_signal(|| None::<Dimensions>);
    let mut gesture = use_signal(|| None::<Gesture>);

    let current = index();
    let held = move || match *zoom.peek() {
        held if held.index == *index.peek() => held,
        _ => Zoom::fitted(*index.peek()),
    };
    let active = match zoom() {
        held if held.index == current => held,
        _ => Zoom::fitted(current),
    };
    let max_zoom = options.max_zoom.unwrap_or(theme.lightbox.max_zoom).max(1.0);
    let toggle_scale = move |scale: f64| match scale > 1.0 {
        true => 1.0,
        false => TOGGLE_ZOOM.min(max_zoom),
    };
    let zoomable = options.zoom;
    let swipe = options.close_on_swipe_down;

    // Moves to picture `target`, and focus with it when focus was on the
    // picture - otherwise the keyboard is stranded on a slide scrolled away.
    let go = move |target: usize, refocus: bool| {
        let target = target.min(last);
        let mut index = index;
        if target == *index.peek() {
            return;
        }
        index.set(target);
        if refocus
            && let Ok(image) = stage.query_selector(&format!("#{}", image_id(&base_id(), target)))
        {
            let _ = image.focus();
        }
    };

    // One handle per picture, and the drag captures on the one showing. Not on
    // the stage: pointer capture retargets the click that follows a press
    // ([[codebase/components/scroller]]), so a double-click on a zoomed
    // picture would land on the stage, not on the picture's own
    // `ondoubleclick`. Grown in render,
    // where a signal may be created, because a later opening can bring more
    // pictures; a `RefCell`, not a signal, so growing it re-renders nothing.
    let pictures = use_hook(|| Rc::new(RefCell::new(Vec::<ElementHandle>::new())));
    {
        let mut pictures = pictures.borrow_mut();
        while pictures.len() < count {
            pictures.push(ElementHandle::new());
        }
    }
    let picture = |i: usize| pictures.borrow()[i];

    let drag = use_drag(DragOptions {
        capture: if count > 0 { picture(current) } else { stage },
        on_start: Callback::new(move |_: DragStart| {}),
        on_move: Callback::new(move |moved: DragMove| {
            let delta = moved.delta();
            let held_gesture = *gesture.peek();
            match held_gesture {
                Some(Gesture::Pan { origin }) => {
                    let Some(dimensions) = *size.peek() else {
                        return;
                    };
                    let from = held();
                    zoom.set(
                        Zoom {
                            x: origin.x + delta.x,
                            y: origin.y + delta.y,
                            ..from
                        }
                        .clamped(dimensions),
                    );
                }
                Some(Gesture::Swipe { .. }) => gesture.set(Some(Gesture::Swipe { delta })),
                None => {}
            }
        }),
        on_end: Callback::new(move |()| {
            if let Some(Gesture::Swipe { delta }) = *gesture.peek()
                && swipe_closes(delta)
            {
                close.call(());
            }
            gesture.set(None);
        }),
    });

    let show_captions = options.captions;
    let caption = items
        .get(current)
        .and_then(|item| item.caption.clone())
        .filter(|_| show_captions);
    let described = caption.as_ref().map(|_| caption_id());

    let swiping = match gesture() {
        Some(Gesture::Swipe { delta }) => Some(delta.y.max(0.0)),
        _ => None,
    };

    let slide = |i: usize| -> Element {
        let item = &items[i];
        let is_current = i == current;
        let zoomed = is_current && active.is_zoomed();
        let image_states: Input<States> = states()
            .with("zoomable", zoomable && !zoomed)
            .with("swipe", swipe && !zoomed)
            .with("zoomed", zoomed)
            .with("dragging", is_current && gesture().is_some())
            .into();
        let transform = match (is_current, swiping) {
            (false, _) => None,
            (true, Some(down)) => Some(format!("translateY({down}px)")),
            (true, None) => active.is_zoomed().then(|| active.transform()),
        };
        let image_variables: Input<Variables> =
            variables().with(LIGHTBOX_TRANSFORM, transform).into();
        let loading = match i.abs_diff(current) <= options.preload {
            true => "eager",
            false => "lazy",
        };

        rsx! {
            Box { framework_sx: &LIGHTBOX_FRAME_SX, "data-lightbox-frame": i,
                Box {
                    component: "img",
                    framework_sx: &LIGHTBOX_IMAGE_SX,
                    states: image_states,
                    variables: image_variables,
                    id: image_id(&base_id(), i),
                    src: item.src.clone(),
                    alt: item.alt.clone(),
                    loading,
                    draggable: "false",
                    // The picture is the pan surface, so it takes the keys:
                    // one tab stop, on the one showing.
                    tabindex: zoomable.then_some(if is_current { "0" } else { "-1" }),
                    aria_describedby: if is_current { described.clone() } else { None },
                    onkeydown: move |event: Event<KeyboardData>| {
                        if !zoomable || i != *index.peek() {
                            return;
                        }
                        let from = held();
                        let pan = match event.key() {
                            Key::ArrowLeft => Some((PAN_STEP, 0.0)),
                            Key::ArrowRight => Some((-PAN_STEP, 0.0)),
                            Key::ArrowUp => Some((0.0, PAN_STEP)),
                            Key::ArrowDown => Some((0.0, -PAN_STEP)),
                            _ => None,
                        };
                        if let Some((dx, dy)) = pan
                            && from.is_zoomed()
                            && let Some(dimensions) = *size.peek()
                        {
                            let moved = from.panned(dx, dy, dimensions);
                            // Only a pan that moved is a pan. At the edge the
                            // key falls through to the slide change, so a
                            // zoomed picture never traps the keyboard - our
                            // call, not Mantine's, which never falls through.
                            if moved != from {
                                event.prevent_default();
                                zoom.set(moved);
                                return;
                            }
                        }
                        match event.key() {
                            Key::ArrowLeft => go(i.saturating_sub(1), true),
                            Key::ArrowRight => go(i + 1, true),
                            Key::Home => go(0, true),
                            Key::End => go(last, true),
                            Key::Character(ref c)
                                if c.eq_ignore_ascii_case("z")
                                    && !event.modifiers().ctrl()
                                    && !event.modifiers().meta()
                                    && !event.modifiers().alt() =>
                            {
                                zoom_about(stage, zoom, size, i, None, toggle_scale);
                            }
                            _ => return,
                        }
                        event.prevent_default();
                    },
                    onwheel: move |event: Event<WheelData>| {
                        if !zoomable || i != *index.peek() {
                            return;
                        }
                        event.prevent_default();
                        let closer = event.data().delta().strip_units().y > 0.0;
                        let client = event.client_coordinates();
                        zoom_about(
                            stage,
                            zoom,
                            size,
                            i,
                            Some(DragPoint { x: client.x, y: client.y }),
                            move |scale| {
                                match closer {
                                    true => scale / WHEEL_FACTOR,
                                    false => scale * WHEEL_FACTOR,
                                }
                                .clamp(1.0, max_zoom)
                            },
                        );
                    },
                    ondoubleclick: move |event: Event<MouseData>| {
                        if !zoomable || i != *index.peek() {
                            return;
                        }
                        let client = event.client_coordinates();
                        zoom_about(
                            stage,
                            zoom,
                            size,
                            i,
                            Some(DragPoint { x: client.x, y: client.y }),
                            toggle_scale,
                        );
                    },
                    onmounted: picture(i).mount(),
                    onpointermove: drag.onpointermove,
                    onpointerup: drag.onpointerup,
                    onpointercancel: drag.onpointercancel,
                    onpointerdown: move |event: Event<PointerData>| {
                        if i != *index.peek() {
                            return;
                        }
                        let from = held();
                        let next = match (from.is_zoomed(), event.data().pointer_type() == "mouse") {
                            (true, _) => Gesture::Pan {
                                origin: DragPoint { x: from.x, y: from.y },
                            },
                            // A mouse has the arrows and Escape; a swipe is a
                            // touch gesture.
                            (false, false) if swipe => Gesture::Swipe {
                                delta: DragPoint { x: 0.0, y: 0.0 },
                            },
                            _ => return,
                        };
                        gesture.set(Some(next));
                        drag.onpointerdown.call(event);
                    },
                }
            }
        }
    };

    let slides: Vec<Element> = (0..count).map(slide).collect();
    let label = options
        .aria_label
        .clone()
        .unwrap_or_else(|| theme.lightbox.label.to_string());

    let stage_body = match count {
        // One picture has nothing to page through, so no carousel: its region
        // and "1 of 1" would only be noise.
        0 | 1 => rsx! { {slides.into_iter()} },
        _ => rsx! {
            Carousel {
                aria_label: label.clone(),
                slides,
                index: Some(current),
                onindexchange: move |next: usize| {
                    if next != *index.peek() {
                        index.set(next);
                    }
                },
                controls: options.controls,
                indicators: false,
            }
        },
    };

    let shown = (count as f64).min(theme.lightbox.thumbnails_per_view.max(1.0));
    let thumbnails_variables: Input<Variables> = variables()
        .with(LIGHTBOX_THUMBNAILS_SHOWN, shown.to_string())
        .into();
    let thumbnails: Vec<Element> = (0..count)
        .map(|i| {
            let item = &items[i];
            let is_current = i == current;
            let thumbnail_states: Input<States> = states().with("current", is_current).into();
            rsx! {
                Box {
                    component: "button",
                    r#type: "button",
                    framework_sx: &LIGHTBOX_THUMBNAIL_SX,
                    states: thumbnail_states,
                    id: thumbnail_id(&base_id(), i),
                    aria_label: CarouselDefaults::format_label(theme.lightbox.thumbnail_label, i, count),
                    aria_current: is_current.then(|| "true".to_string()),
                    // Roving: one tab stop for the strip.
                    tabindex: if is_current { "0" } else { "-1" },
                    onclick: move |_| go(i, false),
                    onkeydown: move |event: Event<KeyboardData>| {
                        let target = match event.key() {
                            Key::ArrowLeft => i.saturating_sub(1),
                            Key::ArrowRight => (i + 1).min(last),
                            Key::Home => 0,
                            Key::End => last,
                            _ => return,
                        };
                        event.prevent_default();
                        go(target, false);
                        if let Ok(thumbnail) =
                            stage.query_selector(&format!("#{}", thumbnail_id(&base_id(), target)))
                        {
                            let _ = thumbnail.focus();
                        }
                    },
                    Box {
                        component: "img",
                        framework_sx: &LIGHTBOX_THUMBNAIL_IMAGE_SX,
                        src: item.thumbnail_src.clone().unwrap_or_else(|| item.src.clone()),
                        alt: "",
                        draggable: "false",
                    }
                }
            }
        })
        .collect();
    let show_thumbnails = options.thumbnails && count > 1;

    rsx! {
        Dialog {
            aria_label: label.clone(),
            close_label: theme.lightbox.close_label,
            sx: &LIGHTBOX_DIALOG_SX,
            Box {
                onmounted: stage.mount(),
                {stage_body}
                if let Some(caption) = caption {
                    Box { component: "p", id: caption_id(), framework_sx: &LIGHTBOX_CAPTION_SX, "{caption}" }
                }
                if show_thumbnails {
                    Box { framework_sx: &LIGHTBOX_THUMBNAILS_SX, variables: thumbnails_variables,
                        Carousel {
                            aria_label: theme.lightbox.thumbnails_label,
                            slides: thumbnails,
                            // Follows the picture, never leads it: a clamped
                            // index near the ends is the strip's business.
                            index: Some(current),
                            per_view: shown,
                            gap: theme.lightbox.thumbnails_gap,
                            align: "center",
                            controls: false,
                            indicators: false,
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: Dimensions = Dimensions {
        width: 800.0,
        height: 600.0,
    };

    #[test]
    fn a_fitted_picture_cannot_move() {
        let fitted = Zoom::fitted(0);

        assert_eq!(fitted.panned(40.0, -40.0, FRAME), fitted);
    }

    /// At scale 2 an 800px picture overhangs 400px each side, and no further.
    #[test]
    fn a_pan_stops_where_the_picture_would_leave_the_frame() {
        let zoomed = Zoom {
            scale: 2.0,
            ..Zoom::fitted(0)
        };

        let far = zoomed.panned(10_000.0, -10_000.0, FRAME);

        assert_eq!((far.x, far.y), (400.0, -300.0));
    }

    /// The key-at-the-edge rule rests on this: once clamped, another pan the
    /// same way is no change at all, so the key goes to the slide instead.
    #[test]
    fn a_pan_at_the_edge_is_no_change() {
        let at_edge = Zoom {
            scale: 2.0,
            x: 400.0,
            ..Zoom::fitted(0)
        };

        assert_eq!(at_edge.panned(PAN_STEP, 0.0, FRAME), at_edge);
        assert_ne!(at_edge.panned(-PAN_STEP, 0.0, FRAME), at_edge);
    }

    /// A pan short of the edge by less than a step still moves, and lands on
    /// the edge - the next press falls through.
    #[test]
    fn a_pan_that_reaches_the_edge_is_clamped_onto_it() {
        let near = Zoom {
            scale: 2.0,
            x: 390.0,
            ..Zoom::fitted(0)
        };

        assert_eq!(near.panned(PAN_STEP, 0.0, FRAME).x, 400.0);
    }

    #[test]
    fn zooming_about_the_centre_does_not_move_the_picture() {
        let zoomed = Zoom::fitted(0).scaled(2.0, DragPoint { x: 0.0, y: 0.0 }, FRAME);

        assert_eq!((zoomed.scale, zoomed.x, zoomed.y), (2.0, 0.0, 0.0));
    }

    /// The spot under the cursor stays under it: a point 100px right of centre
    /// on a fitted picture is still 100px right of centre at scale 2.
    #[test]
    fn zooming_about_a_point_keeps_that_point_still() {
        let point = DragPoint { x: 100.0, y: -50.0 };
        let zoomed = Zoom::fitted(0).scaled(2.0, point, FRAME);

        // Image-local position of the point, then back to the screen.
        let local = (point.x / 1.0, point.y / 1.0);
        let screen = (
            zoomed.x + zoomed.scale * local.0,
            zoomed.y + zoomed.scale * local.1,
        );
        assert_eq!(screen, (point.x, point.y));
    }

    #[test]
    fn the_transform_divides_the_shift_by_the_scale() {
        let zoomed = Zoom {
            scale: 2.0,
            x: 100.0,
            y: -40.0,
            ..Zoom::fitted(0)
        };

        assert_eq!(zoomed.transform(), "scale(2) translate(50px, -20px)");
    }

    #[test]
    fn only_a_long_mostly_downward_swipe_closes() {
        assert!(swipe_closes(DragPoint { x: 10.0, y: 120.0 }));
        assert!(!swipe_closes(DragPoint { x: 0.0, y: 40.0 }));
        assert!(!swipe_closes(DragPoint { x: 200.0, y: 120.0 }));
        assert!(!swipe_closes(DragPoint { x: 0.0, y: -200.0 }));
    }
}
