use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

use crate::{
    components::{
        ActionIcon, HtmlTag, Input, Placement, Title, Variables,
        common::{CloseIcon, focus_ring_sx},
        layout::{Float, use_box},
        surface::paper_sx,
        variables,
    },
    context::WindowHost,
    hooks::{DragMove, DragOptions, DragStart, drag_handle_sx, use_drag, use_element, use_id},
    platform::{ElementApi, key_taken},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{CssVar, PAPER_BORDER_COLOR, Size, SizeCss},
};

/// A window's position and size in viewport pixels, handed to
/// `onmove`/`onresize` so a caller can persist it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// How a window looks and behaves, shared by every opening.
#[derive(Clone, Default, PartialEq)]
pub struct FloatingWindowOptions {
    /// Title bar heading, and the accessible name unless `aria_label` is set.
    pub title: Option<String>,
    pub aria_label: Option<String>,
    /// Where it first appears. Dragging takes over from there.
    pub placement: Input<Placement>,
    /// Draws the corner resize handle.
    pub resizable: bool,
    /// Pins it where `placement` put it: no drag, no keyboard move.
    pub pinned: bool,
    pub z_index: Input<ThemeAwareValue>,
    /// On the window itself - this is where `min_width`/`max_width` and
    /// friends go. The resize handle asks for a size, and these clamp it.
    /// The viewport cap always applies on top: a `max_width("40rem")` is
    /// still no wider than the screen.
    pub sx: Input<Sx>,
    /// After a drag or a keyboard move.
    pub onmove: Option<Callback<WindowRect>>,
    /// After a resize, by pointer or keyboard.
    pub onresize: Option<Callback<WindowRect>>,
}

static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(0);

// Per instance and unbounded, so they ride in `Variables`, never in a class.
const WINDOW_WIDTH: CssVar = CssVar::new("--lsx-floating-window-width");
const WINDOW_HEIGHT: CssVar = CssVar::new("--lsx-floating-window-height");

// A window is a `Paper`: background, border, radius, shadow and the focus
// contrast all come from `paper_sx()`. The parts are keyed by data attributes
// under the one class, the `Accordion` arrangement.
static WINDOW_SX: StaticSx = StaticSx::new(|| {
    paper_sx()
        // The resize handle's containing block.
        .position("relative")
        .display("flex")
        .flex_direction("column")
        .width(WINDOW_WIDTH.value_or("auto"))
        .height(WINDOW_HEIGHT.value_or("auto"))
        // The cap when the caller's `sx` sets no max. A caller's max replaces
        // these, so the `Float` wrapper and the width variable cap it again.
        .max_width("100dvw")
        .max_height("100dvh")
        .overflow("hidden")
        .selector("&:focus-visible", focus_ring_sx())
        .selector(
            "& > [data-window-title-bar]",
            sx().display("flex")
                .align_items("center")
                .gap("sm")
                .padding(format!(
                    "{} {}",
                    SizeCss::SPACING.value(Size::Xs),
                    SizeCss::SPACING.value(Size::Sm)
                ))
                .border_bottom(format!("1px solid {}", PAPER_BORDER_COLOR.value())),
        )
        .selector(
            "& > [data-window-title-bar] > [data-window-handle]",
            sx().flex("1")
                .min_width("0")
                .min_height("1.5em")
                .selector("& h2", sx().margin("0"))
                .selector("&:focus-visible", focus_ring_sx().outline_offset("-2px")),
        )
        .selector(
            "& > [data-window-title-bar] > [data-window-handle][tabindex]",
            drag_handle_sx().cursor("move").user_select("none"),
        )
        .selector(
            "& > [data-window-body]",
            sx().flex("1 1 auto")
                .min_height("0")
                .overflow("auto")
                .padding("sm"),
        )
        .selector(
            "& > [role=\"separator\"]",
            drag_handle_sx()
                .position("absolute")
                .right("0")
                .bottom("0")
                .width("14px")
                .height("14px")
                .cursor("nwse-resize")
                // Two short strokes in the corner, the usual grip.
                .background(
                    "linear-gradient(135deg, transparent 55%, currentColor 55%, currentColor 62%, \
                     transparent 62%, transparent 75%, currentColor 75%, currentColor 82%, transparent 82%)",
                )
                .opacity("0.6")
                .selector("&:focus-visible", focus_ring_sx().outline_offset("-2px")),
        )
});

