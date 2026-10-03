use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::crop::{CropRect, CropShape, Grip};
use crate::{
    components::{
        accessibility::visually_hidden_sx,
        buttons::ActionIcon,
        common::{
            Glyph, HtmlTag, Input, Part, States, Variables, base_props, disabled_look_sx,
            focus_ring_sx, has_shortcut_modifier, parts_enum, variables,
        },
        form::{Slider, SliderChangeEvent},
        layout::use_box,
    },
    context::IconSlot,
    hooks::{
        DragMove, DragOptions, DragPoint, DragStart, use_drag, use_element, use_id,
        use_local_state, use_localization,
    },
    localization::fill,
    platform::{self, ElementApi, when_laid_out},
    sx::{StaticSx, sx},
    theme::{CssVar, PAPER_BACKGROUND},
    utils::warn,
};

/// The crop box's edges, as fractions of the image.
const CROP_X: CssVar = CssVar::new("--lsx-image-cropper-x");
const CROP_Y: CssVar = CssVar::new("--lsx-image-cropper-y");
const CROP_WIDTH: CssVar = CssVar::new("--lsx-image-cropper-width");
const CROP_HEIGHT: CssVar = CssVar::new("--lsx-image-cropper-height");
/// Pan mode: the image's scale and offset, the offset in fractions of the cropper.
const IMAGE_SCALE: CssVar = CssVar::new("--lsx-image-cropper-scale");
const IMAGE_X: CssVar = CssVar::new("--lsx-image-cropper-image-x");
const IMAGE_Y: CssVar = CssVar::new("--lsx-image-cropper-image-y");

/// Arrow keys move this far, Shift+arrow ten times as far.
const KEY_STEP: f64 = 0.01;
/// Pan mode's + and - zoom by this factor, the Larger and Smaller buttons scale by it.
const ZOOM_STEP: f64 = 1.1;
/// A move button's step, Shift+arrow's half.
const NUDGE_STEP: f64 = 0.05;
/// The smallest box side, as a fraction of the image.
const MIN_SIZE: f64 = 0.05;
/// A handle's hit area (WCAG 2.5.8), around a smaller visible square.
const HANDLE_HIT: &str = "24px";
/// A finger's hit area (WCAG 2.5.5, todo 1387).
const HANDLE_HIT_COARSE: &str = "44px";
const HANDLE_DOT: &str = "12px";
const COARSE: &str = "(pointer: coarse)";

parts_enum! {
    /// [`ImageCropper`]'s inner parts, for its `parts` prop.
    pub enum ImageCropperPart {
        Image = "image" => "& > [data-slot='image']",
        /// The dimmed image outside the crop; a drag on it moves the box.
        Mask = "mask" => "& > [data-slot='mask'] > *",
        /// The crop box, a tab stop of its own.
        Box = "box" => "& > [data-slot='box']",
        /// Over the box, taking its drags and holding the handles: a slider's
        /// children are hidden from assistive tech.
        Frame = "frame" => "& > [data-slot='frame']",
        /// One of the eight resize handles; the corners are tab stops.
        Handle = "handle" => "& > [data-slot='frame'] > [data-slot='handle']",
        /// Pan mode: the bar under the image holding the zoom slider.
        Zoom = "zoom" => "& > [data-slot='zoom']",
        /// The buttons under the image that move and resize the box without a drag.
        Controls = "controls" => "& > [data-slot='controls']",
    }
}

/// The touches down on the box, for a two-finger pinch.
#[derive(Clone, Copy, Default)]
struct Touches {
    points: [Option<(i32, DragPoint)>; 2],
    /// The box, the fingers' spread and midpoint when the second one went down.
    pinch: Option<(CropRect, f64, DragPoint)>,
    /// Set by a pinch: the first finger's drag moves nothing until it lifts.
    pinched: bool,
}

impl Touches {
    fn spread(&self) -> Option<f64> {
        match self.points {
            [Some((_, a)), Some((_, b))] => Some((a.x - b.x).hypot(a.y - b.y)),
            _ => None,
        }
    }

    fn midpoint(&self) -> Option<DragPoint> {
        match self.points {
            [Some((_, a)), Some((_, b))] => Some(DragPoint {
                x: (a.x + b.x) / 2.0,
                y: (a.y + b.y) / 2.0,
            }),
            _ => None,
        }
    }

