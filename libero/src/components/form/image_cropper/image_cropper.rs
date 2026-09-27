use dioxus::prelude::*;

use super::crop::{CropRect, CropShape, Grip};
use crate::{
    components::{
        common::{
            HtmlTag, Input, Part, States, Variables, base_props, focus_ring_sx,
            has_shortcut_modifier, parts_enum, variables,
        },
        layout::use_box,
    },
    hooks::{
        DragMove, DragOptions, DragStart, use_drag, use_element, use_id, use_local_state,
        use_localization,
    },
    localization::fill,
    platform::{ElementApi, when_laid_out},
    sx::{StaticSx, sx},
    theme::CssVar,
    utils::warn,
};

/// The crop box's edges, as fractions of the image.
const CROP_X: CssVar = CssVar::new("--lsx-image-cropper-x");
const CROP_Y: CssVar = CssVar::new("--lsx-image-cropper-y");
const CROP_WIDTH: CssVar = CssVar::new("--lsx-image-cropper-width");
const CROP_HEIGHT: CssVar = CssVar::new("--lsx-image-cropper-height");

/// Arrow keys move this far, Shift+arrow ten times as far.
const KEY_STEP: f64 = 0.01;
/// The smallest box side, as a fraction of the image.
const MIN_SIZE: f64 = 0.05;
/// A handle's hit area (WCAG 2.5.8), around a smaller visible square.
const HANDLE_HIT: &str = "24px";
const HANDLE_DOT: &str = "12px";

parts_enum! {
    /// [`ImageCropper`]'s inner parts, for its `parts` prop.
    pub enum ImageCropperPart {
        Image = "image" => "& > [data-slot='image']",
        /// The dimmed image outside the crop.
        Mask = "mask" => "& > [data-slot='mask'] > *",
        /// The crop box, a tab stop of its own.
        Box = "box" => "& > [data-slot='box']",
        /// Over the box, taking its drags and holding the handles: a slider's
        /// children are hidden from assistive tech.
        Frame = "frame" => "& > [data-slot='frame']",
        /// One of the eight resize handles; the corners are tab stops.
        Handle = "handle" => "& > [data-slot='frame'] > [data-slot='handle']",
    }
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
    let base = sx()
        .position("relative")
        .display("inline-block")
        // The box maps onto this, so it must hug the image, in a flex column too.
        .width("fit-content")
        .height("fit-content")
        .max_width("100%")
        .line_height("0")
        .user_select("none")
        .selector(
            "& > [data-slot='image']",
            sx().display("block")
                .max_width("100%")
                .height("auto")
                .pointer_events("none"),
        )
        // Clipped apart from the box, so a handle on the image's edge keeps
        // its whole hit area.
        .selector(
            "& > [data-slot='mask']",
            sx().position("absolute")
                .top("0")
                .left("0")
                .right("0")
                .bottom("0")
                .overflow("hidden")
                .pointer_events("none"),
        )
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
        // Only the box claims touches: the rest of the image still scrolls the page.
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
                .justify_content("center"),
        )
        // Over the handles between them, and at least a dot's size: a box at
        // `min_size` stays movable when the handles' hit areas meet (1264).
        .selector("& > [data-slot='frame'] > [data-slot='move']", {
            let side = format!("max(calc(100% - {HANDLE_HIT}), min(100%, {HANDLE_DOT}))");
            sx().position("absolute")
                .left("50%")
                .top("50%")
                .width(side.clone())
                .height(side)
                .transform("translate(-50%, -50%)")
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
        .when(
            "disabled",
            sx().opacity("0.5").cursor("not-allowed").selector(
                "& > [data-slot='frame'], & > [data-slot='frame'] > [data-slot='handle']",
                sx().cursor("not-allowed"),
            ),
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
        /// The smallest side, a fraction of the image's. Default 0.05.
        #[props(default)]
        min_size: Option<f64>,
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

/// A box with handles over an image, picking the part to keep. Drag the box
/// or a handle, or use the arrow keys on the box and its corners.
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
    let box_element = use_element();
    let corners = [use_element(), use_element(), use_element(), use_element()];
    let keys_id = use_id();

    // The image's displayed width over its height, measured once it loads.
    let mut image_ratio = use_signal(|| None::<f64>);
    let mut held = use_signal(|| None::<CropRect>);
    let grip = use_local_state(|| Grip::Move);
    // The box and the image's client size where a drag started.
    let start = use_local_state(|| (CropRect::FULL, 0.0_f64, 0.0_f64));

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
            let rect = from.resized(grip.get(), delta.x / width, delta.y / height, ratio, min);
            if rect != shown {
                emit.call(rect);
            }
        })
    };
    let onstart = {
        let (start, grip, early) = (start.clone(), grip.clone(), early.clone());
        use_callback(move |event: DragStart| {
            if !interactive {
                event.cancel.call(());
                return;
            }
            let (start, grabbed, early) = (start.clone(), grip.get(), early.clone());
            start.set((shown, 0.0, 0.0));
            early.set(None);
            // Started here, awaited in the task: under Blitz a read resolves
            // where it is called.
            let size = root.dimensions();
            spawn(async move {
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
            let grip = grip.clone();
            Callback::new(move |()| focus_grip.call(grip.get()))
        },
    });

    let press = {
        let grip = grip.clone();
        move |grabbed: Grip| {
            let grip = grip.clone();
            move |event: Event<PointerData>| {
                event.stop_propagation();
                grip.set(grabbed);
                drag.onpointerdown.call(event);
            }
        }
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
            let (dx, dy) = match event.key() {
                Key::ArrowLeft => (-step, 0.0),
                Key::ArrowRight => (step, 0.0),
                Key::ArrowUp => (0.0, -step),
                Key::ArrowDown => (0.0, step),
                _ => return,
            };
            event.prevent_default();
            let rect = shown.resized(grabbed, dx, dy, ratio, min);
            if rect != shown {
                emit.call(rect);
            }
        }
    };

    let percent = |fraction: f64| (fraction * 100.0).round();
    let valuetext = fill(
        words.value,
        &[
            ("width", &percent(shown.width)),
            ("height", &percent(shown.height)),
            ("x", &percent(shown.x)),
            ("y", &percent(shown.y)),
        ],
    );
    let tabindex = if interactive { "0" } else { "-1" };
    let crop_variables: Input<Variables> = variables()
        .with(CROP_X, shown.x.to_string())
        .with(CROP_Y, shown.y.to_string())
        .with(CROP_WIDTH, shown.width.to_string())
        .with(CROP_HEIGHT, shown.height.to_string())
        .into();
    let states: Input<States> = props
        .states
        .clone()
        .unwrap_or_default()
        .with("circle", props.shape == CropShape::Circle)
        .with("disabled", props.disabled)
        .into();
    let aria_label = props
        .aria_label
        .clone()
        .unwrap_or_else(|| words.label.to_string());

    let handles = Grip::HANDLES.into_iter().map(|grabbed| {
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
    use_box()
        .framework_sx(&IMAGE_CROPPER_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .variables(&crop_variables)
        .prepare()
        .element(&root)
        .event("onpointermove", drag.onpointermove)
        .event("onpointerup", drag.onpointerup)
        .event("onpointercancel", drag.onpointercancel)
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
                        onpointerdown: onpointerdown_box,
                        {handles}
                        div { "data-slot": "move" }
                    }
                    span { id: keys_id(), hidden: true, "{words.keys}" }
                }
            },
        )
}