/// Arrow keys to a pixel delta, `None` for any other key.
fn arrow_delta(key: &Key, step: f64) -> Option<(f64, f64)> {
    match key {
        Key::ArrowLeft => Some((-step, 0.0)),
        Key::ArrowRight => Some((step, 0.0)),
        Key::ArrowUp => Some((0.0, -step)),
        Key::ArrowDown => Some((0.0, step)),
        _ => None,
    }
}

/// `left`/`top` for a dragged window: where it was put, clamped into the
/// viewport by CSS so it re-clamps when the viewport changes, with no
/// listener. A window larger than the viewport pins to the top-left, because
/// `clamp` answers its minimum when the maximum is below it.
fn clamped(at: f64, size: Option<f64>, viewport: &str) -> String {
    let size = size.unwrap_or(0.0);
    format!("clamp(0px, {at}px, calc({viewport} - {size}px))")
}

/// Reads the window's current rect and hands it to `callback`. The reads
/// start here, in the handler, and are awaited in the task - see
/// `platform::Read`.
fn report(root: crate::hooks::ElementHandle, callback: Option<Callback<WindowRect>>) {
    let Some(callback) = callback else { return };
    let (offset, dimensions) = (root.client_offset(), root.dimensions());
    spawn(async move {
        if let (Ok((x, y)), Ok(dimensions)) = (offset.await, dimensions.await) {
            callback.call(WindowRect {
                x,
                y,
                width: dimensions.width,
                height: dimensions.height,
            });
        }
    });
}

#[derive(Props, Clone, PartialEq)]
pub(crate) struct FloatingWindowProps {
    options: FloatingWindowOptions,
    onclose: Callback<()>,
    children: Element,
}

