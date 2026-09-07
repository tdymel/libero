use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        Box, Carousel, CarouselJump, Dialog, Input, States, Variables,
        common::{
            inset_focus_ring_sx, ring_overlay, ring_overlay_sx, states, use_name_warning, variables,
        },
    },
    hooks::{
        Drag, DragMove, DragOptions, DragPoint, DragStart, ElementHandle, LightboxItem,
        LightboxOpening, LightboxOptions, drag_handle_sx, id_selector, use_drag, use_element,
        use_id, use_modal_close, use_theme,
    },
    platform::{Dimensions, ElementApi},
    sx::{REDUCED_MOTION, StaticSx, sx},
    theme::{
        CarouselDefaults, CssVar, LIGHTBOX_STAGE_HEIGHT, LIGHTBOX_THUMBNAIL_SIZE,
        LIGHTBOX_THUMBNAILS_GAP, LIGHTBOX_WIDTH, Size, SizeCss,
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

/// On a phone the viewer is the whole screen: it has none to spare for a
/// margin round a picture. Either way up - `xs` alone (576px) caught every
/// phone held upright and none held sideways, which are 640 to 932px wide but
/// never more than 480px tall.
fn phone() -> String {
    format!(
        "(width < {}), (height < 30rem)",
        Size::Sm.breakpoint_value()
    )
}

/// The dialog's padding on one side of a phone: the usual `sm`, plus whatever
/// a notch or a home indicator takes there.
fn safe_padding(side: &str) -> String {
    format!(
        "calc({} + env(safe-area-inset-{side}, 0px))",
        SizeCss::SPACING.value(Size::Sm)
    )
}

// On a phone the dialog is the screen: fixed to the whole viewport, no
// margin, radius or shadow, and a column in which the stage takes whatever
// the close button, caption and thumbnails leave.
static LIGHTBOX_DIALOG_SX: StaticSx = StaticSx::new(|| {
    sx().width("100%")
        .max_width(LIGHTBOX_WIDTH.value())
        .padding("sm")
        .media(
            phone(),
            sx().position("fixed")
                .inset("0")
                .max_width("none")
                .margin("0")
                .border_radius("0")
                .box_shadow("none")
                .display("flex")
                .flex_direction("column")
                .padding_top(safe_padding("top"))
                .padding_right(safe_padding("right"))
                .padding_bottom(safe_padding("bottom"))
                .padding_left(safe_padding("left")),
        )
});

// Everything under the dialog's header: the stage, the caption, the strip.
static LIGHTBOX_BODY_SX: StaticSx = StaticSx::new(|| {
    sx().media(
        phone(),
        sx().flex("1")
            .min_height("0")
            .display("flex")
            .flex_direction("column"),
    )
});

// On a phone the pictures also reach past the dialog's `sm` padding, to the
// safe area's edge, and the stage is a size container: a frame is as tall as
// the room left, `100cqh`, where elsewhere it is the theme's stage height.
static LIGHTBOX_STAGE_SX: StaticSx = StaticSx::new(|| {
    let bleed = format!("calc(-1 * {})", SizeCss::SPACING.value(Size::Sm));
    sx().media(
        phone(),
        sx().margin_left(bleed.clone())
            .margin_right(bleed)
            .flex("1")
            .min_height("0")
            .container_type("size"),
    )
});

// Every slide is this one box, so a picture is fitted into the stage rather
// than sizing it. The ring sits here, inset, because the picture itself is
// scaled while zoomed and an outline on it would be scaled and clipped with it.
// Drawn by the overlay after the picture, which is what takes the focus.
static LIGHTBOX_FRAME_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative")
        .height(LIGHTBOX_STAGE_HEIGHT.value())
        .media(phone(), sx().height("100cqh"))
        .overflow("hidden")
        .selector("& > [data-ring]", ring_overlay_sx())
        .selector(
            "& > :focus-visible ~ [data-ring]",
            inset_focus_ring_sx("-2px"),
        )
});

static LIGHTBOX_IMAGE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .height("100%")
        // Shrinks a large picture to the stage and leaves a small one at its
        // own size: an upscaled thumbnail only shows its pixels.
        .object_fit("scale-down")
        // Replaces `Box`'s own ring: the frame draws this one.
        .focus_visible(sx().outline("none"))
        .transform(LIGHTBOX_TRANSFORM.value_or("none"))
        .transition("transform 150ms ease")
        .media(REDUCED_MOTION, sx().transition("none"))
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
        .focus_visible(inset_focus_ring_sx("-2px"))
});