    /// Moves a known touch, else takes a free slot; `false` for neither.
    fn put(&mut self, id: i32, point: DragPoint) -> bool {
        let slot = match self
            .points
            .iter()
            .position(|p| p.is_some_and(|(at, _)| at == id))
        {
            Some(slot) => slot,
            None => match self.points.iter().position(Option::is_none) {
                Some(slot) => slot,
                None => return false,
            },
        };
        self.points[slot] = Some((id, point));
        true
    }

    fn knows(&self, id: i32) -> bool {
        self.points
            .iter()
            .any(|p| p.is_some_and(|(at, _)| at == id))
    }

    fn lift(&mut self, id: i32) {
        for point in &mut self.points {
            if point.is_some_and(|(at, _)| at == id) {
                *point = None;
            }
        }
        self.pinch = None;
    }
}

fn client(event: &Event<PointerData>) -> DragPoint {
    let at = event.client_coordinates();
    DragPoint { x: at.x, y: at.y }
}

fn placed() -> crate::sx::Sx {
    let at = |var: CssVar| format!("calc({} * 100%)", var.value_or("0"));
    sx().position("absolute")
        .left(at(CROP_X))
        .top(at(CROP_Y))
        .width(at(CROP_WIDTH))
        .height(at(CROP_HEIGHT))
}

