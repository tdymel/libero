//! The pictures and the thumbnail strip, and the drag on a picture.

use dioxus::{html::input_data::MouseButton, prelude::*};

use super::{
    lightbox::LightboxPart,
    styles::{
        LIGHTBOX_FRAME_SX, LIGHTBOX_IMAGE_SX, LIGHTBOX_THUMBNAIL_IMAGE_SX, LIGHTBOX_THUMBNAIL_SX,
        LIGHTBOX_TRANSFORM,
    },
    use_lightbox::LightboxItem,
    zoom::{
        CLICK_SLOP, Gesture, PAN_STEP, Slide, Zoom, Zooming, centre_on, swipe_closes, wheel_step,
        zoom_about,
    },
};
use crate::{
    components::{
        common::{
            Input, Part, SVG_FIT, States, Variables, has_shortcut_modifier, ring_overlay, states,
            svg_fit, svg_fit_variables, variables,
        },
        layout::Box,
    },
    hooks::{Drag, DragMove, DragOptions, DragPoint, DragStart, ElementHandle, use_drag},
    localization::fill,
    platform::{self, ElementApi, logical_key},
};

pub(super) fn image_id(base: &str, index: usize) -> String {
    format!("{base}-image-{index}")
}

pub(super) fn thumbnail_id(base: &str, index: usize) -> String {
    format!("{base}-thumbnail-{index}")
}

/// Everything a picture and a thumbnail read: the zoom, the drag, the ids and
/// the mover.
#[derive(Clone, Copy)]
pub(super) struct Stage {
    pub(super) zooming: Zooming,
    pub(super) drag: Drag,
    pub(super) base_id: Signal<String>,
    /// The element a move sends focus to, read by the effect that focuses it.
    pub(super) focus_next: Signal<Option<String>>,
    pub(super) current: usize,
    pub(super) last: usize,
    pub(super) active: Zoom,
    pub(super) zoomable: bool,
    pub(super) swipe: bool,
    pub(super) preload: usize,
    /// How far a swipe-to-close has travelled down, while one is in progress.
    pub(super) swiping: Option<f64>,
}

impl Stage {
    /// Moves to picture `target`, and focus with it when focus was on the
    /// picture - otherwise the keyboard is stranded on a slide scrolled away.
    pub(super) fn go(self, target: usize, refocus: bool) {
        let target = target.min(self.last);
        let (mut index, mut focus_next) = (self.zooming.index, self.focus_next);
        if target == *index.peek() {
            return;
        }
        index.set(target);
        if refocus {
            focus_next.set(Some(image_id(&(self.base_id)(), target)));
        }
    }
}

/// The pan and the swipe-to-close, on the picture: capture on the stage would
/// retarget its double-click there ([[codebase/components/scroller]]).
pub(super) fn use_lightbox_drag(
    zooming: Zooming,
    capture: ElementHandle,
    close: Callback<()>,
) -> Drag {
    let Zooming {
        mut zoom,
        fit,
        mut gesture,
        mut dragged,
        ..
    } = zooming;

    use_drag(DragOptions {
        capture,
        onstart: Callback::new(move |_: DragStart| {}),
        onmove: Callback::new(move |moved: DragMove| {
            let delta = moved.delta();
            let held_gesture = *gesture.peek();
            match held_gesture {
                Some(Gesture::Pan { origin }) => {
                    if !*dragged.peek() && delta.x.abs().max(delta.y.abs()) > CLICK_SLOP {
                        dragged.set(true);
                    }
                    let Some(bounds) = *fit.peek() else {
                        return;
                    };
                    let from = zooming.held();
                    zoom.set(
                        Zoom {
                            x: origin.x + delta.x,
                            y: origin.y + delta.y,
                            ..from
                        }
                        .clamped(bounds),
                    );
                }
                Some(Gesture::Swipe { .. }) => gesture.set(Some(Gesture::Swipe { delta })),
                // The pan restarts from where the pinch left the picture.
                Some(Gesture::Settling) => {
                    let from = zooming.held();
                    gesture.set(from.is_zoomed().then_some(Gesture::Pan {
                        origin: DragPoint {
                            x: from.x - delta.x,
                            y: from.y - delta.y,
                        },
                    }));
                }
                Some(Gesture::Pinch { .. }) | None => {}
            }
        }),
        onend: Callback::new(move |()| {
            if let Some(Gesture::Swipe { delta }) = *gesture.peek()
                && swipe_closes(delta)
            {
                close.call(());
            }
            gesture.set(None);
        }),
    })
}