static LIGHTBOX_THUMBNAIL_IMAGE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .height("100%")
        .object_fit("cover")
});

/// What a zoom is bounded by: the frame, and the picture as it is shown in it.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Fit {
    frame: Dimensions,
    picture: Dimensions,
}

impl Fit {
    /// The picture as `object-fit: scale-down` shows it: shrunk to fit the
    /// frame, never grown past its natural size. Without a natural size - not
    /// decoded yet, or a renderer that cannot say - it is taken to fill the
    /// frame, which only makes the pan bounds generous.
    fn new(frame: Dimensions, natural: Option<Dimensions>) -> Self {
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
    /// `(picture * s - frame) / 2` on each side, and that is how far it may
    /// move. A picture still smaller than the frame does not move at all.
    fn clamped(self, fit: Fit) -> Self {
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
    fn scaled(self, scale: f64, point: DragPoint, fit: Fit) -> Self {
        let ratio = scale / self.scale;
        Self {
            scale,
            x: point.x * (1.0 - ratio) + self.x * ratio,
            y: point.y * (1.0 - ratio) + self.y * ratio,
            ..self
        }
        .clamped(fit)
    }

    fn panned(self, dx: f64, dy: f64, fit: Fit) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            ..self
        }
        .clamped(fit)
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
/// centre. The frame and the picture's natural size are read first - for the
/// pan bounds and for where the cursor sits - so the zoom lands once they do.
/// Where a second `open_with` leaves the viewer: on its index, clamped to its
/// gallery, and fitted. A zoom is keyed by index only, so kept, it would carry
/// onto whichever new picture lands at the same index.
fn reopened(zoom: Zoom, index: usize, count: usize) -> (usize, Zoom) {
    let zoom = match zoom.is_zoomed() {
        true => Zoom::fitted(usize::MAX),
        false => zoom,
    };
    (index.min(count.saturating_sub(1)), zoom)
}

fn zoom_about(
    stage: ElementHandle,
    picture: ElementHandle,
    mut zoom: Signal<Zoom>,
    mut fit: Signal<Option<Fit>>,
    index: usize,
    client: Option<DragPoint>,
    next: impl Fn(f64) -> f64 + 'static,
) {
    let Ok(frame) = stage.query_selector(&frame_selector(index)) else {
        return;
    };
    let (dimensions, origin, natural) = (
        frame.dimensions(),
        frame.client_offset(),
        picture.natural_size(),
    );
    spawn(async move {
        let (Ok(dimensions), Ok((left, top))) = (dimensions.await, origin.await) else {
            return;
        };
        let bounds = Fit::new(dimensions, natural.await.ok());
        fit.set(Some(bounds));
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
            true => from.scaled(scale, point, bounds),
            false => Zoom::fitted(index),
        });
    });
}

/// The zoom and the gesture, and the two scales that move them. One `Copy`
/// argument, so the drag, the keys, the wheel and the double-click all read
/// the same state.
#[derive(Clone, Copy)]
struct Zooming {
    index: Signal<usize>,
    zoom: Signal<Zoom>,
    /// Measured whenever a zoom starts, so a zoomed picture always has bounds
    /// to pan within - and a key can decide synchronously whether it pans.
    fit: Signal<Option<Fit>>,
    gesture: Signal<Option<Gesture>>,
    max_zoom: f64,
}

impl Zooming {
    /// The zoom held for the picture showing, or a fresh fit when the held one
    /// belongs to another index.
    fn held(self) -> Zoom {
        match *self.zoom.peek() {
            held if held.index == *self.index.peek() => held,
            _ => Zoom::fitted(*self.index.peek()),
        }
    }

    /// Double-click and `z` toggle between fit and [`TOGGLE_ZOOM`].
    fn toggle_scale(self, scale: f64) -> f64 {
        match scale > 1.0 {
            true => 1.0,
            false => TOGGLE_ZOOM.min(self.max_zoom),
        }
    }
}

/// Everything a picture and a thumbnail read: the zoom, the drag, the ids and
/// the mover.
#[derive(Clone, Copy)]
struct Stage {
    zooming: Zooming,
    drag: Drag,
    stage: ElementHandle,
    base_id: Signal<String>,
    /// The element a move sends focus to, read by the effect that focuses it.
    focus_next: Signal<Option<String>>,
    current: usize,
    last: usize,
    active: Zoom,
    zoomable: bool,
    swipe: bool,
    preload: usize,
    /// How far a swipe-to-close has travelled down, while one is in progress.
    swiping: Option<f64>,
}