static IMAGE_CROPPER_SX: StaticSx = StaticSx::new(|| {
    let handle = |slot: &str, left: &str, top: &str, cursor: &str| {
        (
            format!("& > [data-slot='frame'] > [data-grip='{slot}']"),
            sx().left(left.to_string())
                .top(top.to_string())
                .cursor(cursor.to_string()),
        )
    };
    // A grid: the image's cell holds the box's parts, the rows under it the bars.
    let base = sx()
        .position("relative")
        .display("inline-grid")
        // The box maps onto this, so it must hug the image, in a flex column too.
        .width("fit-content")
        .height("fit-content")
        .max_width("100%")
        .line_height("0")
        .user_select("none")
        .selector(
            "& > [data-slot='image']",
            sx().display("block")
                .grid_area("1 / 1")
                // Not stretched: Blitz sized the row by the picture at the container's width (2116).
                .align_self("start")
                .justify_self("start")
                .max_width("100%")
                .height("auto")
                .pointer_events("none"),
        )
        .selector(
            "& > [data-slot='mask'], & > [data-slot='box'], & > [data-slot='frame']",
            // All four lines: an absolute item's `auto` line is the padding edge.
            sx().grid_area("1 / 1 / 2 / 2"),
        )
        // No width of its own, so the buttons wrap under a narrow image rather than widen its cell.
        .selector(
            "& > [data-slot='controls']",
            sx().grid_area("2 / 1")
                .width("0")
                .min_width("100%")
                .display("flex")
                .flex_wrap("wrap")
                .justify_content("center")
                .padding_block("xs")
                .line_height("normal"),
        )
        // Clipped apart from the box, so a handle on the image's edge keeps
        // its whole hit area. Under the frame, it takes a drag or pinch begun on the image (2113).
        .selector(
            "& > [data-slot='mask']",
            sx().position("absolute")
                .top("0")
                .left("0")
                .right("0")
                .bottom("0")
                .overflow("hidden")
                .cursor("move")
                .touch_action("none"),
        )
        .selector("& > [data-slot='mask'] > *", sx().pointer_events("none"))
        .selector(
            "& > [data-slot='mask'] > *",
            // Forced colours drop a box-shadow unless the element opts out (todo 1278).
            placed()
                .box_shadow("0 0 0 9999px rgba(0, 0, 0, 0.55)")
                .with("forced-color-adjust", "none"),
        )
        .selector(
            "& > [data-slot='box']",
            placed().outline("1px solid rgba(255, 255, 255, 0.9)"),
        )
        .selector("& > [data-slot='box']:focus-visible", focus_ring_sx())
        .selector(
            "& > [data-slot='frame']",
            placed().cursor("move").touch_action("none"),
        )
        .selector(
            "& > [data-slot='frame'] > [data-slot='handle']",
            sx().position("absolute")
                .width(HANDLE_HIT)
                .height(HANDLE_HIT)
                .transform("translate(-50%, -50%)")
                .display("flex")
                .align_items("center")
                .justify_content("center")
                .media(
                    COARSE,
                    sx().width(HANDLE_HIT_COARSE).height(HANDLE_HIT_COARSE),
                ),
        )
        // Over the handles between them, and at least a dot's size: a box at
        // `min_size` stays movable when the handles' hit areas meet (1264).
        .selector("& > [data-slot='frame'] > [data-slot='move']", {
            let side = |hit: &str| format!("max(calc(100% - {hit}), min(100%, {HANDLE_DOT}))");
            sx().position("absolute")
                .left("50%")
                .top("50%")
                .width(side(HANDLE_HIT))
                .height(side(HANDLE_HIT))
                .transform("translate(-50%, -50%)")
                .media(
                    COARSE,
                    sx().width(side(HANDLE_HIT_COARSE))
                        .height(side(HANDLE_HIT_COARSE)),
                )
        })
        .selector(
            "& > [data-slot='frame'] > [data-slot='handle'] > span",
            sx().display("block")
                .width(HANDLE_DOT)
                .height(HANDLE_DOT)
                .box_sizing("border-box")
                .background("#ffffff")
                .border("1px solid rgba(0, 0, 0, 0.6)"),
        )
        .selector(
            "& > [data-slot='frame'] > [data-slot='handle']:focus-visible",
            focus_ring_sx(),
        )
        .when(
            "circle",
            sx().selector("& > [data-slot='mask'] > *", sx().border_radius("50%")),
        )
        // The frame holds still; the image moves under it, and the whole cropper takes touches.
        // A grid row under the image holds the zoom bar, and the box's parts place in the
        // image's cell, so the bar never covers the box (1613).
        .when(
            "pan",
            sx().overflow("hidden")
                .selector(
                    "& > [data-slot='image']",
                    sx().with("transform-origin", "0 0").transform(format!(
                        "translate(calc({} * 100%), calc({} * 100%)) scale({})",
                        IMAGE_X.value_or("0"),
                        IMAGE_Y.value_or("0"),
                        IMAGE_SCALE.value_or("1")
                    )),
                )
                .selector(
                    "& > [data-slot='frame']",
                    sx().left("0")
                        .top("0")
                        .width("100%")
                        .height("100%")
                        .cursor("grab"),
                )
                // A single-pointer zoom (WCAG 2.5.1), on the page's paper over the zoomed image.
                .selector(
                    "& > [data-slot='zoom']",
                    sx().grid_area("2 / 1")
                        .position("relative")
                        .padding_inline("sm")
                        .padding_block("xs")
                        .line_height("normal")
                        .background(PAPER_BACKGROUND.value()),
                )
                .selector(
                    "& > [data-slot='controls']",
                    sx().grid_area("3 / 1").background(PAPER_BACKGROUND.value()),
                ),
        )
        // `controls: false`: out of sight, back while focus is inside, so a keyboard keeps them.
        .when(
            "hidden-controls",
            sx().selector(
                "& > [data-slot='controls']:not(:focus-within), & > [data-slot='zoom']:not(:focus-within)",
                // The bar's own `min-width: 100%` would outgrow the 1px.
                visually_hidden_sx().min_width("0"),
            ),
        )
        .when(
            "disabled",
            disabled_look_sx("not-allowed")
                .selector(
                    "& > [data-slot='mask'], & > [data-slot='frame'], & > [data-slot='frame'] > [data-slot='handle']",
                    sx().cursor("not-allowed"),
                )
                // A disabled cropper lets the page scroll from its image.
                .selector("& > [data-slot='mask']", sx().touch_action("auto")),
        );

    [
        handle("nw", "0", "0", "nwse-resize"),
        handle("n", "50%", "0", "ns-resize"),
        handle("ne", "100%", "0", "nesw-resize"),
        handle("e", "100%", "50%", "ew-resize"),
        handle("se", "100%", "100%", "nwse-resize"),
        handle("s", "50%", "100%", "ns-resize"),
        handle("sw", "0", "100%", "nesw-resize"),
        handle("w", "0", "50%", "ew-resize"),
    ]
    .into_iter()
    .fold(base, |acc, (selector, rule)| acc.selector(selector, rule))
});