/// The window the hook portals. It owns its geometry: the caller never sees
/// a position unless it asks through `onmove`/`onresize`.
#[component]
pub(crate) fn FloatingWindow(props: FloatingWindowProps) -> Element {
    let defaults = crate::hooks::use_theme().floating_window;
    let FloatingWindowOptions {
        title,
        aria_label,
        placement,
        resizable,
        pinned,
        z_index,
        sx: user_sx,
        onmove,
        onresize,
    } = props.options;
    let onclose = props.onclose;
    let title_id = use_id();
    let root = use_element();
    crate::utils::use_name_warning(
        title.is_some() || aria_label.is_some(),
        "FloatingWindow: no `title` or `aria_label` in its options, so it is announced as \
         just \"dialog\".",
    );

    // Stacking: opened on top, raised to the top when clicked or focused.
    let host = use_context::<WindowHost>();
    let id = use_hook(|| NEXT_WINDOW_ID.fetch_add(1, Ordering::Relaxed));
    use_effect(move || host.raise(id));
    use_drop(move || host.remove(id));
    let z_index = match z_index {
        Input::None => Input::from(host.z_index(id).to_string()),
        caller => caller,
    };

    // Top-left in viewport pixels once moved; `None` while `placement` decides.
    let mut position = use_signal(|| None::<(f64, f64)>);
    // What the resize handle asked for; `None` is the content's own size.
    let mut size = use_signal(|| None::<(f64, f64)>);
    // The rendered border box, for the clamp. Kept current by `onresize`.
    let mut measured = use_signal(|| None::<(f64, f64)>);
    // Where a pointer drag started, read once at pointerdown.
    let mut move_origin = use_signal(|| None::<(f64, f64)>);
    let mut size_origin = use_signal(|| None::<(f64, f64)>);

    // Focus the window itself on open, which is what APG asks of a non-modal
    // dialog. Once: a later re-render must not pull focus back from the page.
    // A keyboard move or resize reports once the new geometry has rendered:
    // reading the rect in the same task as the write would report the old one.
    // An effect runs after the render commits, and the read forces layout.
    let mut owed = use_signal(|| None::<Option<Callback<WindowRect>>>);
    use_effect(move || {
        if let Some(callback) = owed() {
            owed.set(None);
            report(root, callback);
        }
    });

    let mut focused = use_signal(|| false);
    use_effect(move || {
        if root.is_mounted() && !*focused.peek() {
            focused.set(true);
            let _ = root.focus();
        }
    });

    let move_drag = use_drag(DragOptions {
        capture: root,
        on_start: Callback::new(move |_: DragStart| {
            move_origin.set(None);
            let offset = root.client_offset();
            spawn(async move {
                if let Ok(offset) = offset.await {
                    move_origin.set(Some(offset));
                }
            });
        }),
        on_move: Callback::new(move |event: DragMove| {
            if let Some((x, y)) = move_origin() {
                let delta = event.delta();
                position.set(Some((x + delta.x, y + delta.y)));
            }
        }),
        on_end: Callback::new(move |()| report(root, onmove)),
    });

    let resize_drag = use_drag(DragOptions {
        capture: root,
        on_start: Callback::new(move |_: DragStart| {
            size_origin.set(None);
            let dimensions = root.dimensions();
            spawn(async move {
                if let Ok(dimensions) = dimensions.await {
                    size_origin.set(Some((dimensions.width, dimensions.height)));
                }
            });
        }),
        on_move: Callback::new(move |event: DragMove| {
            if let Some((width, height)) = size_origin() {
                let delta = event.delta();
                size.set(Some((
                    (width + delta.x).max(0.0),
                    (height + delta.y).max(0.0),
                )));
            }
        }),
        on_end: Callback::new(move |()| report(root, onresize)),
    });

    // Both drags capture on the root, so it receives every move; only the one
    // that started reacts, because `use_drag` filters on its own pointer id.
    let onpointermove = move |event: Event<PointerData>| {
        if (move_drag.dragging)() {
            move_drag.onpointermove.call(event);
        } else if (resize_drag.dragging)() {
            resize_drag.onpointermove.call(event);
        }
    };
    let onpointerup = move |event: Event<PointerData>| {
        if (move_drag.dragging)() {
            move_drag.onpointerup.call(event);
        } else if (resize_drag.dragging)() {
            resize_drag.onpointerup.call(event);
        }
    };
    let onpointercancel = move |event: Event<PointerData>| {
        if (move_drag.dragging)() {
            move_drag.onpointercancel.call(event);
        } else if (resize_drag.dragging)() {
            resize_drag.onpointercancel.call(event);
        }
    };

    let move_step = f64::from(defaults.move_step);
    let resize_step = f64::from(defaults.resize_step);

    // The title bar is the keyboard move handle: Arrow moves by a step,
    // Shift+Arrow by a pixel.
    let onhandlekey = move |event: Event<KeyboardData>| {
        let step = if event.modifiers().shift() {
            1.0
        } else {
            move_step
        };
        let Some((dx, dy)) = arrow_delta(&event.key(), step) else {
            return;
        };
        event.prevent_default();
        event.stop_propagation();
        let offset = root.client_offset();
        spawn(async move {
            if let Ok((x, y)) = offset.await {
                position.set(Some((x + dx, y + dy)));
                owed.set(Some(onmove));
            }
        });
    };

    // The corner handle: Arrow resizes by a step, Shift+Arrow by a pixel, and
    // Home/End ask for nothing and for everything, which the window's
    // min/max constraints then clamp.
    let onresizekey = move |event: Event<KeyboardData>| {
        let key = event.key();
        let request = match key {
            Key::Home => Some(Err((0.0, 0.0))),
            Key::End => Some(Err((f64::from(u16::MAX), f64::from(u16::MAX)))),
            _ => {
                let step = if event.modifiers().shift() {
                    1.0
                } else {
                    resize_step
                };
                arrow_delta(&key, step).map(Ok)
            }
        };
        let Some(request) = request else { return };
        event.prevent_default();
        event.stop_propagation();
        let dimensions = root.dimensions();
        spawn(async move {
            let next = match request {
                Err(absolute) => absolute,
                Ok((dx, dy)) => match dimensions.await {
                    Ok(dimensions) => (
                        (dimensions.width + dx).max(0.0),
                        (dimensions.height + dy).max(0.0),
                    ),
                    Err(_) => return,
                },
            };
            size.set(Some(next));
            owed.set(Some(onresize));
        });
    };

    // Not a dismiss layer: a window is non-modal and hears only presses from
    // inside it. It still skips one that something inside already took - an
    // open `Select` closing its list, or on the web a `HoverCard` answering at
    // the document - so one Escape closes one layer, as in `Modal`.
    let onkeydown = move |event: Event<KeyboardData>| {
        if event.key() == Key::Escape && !key_taken(&event) {
            event.stop_propagation();
            onclose.call(());
        }
    };

    let (float_placement, offset_x, offset_y) = match position() {
        Some((x, y)) => {
            let measured = measured();
            (
                Input::<Placement>::Value(Placement::TopStart),
                Input::from(clamped(x, measured.map(|m| m.0), "100dvw")),
                Input::from(clamped(y, measured.map(|m| m.1), "100dvh")),
            )
        }
        None => (
            Input::<Placement>::Value(placement.copied_or(defaults.placement)),
            Input::None,
            Input::None,
        ),
    };

    // A resized width is capped at the viewport here, because a `max_width` in
    // the caller's `sx` replaces the window's own cap. The height needs no
    // `min`: the wrapper is a column flexbox capped at `100dvh`, and the window
    // shrinks into it.
    let window_variables: Input<Variables> = variables()
        .with(
            WINDOW_WIDTH,
            size().map(|(width, _)| format!("min({width}px, 100dvw)")),
        )
        .with(
            WINDOW_HEIGHT,
            size().map(|(_, height)| format!("{height}px")),
        )
        .into();

    let (width, height) = measured().unwrap_or_default();

    let window = use_box()
        .framework_sx(&WINDOW_SX)
        .sx(&user_sx)
        .variables(&window_variables)
        .states(
            &crate::components::States::default()
                .active(defaults.radius.radius_state_name())
                .active(defaults.shadow.shadow_state_name())
                .with("bordered", true)
                .into(),
        )
        .prepare()
        .element(&root)
        .attr("role", "dialog")
        .attr("tabindex", "-1")
        .attr(
            "aria-labelledby",
            (aria_label.is_none() && title.is_some()).then(&*title_id),
        )
        .attr("aria-label", aria_label.clone())
        .event("onkeydown", onkeydown)
        .event("onfocusin", move |_: Event<FocusData>| host.raise(id))
        // Pointer too: a drag's pointerdown prevents the focus a click
        // would otherwise bring.
        .event("onpointerdown", move |_: Event<PointerData>| host.raise(id))
        .event("onpointermove", onpointermove)
        .event("onpointerup", onpointerup)
        .event("onpointercancel", onpointercancel)
        .event("onresize", move |event: Event<ResizeData>| {
            if let Ok(size) = event.get_border_box_size() {
                measured.set(Some((size.width, size.height)));
            }
        })
        .render(
            HtmlTag::Div,
            Vec::new(),
            rsx! {
                div { "data-window-title-bar": "",
                    div {
                        "data-window-handle": "",
                        // Focusable because it is the keyboard move handle,
                        // and named for that job; the heading inside it is
                        // the window's name.
                        role: if !pinned { "group" },
                        tabindex: if !pinned { "0" },
                        "aria-label": if !pinned { defaults.move_label },
                        onpointerdown: move |event| {
                            if !pinned {
                                move_drag.onpointerdown.call(event);
                            }
                        },
                        onkeydown: move |event| {
                            if !pinned {
                                onhandlekey(event);
                            }
                        },
                        if let Some(title) = title.clone() {
                            Title { id: title_id(), component: "h2", size: "sm", "{title}" }
                        }
                    }
                    ActionIcon {
                        variant: "transparent",
                        color: "grey",
                        size: "sm",
                        aria_label: defaults.close_label,
                        onclick: move |_| onclose.call(()),
                        CloseIcon {}
                    }
                }
                div { "data-window-body": "", {props.children} }
                if resizable {
                    div {
                        role: "separator",
                        tabindex: "0",
                        "aria-label": defaults.resize_label,
                        // One handle resizes two axes, so a single number is
                        // a compromise: the width, with the whole size in words.
                        "aria-valuenow": "{width.round()}",
                        "aria-valuetext": "{width.round()} by {height.round()} pixels",
                        onpointerdown: move |event| resize_drag.onpointerdown.call(event),
                        onkeydown: onresizekey,
                    }
                }
            },
        );

    rsx! {
        Float {
            fixed: true,
            placement: float_placement,
            offset_x,
            offset_y,
            z_index,
            // As wide as the window, not as wide as the space left of a
            // centred anchor - `left: 50%` would otherwise wrap it at half
            // the viewport. The viewport cap lives here too, where no caller
            // `sx` reaches: an auto-width window stretches to this box, and the
            // column flexbox shrinks the window's height into it (`overflow:
            // hidden` drops a flex item's automatic minimum to 0).
            sx: sx()
                .width("max-content")
                .max_width("100dvw")
                .max_height("100dvh")
                .display("flex")
                .flex_direction("column"),
            {window}
        }
    }
}