/// One picture: the pan surface, the tab stop while it is the one showing, and
/// the four ways it zooms.
pub(super) fn lightbox_slide(
    stage: Stage,
    i: usize,
    item: &LightboxItem,
    slide: Slide,
    described: Option<String>,
) -> Element {
    let Slide {
        frame,
        picture: image,
    } = slide;
    let Stage {
        zooming,
        drag,
        base_id,
        current,
        last,
        active,
        zoomable,
        swipe,
        preload,
        swiping,
        ..
    } = stage;
    let Zooming {
        index,
        mut zoom,
        fit,
        mut gesture,
        mut dragged,
        mut touches,
        max_zoom,
        ..
    } = zooming;

    let is_current = i == current;
    let zoomed = is_current && active.is_zoomed();
    let fitted = svg_fit(&item.src);
    let frame_states: Input<States> = states().with(SVG_FIT, fitted).into();
    let image_states: Input<States> = states()
        .with("zoomable", zoomable && !zoomed)
        .with("sideways", (swipe || zoomable) && !zoomed)
        .with("zoomed", zoomed)
        .with("dragging", is_current && gesture().is_some())
        .with(SVG_FIT, fitted)
        .into();
    let transform = match (is_current, swiping) {
        (false, _) => None,
        (true, Some(down)) => Some(format!("translateY({down}px)")),
        (true, None) => active.is_zoomed().then(|| active.transform()),
    };
    let image_variables: Input<Variables> = variables().with(LIGHTBOX_TRANSFORM, transform).into();
    let loading = match i.abs_diff(current) <= preload {
        true => "eager",
        false => "lazy",
    };

    // Keyed by picture: Chromium fetches a new `src` on a loaded `<img>` at once,
    // whatever `loading` says (todo 227). Inputs are the frame's (todos 920, 928).
    rsx! {
        Box {
            key: "{item.src}",
            framework_sx: &LIGHTBOX_FRAME_SX,
            states: frame_states,
            "data-lightbox-frame": i,
            "data-slot": LightboxPart::Frame.slot(),
            onwheel: move |event: Event<WheelData>| {
                if !zoomable || i != *index.peek() {
                    return;
                }
                event.prevent_default();
                let closer = platform::wheel_travel_y(&event.data(), 1.0, 1.0) > 0.0;
                let client = event.client_coordinates();
                zoom_about(
                    frame,
                    image,
                    zooming,
                    i,
                    Some(DragPoint { x: client.x, y: client.y }),
                    move |scale| wheel_step(scale, closer, max_zoom),
                );
            },
            ondoubleclick: move |event: Event<MouseData>| {
                if !zoomable || i != *index.peek() {
                    return;
                }
                let client = event.client_coordinates();
                zoom_about(
                    frame,
                    image,
                    zooming,
                    i,
                    Some(DragPoint { x: client.x, y: client.y }),
                    move |scale| zooming.step_scale(scale),
                );
            },
            // The drag still captures on the picture, so the moves and the
            // release land there.
            onpointerdown: move |event: Event<PointerData>| {
                if i != *index.peek() {
                    return;
                }
                if event.data().pointer_type() != "mouse" {
                    let (id, client) = (event.pointer_id(), event.client_coordinates());
                    let mut touches = touches.write();
                    touches.retain(|(held, _)| *held != id);
                    touches.push((id, DragPoint { x: client.x, y: client.y }));
                }
                if touches.peek().len() >= 2 {
                    if zoomable && !matches!(*gesture.peek(), Some(Gesture::Pinch { .. })) {
                        event.prevent_default();
                        // Implicit on touch; Blitz and a WebView have none to take.
                        let _ = image.set_pointer_capture(event.pointer_id());
                        zooming.start_pinch(slide);
                    }
                    return;
                }
                // `use_drag` declines a right or middle press, so no end would clear the gesture.
                if matches!(event.trigger_button(), Some(button) if button != MouseButton::Primary) {
                    return;
                }
                if *dragged.peek() {
                    dragged.set(false);
                }
                let from = zooming.held();
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
            Box {
                component: "img",
                framework_sx: &LIGHTBOX_IMAGE_SX,
                "data-slot": LightboxPart::Image.slot(),
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
                    // Ctrl/Alt/Meta chords are the browser's: Alt+Left is Back.
                    if !zoomable || i != *index.peek() || has_shortcut_modifier(&event) {
                        return;
                    }
                    let from = zooming.held();
                    let pan = match event.key() {
                        Key::ArrowLeft => Some((PAN_STEP, 0.0)),
                        Key::ArrowRight => Some((-PAN_STEP, 0.0)),
                        Key::ArrowUp => Some((0.0, PAN_STEP)),
                        Key::ArrowDown => Some((0.0, -PAN_STEP)),
                        _ => None,
                    };
                    if let Some((dx, dy)) = pan
                        && from.is_zoomed()
                        && let Some(bounds) = *fit.peek()
                    {
                        let moved = from.panned(dx, dy, bounds);
                        // At the edge the key falls through to the slide
                        // change, so a zoomed picture never traps the keyboard.
                        if moved != from {
                            event.prevent_default();
                            zoom.set(moved);
                            return;
                        }
                    }
                    // The pan above stays physical; the slides follow the
                    // strip, which mirrors under RTL.
                    match logical_key(&event) {
                        Key::ArrowLeft => stage.go(i.saturating_sub(1), true),
                        Key::ArrowRight => stage.go(i + 1, true),
                        Key::Home => stage.go(0, true),
                        Key::End => stage.go(last, true),
                        Key::Character(ref c) if c.eq_ignore_ascii_case("z") => {
                            zoom_about(frame, image, zooming, i, None, move |scale| {
                                zooming.step_scale(scale)
                            });
                        }
                        // The wheel's steps, so the keyboard reaches `max_zoom` too.
                        Key::Character(ref c) if matches!(c.as_str(), "+" | "=" | "-") => {
                            let closer = c == "-";
                            zoom_about(frame, image, zooming, i, None, move |scale| {
                                wheel_step(scale, closer, max_zoom)
                            });
                        }
                        _ => return,
                    }
                    event.prevent_default();
                },
                // Panning without a drag: a click centres the spot clicked.
                onclick: move |event: Event<MouseData>| {
                    if !zoomable || i != *index.peek() {
                        return;
                    }
                    let from = zooming.held();
                    let centres = !*dragged.peek() && from.is_zoomed();
                    let ticket = zooming.clicked(centres.then_some(from));
                    if centres {
                        let client = event.client_coordinates();
                        centre_on(frame, image, zooming, i, DragPoint { x: client.x, y: client.y }, ticket);
                    }
                },
                // Before the frame zooms: a double-click's two clicks must not pan (todo 2283).
                ondoubleclick: move |_| {
                    if !zoomable || i != *index.peek() {
                        return;
                    }
                    if let Some(before) = zooming.undo_clicks() {
                        zoom.set(before);
                    }
                },
                onpointermove: move |event: Event<PointerData>| {
                    let id = event.pointer_id();
                    let at = touches.peek().iter().position(|(held, _)| *held == id);
                    if let Some(at) = at {
                        let client = event.client_coordinates();
                        touches.write()[at].1 = DragPoint { x: client.x, y: client.y };
                        zooming.pinch_moved();
                    }
                    drag.onpointermove.call(event);
                },
                onpointerup: move |event: Event<PointerData>| {
                    zooming.lift(event.pointer_id());
                    drag.onpointerup.call(event);
                },
                onpointercancel: move |event: Event<PointerData>| {
                    zooming.lift(event.pointer_id());
                    drag.onpointercancel.call(event);
                },
            }
            {ring_overlay()}
        }
    }
}