base_props! {
    parts(ImageCropperPart);
    pub struct ImageCropperProps {
        /// The image: any URL, a `data:` URL included.
        #[props(into)]
        src: String,
        #[props(into)]
        alt: String,
        /// Controlled: pair it with `onchange`. Unset starts centred at 80% of
        /// the largest box `aspect` allows, of the whole image without one.
        #[props(default)]
        value: Option<CropRect>,
        /// Fires on every move of the box, a drag or a key.
        #[props(default)]
        onchange: Option<EventHandler<CropRect>>,
        /// Locks the box's width over height, in image pixels: `1.0` is square.
        #[props(default)]
        aspect: Option<f64>,
        /// Masks the image outside the box with a rectangle or an ellipse.
        #[props(default)]
        shape: CropShape,
        /// Holds the box still and moves the image under it: a drag pans the
        /// image, a pinch, the wheel or the + and - keys zoom it. No resize handles.
        #[props(default)]
        pan: bool,
        /// The smallest side, a fraction of the image's. Default 0.05.
        #[props(default)]
        min_size: Option<f64>,
        /// `false` hides the buttons under the image, and with `pan` the zoom bar, from
        /// sight only: they show again while focus is inside them. The keys stay.
        #[props(default = true)]
        controls: bool,
        /// Draws the crop, takes no input.
        #[props(default)]
        disabled: bool,
        /// Names the crop box; `Localization.image_cropper.label` by default.
        #[props(default, into)]
        aria_label: Option<String>,
        /// Fires when `src` fails to load; the box is not drawn until `src` changes.
        #[props(default)]
        onerror: Option<EventHandler<()>>,
    }
}