impl Stage {
    /// Moves to picture `target`, and focus with it when focus was on the
    /// picture - otherwise the keyboard is stranded on a slide scrolled away.
    fn go(self, target: usize, refocus: bool) {
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

/// The pan and the swipe-to-close, on the picture showing. Not on the stage:
/// pointer capture retargets the click that follows a press
/// ([[codebase/components/scroller]]), so a double-click on a zoomed picture
/// would land on the stage, not on the picture's own `ondoubleclick`.
fn use_lightbox_drag(zooming: Zooming, capture: ElementHandle, close: Callback<()>) -> Drag {
    let Zooming {
        mut zoom,
        fit,
        mut gesture,
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
                None => {}
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
fn lightbox_slide(
    stage: Stage,
    i: usize,
    item: &LightboxItem,
    image: ElementHandle,
    described: Option<String>,
) -> Element {
    let Stage {
        zooming,
        drag,
        stage: root,
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
        max_zoom,
    } = zooming;

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
    let image_variables: Input<Variables> = variables().with(LIGHTBOX_TRANSFORM, transform).into();
    let loading = match i.abs_diff(current) <= preload {
        true => "eager",
        false => "lazy",
    };

    // Keyed by picture, so a gallery swapped in by a second `open_with`
    // gets new `<img>`s: Chromium fetches a new `src` at once on an
    // element that has loaded before, whatever `loading` says (todo 227).
    // `Carousel` keys its slide wrappers by position, so the strip and its
    // scroll stay put and only what is inside a wrapper is replaced.
    rsx! {
        Box { key: "{item.src}", framework_sx: &LIGHTBOX_FRAME_SX, "data-lightbox-frame": i,
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
                        Key::ArrowLeft => stage.go(i.saturating_sub(1), true),
                        Key::ArrowRight => stage.go(i + 1, true),
                        Key::Home => stage.go(0, true),
                        Key::End => stage.go(last, true),
                        Key::Character(ref c)
                            if c.eq_ignore_ascii_case("z")
                                && !event.modifiers().ctrl()
                                && !event.modifiers().meta()
                                && !event.modifiers().alt() =>
                        {
                            zoom_about(root, image, zoom, fit, i, None, move |scale| {
                                zooming.toggle_scale(scale)
                            });
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
                        root,
                        image,
                        zoom,
                        fit,
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
                        root,
                        image,
                        zoom,
                        fit,
                        i,
                        Some(DragPoint { x: client.x, y: client.y }),
                        move |scale| zooming.toggle_scale(scale),
                    );
                },
                onmounted: image.mount(),
                onpointermove: drag.onpointermove,
                onpointerup: drag.onpointerup,
                onpointercancel: drag.onpointercancel,
                onpointerdown: move |event: Event<PointerData>| {
                    if i != *index.peek() {
                        return;
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
            }
            {ring_overlay()}
        }
    }
}

/// The strip under the stage: one roving tab stop for the whole thing, and a
/// click or an arrow moves the picture without taking the focus off the strip.
fn lightbox_thumbnails(
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
            rsx! {
                Box {
                    component: "button",
                    r#type: "button",
                    framework_sx: &LIGHTBOX_THUMBNAIL_SX,
                    states: thumbnail_states,
                    id: thumbnail_id(&base_id(), i),
                    aria_label: CarouselDefaults::format_label(thumbnail_label, i, count),
                    aria_current: is_current.then(|| "true".to_string()),
                    // Roving: one tab stop for the strip.
                    tabindex: if is_current { "0" } else { "-1" },
                    onclick: move |_| stage.go(i, false),
                    onkeydown: move |event: Event<KeyboardData>| {
                        let target = match event.key() {
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
                        src: item.thumbnail_src.clone().unwrap_or_else(|| item.src.clone()),
                        alt: "",
                        draggable: "false",
                    }
                }
            }
        })
        .collect()
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
    let mut zoom = use_signal(|| Zoom::fitted(usize::MAX));
    // The opening `index` and `zoom` were last reset for, and whether a
    // picture held focus when a new one arrived.
    let settled = use_hook(|| Rc::new(RefCell::new((opening.clone(), false))));
    // A second `open_with` while this one shows replaces the gallery in place,
    // but the effect below only resets `index` after this render. Until then
    // the new gallery is drawn at its own index: the new frames are keyed
    // `<img>`s, and one created `eager` around the old index fetches at once
    // and cannot be taken back (todo 227).
    let pending = settled.borrow().0 != opening;
    // The swap's own move is instant: a smooth scroll back from the old index
    // would pass over new pictures that are not shown, and fetch them. Counted
    // once per gallery, not once per pending render: a second count for the
    // same gallery would still be unread when the user's next move arrives,
    // and that move would jump as well.
    let jump = use_context_provider(CarouselJump::default);
    let counted = use_hook(|| Rc::new(RefCell::new(opening.clone())));
    if pending && *counted.borrow() != opening {
        *counted.borrow_mut() = opening.clone();
        jump.swapped();
    }
    if pending {
        // The last render in which the old pictures are still in the document.
        // Their `<img>`s are about to be replaced, so focus on one would drop
        // to the page.
        let focused = stage
            .query_selector("[data-lightbox-frame] img:focus")
            .is_ok();
        settled.borrow_mut().1 = focused;
    }
    // A zoom is keyed by index only, so it would carry onto whichever new
    // picture lands at the same index.
    use_effect(use_reactive!(|opening| {
        let (target, fitted) = reopened(*zoom.peek(), opening.index, opening.items.len());
        if *index.peek() != target {
            index.set(target);
        }
        if *zoom.peek() != fitted {
            zoom.set(fitted);
        }
        let refocus = std::mem::replace(&mut *settled.borrow_mut(), (opening, false)).1;
        if refocus
            && let Ok(image) = stage.query_selector(&id_selector(&image_id(&base_id(), target)))
        {
            let _ = image.focus();
        }
    }));

    let zooming = Zooming {
        index,
        zoom,
        fit: use_signal(|| None::<Fit>),
        gesture: use_signal(|| None::<Gesture>),
        max_zoom: options.max_zoom.unwrap_or(theme.lightbox.max_zoom).max(1.0),
    };

    // Read either way, so this render stays subscribed to the reset.
    let current = match index() {
        _ if pending => opening.index.min(last),
        settled => settled,
    };
    let active = match zoom() {
        held if held.index == current && !pending => held,
        _ => Zoom::fitted(current),
    };

    // The element a move sends focus to. Focused from an effect, not from the
    // handler: until the render that moves the carousels, the target's slide
    // is still `inert`, and `focus()` on it does nothing.
    let mut focus_next = use_signal(|| None::<String>);
    use_effect(move || {
        if let Some(id) = focus_next() {
            focus_next.set(None);
            if let Ok(element) = stage.query_selector(&id_selector(&id)) {
                let _ = element.focus();
            }
        }
    });

    // One handle per picture, and the drag captures on the one showing. Grown
    // in render, where a signal may be created, because a later opening can
    // bring more pictures; a `RefCell`, not a signal, so growing it re-renders
    // nothing.
    let pictures = use_hook(|| Rc::new(RefCell::new(Vec::<ElementHandle>::new())));
    {
        let mut pictures = pictures.borrow_mut();
        while pictures.len() < count {
            pictures.push(ElementHandle::new());
        }
    }
    let picture = |i: usize| pictures.borrow()[i];

    let drag = use_lightbox_drag(
        zooming,
        if count > 0 { picture(current) } else { stage },
        close,
    );

    let gesture = zooming.gesture;
    let swiping = match gesture() {
        Some(Gesture::Swipe { delta }) => Some(delta.y.max(0.0)),
        _ => None,
    };
    let stage_parts = Stage {
        zooming,
        drag,
        stage,
        base_id,
        focus_next,
        current,
        last,
        active,
        zoomable: options.zoom,
        swipe: options.close_on_swipe_down,
        preload: options.preload,
        swiping,
    };

    let caption = items
        .get(current)
        .and_then(|item| item.caption.clone())
        .filter(|_| options.captions);
    let described = caption.as_ref().map(|_| caption_id());

    let slides: Vec<Element> = (0..count)
        .map(|i| lightbox_slide(stage_parts, i, &items[i], picture(i), described.clone()))
        .collect();
    use_name_warning(
        options.aria_label.is_some(),
        "Lightbox: no `aria_label`, falling back to the theme's. A dialog needs a name of its own to be told apart.",
    );
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
    let thumbnails = lightbox_thumbnails(stage_parts, &items, theme.lightbox.thumbnail_label);
    let show_thumbnails = options.thumbnails && count > 1;

    rsx! {
        Dialog {
            aria_label: label.clone(),
            close_label: theme.lightbox.close_label,
            sx: &LIGHTBOX_DIALOG_SX,
            Box {
                framework_sx: &LIGHTBOX_BODY_SX,
                onmounted: stage.mount(),
                Box { framework_sx: &LIGHTBOX_STAGE_SX, {stage_body} }
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

    /// A picture exactly the frame's size: the bounds before natural sizes.
    const FILLS: Fit = Fit {
        frame: FRAME,
        picture: FRAME,
    };

    fn size(width: f64, height: f64) -> Dimensions {
        Dimensions { width, height }
    }

    #[test]
    fn a_small_picture_is_shown_at_its_natural_size() {
        let fit = Fit::new(FRAME, Some(size(200.0, 100.0)));

        assert_eq!(fit.picture, size(200.0, 100.0));
    }

    #[test]
    fn a_large_picture_is_scaled_down_into_the_frame() {
        let fit = Fit::new(FRAME, Some(size(3200.0, 1200.0)));

        assert_eq!(fit.picture, size(800.0, 300.0));
    }

    /// At 3x a 200px picture is still 600px, inside the 800px frame.
    /// Todo 252, for `3203d7fd`: a second `open_with` drops the zoom, even
    /// onto the index the zoom belonged to. Only this decision is testable
    /// here: zooming needs the frame measured, which SSR cannot do. The
    /// effect that calls it was checked in Chromium when it landed.
    #[test]
    fn a_second_open_drops_the_zoom_and_clamps_the_index() {
        let zoomed = Zoom {
            index: 2,
            scale: 2.0,
            x: 10.0,
            y: 0.0,
        };
        assert_eq!(reopened(zoomed, 2, 4), (2, Zoom::fitted(usize::MAX)));
        assert_eq!(reopened(zoomed, 9, 4), (3, Zoom::fitted(usize::MAX)));
        // A fitted zoom is left as it is, so reopening writes nothing.
        assert_eq!(reopened(Zoom::fitted(1), 0, 4), (0, Zoom::fitted(1)));
        assert_eq!(reopened(Zoom::fitted(1), 0, 0).0, 0);
    }

    #[test]
    fn a_zoomed_picture_smaller_than_the_frame_cannot_move() {
        let zoomed = Zoom {
            scale: 3.0,
            ..Zoom::fitted(0)
        };
        let fit = Fit::new(FRAME, Some(size(200.0, 100.0)));

        assert_eq!(zoomed.panned(500.0, 500.0, fit), zoomed);
    }

    /// Scaled down to 800x300, then doubled to 1600x600: it overhangs 400px
    /// sideways and nothing vertically.
    #[test]
    fn a_pan_is_bounded_by_the_picture_not_the_frame() {
        let zoomed = Zoom {
            scale: 2.0,
            ..Zoom::fitted(0)
        };
        let fit = Fit::new(FRAME, Some(size(3200.0, 1200.0)));

        let far = zoomed.panned(10_000.0, 10_000.0, fit);

        assert_eq!((far.x, far.y), (400.0, 0.0));
    }

    #[test]
    fn without_a_natural_size_the_picture_fills_the_frame() {
        assert_eq!(Fit::new(FRAME, None), FILLS);
    }

    #[test]
    fn a_fitted_picture_cannot_move() {
        let fitted = Zoom::fitted(0);

        assert_eq!(fitted.panned(40.0, -40.0, FILLS), fitted);
    }

    /// At scale 2 an 800px picture overhangs 400px each side, and no further.
    #[test]
    fn a_pan_stops_where_the_picture_would_leave_the_frame() {
        let zoomed = Zoom {
            scale: 2.0,
            ..Zoom::fitted(0)
        };

        let far = zoomed.panned(10_000.0, -10_000.0, FILLS);

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

        assert_eq!(at_edge.panned(PAN_STEP, 0.0, FILLS), at_edge);
        assert_ne!(at_edge.panned(-PAN_STEP, 0.0, FILLS), at_edge);
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

        assert_eq!(near.panned(PAN_STEP, 0.0, FILLS).x, 400.0);
    }

    #[test]
    fn zooming_about_the_centre_does_not_move_the_picture() {
        let zoomed = Zoom::fitted(0).scaled(2.0, DragPoint { x: 0.0, y: 0.0 }, FILLS);

        assert_eq!((zoomed.scale, zoomed.x, zoomed.y), (2.0, 0.0, 0.0));
    }

    /// The spot under the cursor stays under it: a point 100px right of centre
    /// on a fitted picture is still 100px right of centre at scale 2.
    #[test]
    fn zooming_about_a_point_keeps_that_point_still() {
        let point = DragPoint { x: 100.0, y: -50.0 };
        let zoomed = Zoom::fitted(0).scaled(2.0, point, FILLS);

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