/// The strip under the stage: one roving tab stop for the whole thing, and a
/// click or an arrow moves the picture without taking the focus off the strip.
pub(super) fn lightbox_thumbnails(
    stage: Stage,
    items: &[LightboxItem],
    thumbnail_label: &'static str,
) -> Vec<Element> {
    let Stage {
        base_id,
        mut focus_next,
        current,
        last,
        ..
    } = stage;
    let count = items.len();

    items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_current = i == current;
            let thumbnail_states: Input<States> = states().with("current", is_current).into();
            let src = item
                .thumbnail_src
                .clone()
                .unwrap_or_else(|| item.src.clone());
            let image_states: Input<States> = states().with(SVG_FIT, svg_fit(&src)).into();
            let image_variables: Input<Variables> =
                svg_fit_variables(variables(), &src, "cover").into();
            rsx! {
                Box {
                    component: "button",
                    r#type: "button",
                    framework_sx: &LIGHTBOX_THUMBNAIL_SX,
                    "data-slot": LightboxPart::Thumbnail.slot(),
                    states: thumbnail_states,
                    id: thumbnail_id(&base_id(), i),
                    aria_label: fill(thumbnail_label, &[("n", &(i + 1)), ("m", &count)]),
                    aria_current: is_current.then(|| "true".to_string()),
                    // Roving: one tab stop for the strip.
                    tabindex: if is_current { "0" } else { "-1" },
                    onclick: move |_| stage.go(i, false),
                    onkeydown: move |event: Event<KeyboardData>| {
                        if has_shortcut_modifier(&event) {
                            return;
                        }
                        let target = match logical_key(&event) {
                            Key::ArrowLeft => i.saturating_sub(1),
                            Key::ArrowRight => (i + 1).min(last),
                            Key::Home => 0,
                            Key::End => last,
                            _ => return,
                        };
                        event.prevent_default();
                        stage.go(target, false);
                        focus_next.set(Some(thumbnail_id(&base_id(), target)));
                    },
                    Box {
                        component: "img",
                        framework_sx: &LIGHTBOX_THUMBNAIL_IMAGE_SX,
                        states: image_states,
                        variables: image_variables,
                        src,
                        alt: "",
                        draggable: "false",
                    }
                }
            }
        })
        .collect()
}