/// A box with handles over an image, picking the part to keep. Drag the box, a
/// handle or the image around the box, pinch with two fingers, or use the arrow keys on the
/// box and its corners. With `pan`, the image moves and zooms under a still box
/// instead, as a phone's profile picture cropper does.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{CropRect, ImageCropper};
/// # fn app() -> Element {
/// let mut crop = use_signal(|| None::<CropRect>);
/// rsx! {
///     ImageCropper {
///         src: "/photo.jpg",
///         alt: "Holiday photo",
///         aspect: 1.0,
///         value: crop(),
///         onchange: move |rect| crop.set(Some(rect)),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/image-cropper>
#[component]
pub fn ImageCropper(props: ImageCropperProps) -> Element {
    let words = use_localization().image_cropper;
    let root = use_element();
    let image = use_element();
    // Pan mode's image cell, the cropper less the bars under it.
    let frame = use_element();
    // What the box maps onto: the image, or in pan mode its cell.
    let stage = if props.pan { frame } else { image };
    let box_element = use_element();
    let corners = [use_element(), use_element(), use_element(), use_element()];
    let keys_id = use_id();

    // The image's displayed width over its height, measured once it loads.
    let mut image_ratio = use_signal(|| None::<f64>);
    let mut held = use_signal(|| None::<CropRect>);
    let grip = use_local_state(|| Grip::Move);
    // The box and the image's client size where a drag started.
    let start = use_local_state(|| (CropRect::FULL, 0.0_f64, 0.0_f64));
    // The cropper's client top-left there, for a pinch's midpoint.
    let origin = use_local_state(|| (0.0_f64, 0.0_f64));
    let touches = use_local_state(Touches::default);

    let pan = props.pan;
    let interactive = !props.disabled && props.onchange.is_some();
    if props.value.is_some() && props.onchange.is_none() && !props.disabled {
        warn("ImageCropper: `value` without `onchange` can never change.");
    }
    let min = props.min_size.unwrap_or(MIN_SIZE).clamp(0.0, 1.0);
    // The pixel `aspect` as a ratio of fractions of this image.
    let ratio = props
        .aspect
        .filter(|aspect| aspect.is_finite() && *aspect > 0.0)
        .map(|aspect| aspect / image_ratio().unwrap_or(1.0));
    let shown = props
        .value
        .or(held())
        .unwrap_or_else(|| CropRect::starting(ratio));

    let onchange = props.onchange;
    let emit = use_callback(move |rect: CropRect| {
        held.set(Some(rect));
        if let Some(onchange) = &onchange {
            onchange.call(rect);
        }
    });

    // An unset `value` reports its starting box once the image's shape is known,
    // and refits it while no key or drag has moved it.
    let (aspect, unset) = (props.aspect, props.value.is_none());
    let mut fitted = use_signal(|| None::<CropRect>);
    let measure = use_callback(move |()| {
        let size = image.dimensions();
        spawn(async move {
            let Ok(size) = size.await else {
                return;
            };
            if size.width <= 0.0 || size.height <= 0.0 {
                return;
            }
            let measured = size.width / size.height;
            if *image_ratio.peek() != Some(measured) {
                image_ratio.set(Some(measured));
            }
            let current = *held.peek();
            if unset && (current.is_none() || current == *fitted.peek()) {
                let ratio = aspect
                    .filter(|aspect| aspect.is_finite() && *aspect > 0.0)
                    .map(|aspect| aspect / measured);
                let rect = CropRect::starting(ratio);
                fitted.set(Some(rect));
                if current != Some(rect) {
                    emit.call(rect);
                }
            }
        });
    });

    // A new picture or aspect starts over: a `src` still loading is fitted again on load.
    let mounted = use_hook(|| std::rc::Rc::new(std::cell::Cell::new(false)));
    use_effect(use_reactive((&props.src, &aspect), move |_| {
        if mounted.replace(true) {
            held.set(None);
            when_laid_out(move || measure.call(()));
        }
    }));

    // A `src` that failed to load: no box over the alt text until another `src`.
    let mut failed = use_signal(|| false);
    use_effect(use_reactive(&props.src, move |_| {
        if *failed.peek() {
            failed.set(false);
        }
    }));
    let onerror = props.onerror;
    let onerror = move |_| {
        warn("ImageCropper: `src` failed to load.");
        failed.set(true);
        if let Some(onerror) = &onerror {
            onerror.call(());
        }
    };

    // Blitz fires no `load`: measure once laid out as well.
    use_effect(move || {
        if image.is_mounted() {
            when_laid_out(move || measure.call(()));
        }
    });

    let focus_grip = use_callback(move |grabbed: Grip| {
        let element = match grabbed {
            Grip::NorthWest => corners[0],
            Grip::NorthEast => corners[1],
            Grip::SouthEast => corners[2],
            Grip::SouthWest => corners[3],
            _ => box_element,
        };
        let _ = element.focus();
    });

    // The latest move while the start is unmeasured: a slow WebView moves first (1230).
    let early = use_local_state(|| None::<DragMove>);
    let move_to = {
        let (start, grip) = (start.clone(), grip.clone());
        use_callback(move |event: DragMove| {
            let (from, width, height) = start.get();
            let delta = event.delta();
            let (dx, dy) = (delta.x / width, delta.y / height);
            let rect = match pan {
                true => from.panned(dx, dy),
                false => from.resized(grip.get(), dx, dy, ratio, min),
            };
            if rect != shown {
                emit.call(rect);
            }
        })
    };
    let onstart = {
        let (start, grip, early, origin) =
            (start.clone(), grip.clone(), early.clone(), origin.clone());
        use_callback(move |event: DragStart| {
            if !interactive {
                event.cancel.call(());
                return;
            }
            let (start, grabbed, early, origin) =
                (start.clone(), grip.get(), early.clone(), origin.clone());
            start.set((shown, 0.0, 0.0));
            early.set(None);
            // Started here, awaited in the task: under Blitz a read resolves
            // where it is called.
            let size = stage.dimensions();
            let offset = stage.client_offset();
            spawn(async move {
                if let Ok(at) = offset.await {
                    origin.set(at);
                }
                let Ok(size) = size.await else {
                    event.cancel.call(());
                    return;
                };
                if size.width <= 0.0 || size.height <= 0.0 {
                    event.cancel.call(());
                    return;
                }
                image_ratio.set(Some(size.width / size.height));
                start.set((shown, size.width, size.height));
                if let Some(moved) = early.get() {
                    move_to.call(moved);
                }
                // The drag cancels the pointerdown, and with it the focus.
                focus_grip.call(grabbed);
            });
        })
    };
    let onmove = {
        let start = start.clone();
        use_callback(move |event: DragMove| match start.get() {
            (_, width, _) if width <= 0.0 => early.set(Some(event)),
            _ => move_to.call(event),
        })
    };
    let drag = use_drag(DragOptions {
        capture: root,
        onstart,
        onmove,
        // Again at the end: Blitz moves focus off on the frame's own press.
        onend: {
            let (grip, touches) = (grip.clone(), touches.clone());
            Callback::new(move |()| {
                touches.set(Touches::default());
                focus_grip.call(grip.get());
            })
        },
    });

    let press = {
        let (grip, touches) = (grip.clone(), touches.clone());
        move |grabbed: Grip| {
            let (grip, touches) = (grip.clone(), touches.clone());
            move |event: Event<PointerData>| {
                event.stop_propagation();
                if event.pointer_type() == "touch" {
                    let mut now = touches.get();
                    if !now.put(event.pointer_id(), client(&event)) {
                        return;
                    }
                    // A second finger pinches the box, or the image, rather than grabbing it.
                    if let (Some(spread), Some(midpoint)) = (now.spread(), now.midpoint()) {
                        event.prevent_default();
                        if interactive && *drag.dragging.peek() {
                            now.pinch = Some((shown, spread, midpoint));
                            now.pinched = true;
                        }
                        touches.set(now);
                        return;
                    }
                    touches.set(now);
                }
                grip.set(grabbed);
                drag.onpointerdown.call(event);
            }
        }
    };

    // The root sees both fingers' moves: the first finger's by capture, the
    // second's bubbling up from its own implicit capture.
    let onpointermove = {
        let (touches, start, origin) = (touches.clone(), start.clone(), origin.clone());
        move |event: Event<PointerData>| {
            let mut now = touches.get();
            if now.knows(event.pointer_id()) {
                now.put(event.pointer_id(), client(&event));
                if let (Some((from, before, was)), Some(spread), Some(midpoint)) =
                    (now.pinch, now.spread(), now.midpoint())
                    && before > 0.0
                {
                    let (_, width, height) = start.get();
                    let (left, top) = origin.get();
                    let at =
                        |point: DragPoint| ((point.x - left) / width, (point.y - top) / height);
                    let rect = match pan {
                        // The image point under the fingers follows them.
                        true if width > 0.0 && height > 0.0 => {
                            from.zoomed(spread / before, at(was), at(midpoint), min)
                        }
                        true => shown,
                        false => from.scaled(spread / before, min),
                    };
                    if rect != shown {
                        emit.call(rect);
                    }
                }
                touches.set(now);
            }
            if !now.pinched {
                drag.onpointermove.call(event);
            }
        }
    };
    let lift = {
        let touches = touches.clone();
        move |event: &Event<PointerData>| {
            let mut now = touches.get();
            if now.knows(event.pointer_id()) {
                now.lift(event.pointer_id());
                touches.set(now);
            }
        }
    };
    let onpointerup = {
        let lift = lift.clone();
        move |event: Event<PointerData>| {
            lift(&event);
            drag.onpointerup.call(event);
        }
    };
    let onpointercancel = move |event: Event<PointerData>| {
        lift(&event);
        drag.onpointercancel.call(event);
    };

    let key = move |grabbed: Grip| {
        move |event: Event<KeyboardData>| {
            if !interactive || has_shortcut_modifier(&event) {
                return;
            }
            let step = match event.modifiers().shift() {
                true => KEY_STEP * 10.0,
                false => KEY_STEP,
            };
            let centre = (0.5, 0.5);
            let rect = match event.key() {
                Key::ArrowLeft => shown.resized(grabbed, -step, 0.0, ratio, min),
                Key::ArrowRight => shown.resized(grabbed, step, 0.0, ratio, min),
                Key::ArrowUp => shown.resized(grabbed, 0.0, -step, ratio, min),
                Key::ArrowDown => shown.resized(grabbed, 0.0, step, ratio, min),
                Key::Character(c) if pan && (c == "+" || c == "=") => {
                    shown.zoomed(ZOOM_STEP, centre, centre, min)
                }
                Key::Character(c) if pan && c == "-" => {
                    shown.zoomed(1.0 / ZOOM_STEP, centre, centre, min)
                }
                _ => return,
            };
            event.prevent_default();
            if rect != shown {
                emit.call(rect);
            }
        }
    };

    // A wheel notch scales by ZOOM_STEP, a trackpad pinch (ctrl+wheel) by its spread: pan mode
    // zooms about the pointer, free mode scales the box about its centre, as a pinch does.
    // Wheel events before the next render chain from the last one's rect.
    let wheeled = use_local_state(|| None::<(CropRect, CropRect)>);
    let onwheel = move |event: Event<WheelData>| {
        if !interactive {
            return;
        }
        event.prevent_default();
        let travel = platform::wheel_travel_y(&event.data(), 40.0, 20.0);
        let factor = match event.modifiers().ctrl() {
            true => (-travel / 100.0).exp(),
            false => ZOOM_STEP.powf(-travel / 100.0),
        };
        let chained = move |wheeled: &Option<(CropRect, CropRect)>| match *wheeled {
            Some((before, after)) if before == shown => after,
            _ => shown,
        };
        if !pan {
            let base = chained(&wheeled.get());
            let rect = base.scaled(factor, min);
            wheeled.set(Some((shown, rect)));
            if rect != base {
                emit.call(rect);
            }
            return;
        }
        let client = event.client_coordinates();
        let (size, offset) = (stage.dimensions(), stage.client_offset());
        let wheeled = wheeled.clone();
        spawn(async move {
            let (Ok(size), Ok((left, top))) = (size.await, offset.await) else {
                return;
            };
            if size.width <= 0.0 || size.height <= 0.0 {
                return;
            }
            let base = chained(&wheeled.get());
            let at = (
                (client.x - left) / size.width,
                (client.y - top) / size.height,
            );
            let rect = base.zoomed(factor, at, at, min);
            wheeled.set(Some((shown, rect)));
            if rect != base {
                emit.call(rect);
            }
        });
    };

    // The single-pointer alternative to every drag (WCAG 2.5.7): a press each.
    let nudge = move |dx: f64, dy: f64| {
        Callback::new(move |_: MouseEvent| {
            let rect = shown.resized(Grip::Move, dx, dy, ratio, min);
            if rect != shown {
                emit.call(rect);
            }
        })
    };
    let resize = move |factor: f64| {
        Callback::new(move |_: MouseEvent| {
            let rect = shown.scaled(factor, min);
            if rect != shown {
                emit.call(rect);
            }
        })
    };
    let has_controls = props.onchange.is_some();

    let percent = |fraction: f64| (fraction * 100.0).round();
    let (scale, image_x, image_y) = shown.image_transform();
    let valuetext = match pan {
        true => fill(
            words.pan_value,
            &[
                ("zoom", &percent(scale)),
                ("width", &percent(shown.width)),
                ("height", &percent(shown.height)),
                ("x", &percent(shown.x)),
                ("y", &percent(shown.y)),
            ],
        ),
        false => fill(
            words.value,
            &[
                ("width", &percent(shown.width)),
                ("height", &percent(shown.height)),
                ("x", &percent(shown.x)),
                ("y", &percent(shown.y)),
            ],
        ),
    };
    // The zoom at the largest and the smallest crop of this shape.
    let zoom_range = (
        shown.scaled(f64::MAX, min).image_transform().0,
        shown.scaled(0.0, min).image_transform().0,
    );
    let zoom_text =
        use_callback(move |zoom: f64| fill(words.zoom_value, &[("zoom", &percent(zoom.exp()))]));
    let keys = if pan { words.pan_keys } else { words.keys };
    let tabindex = if interactive { "0" } else { "-1" };
    let placed_at = if pan { shown.frame() } else { shown };
    let mut crop_variables = variables()
        .with(CROP_X, placed_at.x.to_string())
        .with(CROP_Y, placed_at.y.to_string())
        .with(CROP_WIDTH, placed_at.width.to_string())
        .with(CROP_HEIGHT, placed_at.height.to_string());
    if pan {
        crop_variables = crop_variables
            .with(IMAGE_SCALE, scale.to_string())
            .with(IMAGE_X, image_x.to_string())
            .with(IMAGE_Y, image_y.to_string());
    }
    let crop_variables: Input<Variables> = crop_variables.into();
    let states: Input<States> = props
        .states
        .clone()
        .unwrap_or_default()
        .with("circle", props.shape == CropShape::Circle)
        .with("disabled", props.disabled)
        .with("pan", pan)
        .with("hidden-controls", !props.controls)
        .into();
    let aria_label = props
        .aria_label
        .clone()
        .unwrap_or_else(|| words.label.to_string());

    let handles = Grip::HANDLES.into_iter().filter(|_| !pan).map(|grabbed| {
        let corner = match grabbed {
            Grip::NorthWest => Some((corners[0], words.top_left)),
            Grip::NorthEast => Some((corners[1], words.top_right)),
            Grip::SouthEast => Some((corners[2], words.bottom_right)),
            Grip::SouthWest => Some((corners[3], words.bottom_left)),
            _ => None,
        };
        let onpointerdown = press(grabbed);
        match corner {
            Some((element, label)) => rsx! {
                div {
                    key: "{grabbed.slot()}",
                    "data-slot": ImageCropperPart::Handle.slot(),
                    "data-grip": grabbed.slot(),
                    role: "slider",
                    tabindex,
                    "aria-label": label,
                    "aria-describedby": keys_id(),
                    "aria-valuemin": 0,
                    "aria-valuemax": 100,
                    "aria-valuenow": percent(shown.width),
                    "aria-valuetext": valuetext.clone(),
                    "aria-disabled": (!interactive).then_some("true"),
                    onmounted: element.mount(),
                    onpointerdown,
                    onkeydown: key(grabbed),
                    span {}
                }
            },
            None => rsx! {
                div {
                    key: "{grabbed.slot()}",
                    "data-slot": ImageCropperPart::Handle.slot(),
                    "data-grip": grabbed.slot(),
                    onpointerdown,
                    span {}
                }
            },
        }
    });

    let onpointerdown_box = press(Grip::Move);
    let onpointerdown_image = press(Grip::Move);
    use_box()
        .framework_sx(&IMAGE_CROPPER_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .variables(&crop_variables)
        .prepare()
        .element(&root)
        .event("onpointermove", onpointermove)
        .event("onpointerup", onpointerup)
        .event("onpointercancel", onpointercancel)
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                img {
                    "data-slot": ImageCropperPart::Image.slot(),
                    src: props.src,
                    alt: props.alt,
                    draggable: "false",
                    onmounted: image.mount(),
                    onload: move |_| measure.call(()),
                    onerror,
                }
                if !failed() {
                    div { "data-slot": "mask", "aria-hidden": "true",
                        onpointerdown: onpointerdown_image,
                        onwheel: onwheel.clone(),
                        div {}
                    }
                    div {
                        "data-slot": ImageCropperPart::Box.slot(),
                        role: "slider",
                        tabindex,
                        "aria-label": aria_label,
                        "aria-describedby": keys_id(),
                        "aria-valuemin": 0,
                        "aria-valuemax": 100,
                        "aria-valuenow": percent(shown.x),
                        "aria-valuetext": valuetext.clone(),
                        "aria-disabled": (!interactive).then_some("true"),
                        onmounted: box_element.mount(),
                        onkeydown: key(Grip::Move),
                    }
                    div {
                        "data-slot": ImageCropperPart::Frame.slot(),
                        onmounted: frame.mount(),
                        onpointerdown: onpointerdown_box,
                        onwheel,
                        {handles}
                        div { "data-slot": "move" }
                    }
                    span { id: keys_id(), hidden: true, "{keys}" }
                    if pan {
                        div { "data-slot": ImageCropperPart::Zoom.slot(),
                            // On ln(zoom), so every step zooms by the same factor (1476).
                            Slider::<f64> {
                                value: scale.ln(),
                                min: zoom_range.0.ln(),
                                max: zoom_range.1.ln(),
                                step: 0.01,
                                disabled: !interactive,
                                aria_label: words.zoom,
                                format: zoom_text,
                                oninput: move |event: SliderChangeEvent| {
                                    let factor = event.value().exp() / scale;
                                    let rect = shown.zoomed(factor, (0.5, 0.5), (0.5, 0.5), min);
                                    if rect != shown {
                                        emit.call(rect);
                                    }
                                },
                            }
                        }
                    }
                    if has_controls {
                        div {
                            "data-slot": ImageCropperPart::Controls.slot(),
                            role: "group",
                            "aria-label": aria_label.clone(),
                            ActionIcon {
                                variant: "standard",
                                color: "muted",
                                size: "sm",
                                aria_label: words.move_left,
                                disabled: !interactive,
                                onclick: nudge(-NUDGE_STEP, 0.0),
                                Glyph { slot: IconSlot::ChevronLeft, icon: lucide::chevron_left::outlined }
                            }
                            ActionIcon {
                                variant: "standard",
                                color: "muted",
                                size: "sm",
                                aria_label: words.move_up,
                                disabled: !interactive,
                                onclick: nudge(0.0, -NUDGE_STEP),
                                Glyph { slot: IconSlot::ChevronUp, icon: lucide::chevron_up::outlined }
                            }
                            ActionIcon {
                                variant: "standard",
                                color: "muted",
                                size: "sm",
                                aria_label: words.move_down,
                                disabled: !interactive,
                                onclick: nudge(0.0, NUDGE_STEP),
                                Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined }
                            }
                            ActionIcon {
                                variant: "standard",
                                color: "muted",
                                size: "sm",
                                aria_label: words.move_right,
                                disabled: !interactive,
                                onclick: nudge(NUDGE_STEP, 0.0),
                                Glyph { slot: IconSlot::ChevronRight, icon: lucide::chevron_right::outlined }
                            }
                            // Pan mode zooms through its slider instead.
                            if !pan {
                                ActionIcon {
                                    variant: "standard",
                                    color: "muted",
                                    size: "sm",
                                    aria_label: words.smaller,
                                    disabled: !interactive,
                                    onclick: resize(1.0 / ZOOM_STEP),
                                    Glyph { slot: IconSlot::Minus, icon: lucide::minus::outlined }
                                }
                                ActionIcon {
                                    variant: "standard",
                                    color: "muted",
                                    size: "sm",
                                    aria_label: words.larger,
                                    disabled: !interactive,
                                    onclick: resize(ZOOM_STEP),
                                    Glyph { slot: IconSlot::Plus, icon: lucide::plus::outlined }
                                }
                            }
                        }
                    }
                }
            },
        )
}
