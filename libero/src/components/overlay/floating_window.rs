use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::prelude::*;

use crate::{
    components::{
        ActionIcon, Button, HtmlTag, Input, Menu, MenuEntry, MenuItem, Placement, Title, Variables,
        common::{
            ChevronDownIcon, ChevronLeftIcon, ChevronRightIcon, ChevronUpIcon, CloseIcon,
            focus_ring_sx, has_shortcut_modifier, inset_focus_ring_sx,
        },
        layout::{Float, paper_sx, use_box},
        use_menu, variables,
    },
    context::{ModalContext, WindowHost},
    hooks::{
        Drag, DragMove, DragOptions, DragStart, ElementHandle, drag_handle_sx, escape_closes,
        use_dismiss_layer, use_drag, use_element, use_focus_within, use_id,
    },
    localization::{FloatingWindowLabels, fill},
    platform::{ElementApi, KeyChord, KeySubscription, PlatformError, keyboard},
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
    /// Pins it where `placement` put it: no drag, no keyboard move, no Move
    /// in the title-bar menu.
    pub pinned: bool,
    pub z_index: Input<ThemeAwareValue>,
    /// On the window itself - this is where `min_width`/`max_width` and
    /// friends go. The resize handle asks for a size, and these clamp it.
    /// The viewport cap always applies on top: a `max_width("40rem")` is
    /// still no wider than the screen.
    pub sx: Input<Sx>,
    /// After a drag, a keyboard or button move, or a Reset.
    pub onmove: Option<Callback<WindowRect>>,
    /// After a resize, by pointer, keyboard or button, or a Reset.
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
                // A long word wraps instead of being clipped by the window (1.4.10).
                .selector("& h2", sx().margin("0").with("overflow-wrap", "anywhere"))
                .selector("&:focus-visible", inset_focus_ring_sx("-2px")),
        )
        .selector(
            "& > [data-window-title-bar] > [data-window-handle][tabindex]",
            drag_handle_sx().cursor("move").user_select("none"),
        )
        .selector(
            "& > [data-window-steps]",
            sx().display("flex")
                .flex_wrap("wrap")
                .align_items("center")
                .gap("xs")
                .padding(format!(
                    "{} {}",
                    SizeCss::SPACING.value(Size::Xs),
                    SizeCss::SPACING.value(Size::Sm)
                ))
                .border_bottom(format!("1px solid {}", PAPER_BORDER_COLOR.value())),
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
                .selector("&:focus-visible", inset_focus_ring_sx("-2px")),
        )
});

static FLOAT_SX: StaticSx = StaticSx::new(|| {
    sx().width("max-content")
        .max_width("100dvw")
        .max_height("100dvh")
        .display("flex")
        .flex_direction("column")
});

/// What the title-bar menu asked for: the single-pointer way to move and
/// resize (2.5.7), where the handles need a drag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Adjust {
    Move,
    Resize,
    Reset,
}

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

/// The size range the window's CSS clamps a requested size into, in pixels:
/// the caller's `min-*`/`max-*`, and the viewport on top of the max.
#[derive(Clone, Copy, Debug, PartialEq)]
struct WindowBounds {
    min: (f64, f64),
    max: (f64, f64),
}

impl WindowBounds {
    /// What CSS draws for a requested size: `min-*` wins over `max-*`.
    fn fit(self, (width, height): (f64, f64)) -> (f64, f64) {
        (
            width.min(self.max.0).max(self.min.0),
            height.min(self.max.1).max(self.min.1),
        )
    }
}

/// Reads the window's computed `min-*`/`max-*` and the viewport into `bounds`.
/// Where there is no computed style, `bounds` stays as it was.
fn read_bounds(root: ElementHandle, mut bounds: Signal<Option<WindowBounds>>) {
    let reads = ["min-width", "min-height", "max-width", "max-height"]
        .map(|property| root.computed_px(property));
    let viewport = crate::platform::document().map(|document| document.viewport());
    spawn(async move {
        let mut px = [None; 4];
        for (slot, read) in px.iter_mut().zip(reads) {
            let Ok(value) = read.await else { return };
            *slot = value;
        }
        let viewport = match viewport {
            Some(read) => read.await.ok(),
            None => None,
        };
        let cap = |max: Option<f64>, screen: Option<f64>| {
            max.unwrap_or(f64::INFINITY)
                .min(screen.unwrap_or(f64::INFINITY))
        };
        let next = WindowBounds {
            min: (px[0].unwrap_or(0.0), px[1].unwrap_or(0.0)),
            max: (
                cap(px[2], viewport.map(|v| v.width)),
                cap(px[3], viewport.map(|v| v.height)),
            ),
        };
        if *bounds.peek() != Some(next) {
            bounds.set(Some(next));
        }
    });
}

/// Whether focus is known to be outside `root`. A platform that cannot say
/// answers `false`, so focus still returns to the trigger.
fn focus_elsewhere(root: &ElementHandle) -> bool {
    match root.query_selector(":focus") {
        Ok(_) => false,
        _ if root.is_focused() => false,
        Err(PlatformError::Unsupported) => false,
        Err(_) => true,
    }
}

/// Whether focus is known to be on `root` or inside it.
fn focus_within(root: &ElementHandle) -> bool {
    root.is_focused() || root.query_selector(":focus").is_ok()
}

fn active_element() -> Option<Rc<dyn ElementApi>> {
    crate::platform::document()
        .and_then(|document| document.active_element())
        .map(Rc::from)
}

/// F6 moves focus between the page and the topmost window (todo 572): the
/// window is portaled past the page's end, so Tab reaches it last.
///
/// Not while a dismissible layer is open: a modal or a popover keeps it.
fn use_page_switch(
    root: ElementHandle,
    host: WindowHost,
    id: u64,
    opener: Callback<(), Option<Rc<dyn ElementApi>>>,
) {
    let layer = use_dismiss_layer();
    // Where the page had focus: the opener at first, then wherever F6 left.
    let mut page = use_hook(|| CopyValue::new(opener.call(())));
    // Bumped from the key callback, which on the web runs outside every scope;
    // the effect below moves focus (the `Spotlight` hotkey's shape).
    let tick = use_signal(|| 0u64);
    let slot: Rc<RefCell<Option<Box<dyn KeySubscription>>>> = use_hook(|| {
        let callback = Box::new(move |chord: KeyChord| {
            let modifiers = chord.modifiers;
            // Shift+F6 too: with two stops, backward is forward.
            if chord.key != Key::F6
                || modifiers.ctrl()
                || modifiers.alt()
                || modifiers.meta()
                || !host.is_top(id)
                || layer.any_open()
            {
                return false;
            }
            if !chord.repeat {
                let mut tick = tick;
                let next = tick.peek().wrapping_add(1);
                tick.set(next);
            }
            true
        });
        Rc::new(RefCell::new(
            keyboard().map(|api| api.on_key_unfiltered(callback)),
        ))
    });
    use_drop(move || {
        slot.borrow_mut().take();
    });

    let mut seen = use_signal(|| 0u64);
    use_effect(move || {
        let pressed = tick();
        if pressed == *seen.peek() {
            return;
        }
        seen.set(pressed);
        if focus_within(&root) {
            let target = page.peek().clone().filter(|target| target.is_connected());
            if let Some(target) = target {
                let _ = target.focus();
            }
        } else {
            page.set(active_element());
            let _ = root.focus();
        }
    });
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
    /// Hands the handle's `close` a way to ask whether focus left the window.
    onmount: Callback<Callback<(), bool>>,
    /// Where the page had focus, read by `open` in its handler: Blitz cannot
    /// answer the window's first render (todo 671).
    opener: Callback<(), Option<Rc<dyn ElementApi>>>,
    children: Element,
}

/// The window the hook portals. It owns its geometry: the caller never sees
/// a position unless it asks through `onmove`/`onresize`.
#[component]
pub(crate) fn FloatingWindow(props: FloatingWindowProps) -> Element {
    let defaults = crate::hooks::use_theme().floating_window;
    let localization = crate::hooks::use_localization();
    let labels = localization.floating_window;
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
    // Non-modal: content opened from a modal must not inherit its close.
    use_context_provider(|| ModalContext::NONE);
    let title_id = use_id();
    let root = use_element();
    let focus_left = use_callback(move |()| focus_elsewhere(&root));
    let onmount = props.onmount;
    use_hook(move || onmount.call(focus_left));
    crate::components::common::use_name_warning(
        title.is_some() || aria_label.is_some(),
        "FloatingWindow: no `title` or `aria_label` in its options, so it is announced as \
         just \"dialog\".",
    );

    // Stacking: opened on top, raised to the top when clicked or focused.
    let host = use_context::<WindowHost>();
    let id = use_hook(|| NEXT_WINDOW_ID.fetch_add(1, Ordering::Relaxed));
    use_effect(move || host.raise(id));
    use_drop(move || host.remove(id));
    let focus = use_focus_within(
        move || vec![root.mounted()],
        move |change| {
            if change.within {
                host.raise(id);
            }
        },
    );
    use_page_switch(root, host, id, props.opener);
    let z_index = match z_index {
        Input::None => Input::from(host.z_index(id).to_string()),
        caller => caller,
    };

    let geometry = use_window_geometry(
        root,
        f64::from(defaults.move_step),
        f64::from(defaults.resize_step),
        onmove,
        onresize,
    );
    let WindowGeometry {
        position,
        size,
        mut measured,
        bounds,
        move_drag,
        resize_drag,
        ..
    } = geometry;
    let move_key = use_callback(move |event| geometry.handle_key(event));

    // Move or Resize from the menu shows step buttons until Done or Escape.
    let mut adjusting = use_signal(|| None::<Adjust>);
    let onadjust = use_callback(move |adjust: Adjust| match adjust {
        Adjust::Reset => {
            adjusting.set(None);
            geometry.reset();
        }
        adjust => adjusting.set(Some(adjust)),
    });
    let onstep = use_callback(move |(dx, dy): (f64, f64)| match *adjusting.peek() {
        Some(Adjust::Move) => geometry.move_by(dx * geometry.move_step, dy * geometry.move_step),
        Some(Adjust::Resize) => {
            geometry.resize_to(Ok((dx * geometry.resize_step, dy * geometry.resize_step)))
        }
        _ => {}
    });
    let ondone = use_callback(move |()| {
        adjusting.set(None);
        if let Ok(trigger) = root.query_selector("[data-window-menu]") {
            let _ = trigger.focus();
        }
    });
    let steps = adjusting().map(|adjust| {
        rsx! {
            WindowSteps { adjust, labels, onstep, ondone }
        }
    });

    // Not a dismiss layer: a window is non-modal and hears only presses from
    // inside it. It still skips one that something inside already took - an
    // open `Select` closing its list, or on the web a `HoverCard` answering at
    // the document - so one Escape closes one layer, as in `Modal`. A held
    // Escape's repeats and a composing one are not closes either.
    let onkeydown = move |event: Event<KeyboardData>| {
        if escape_closes(&event) {
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

    // The separator's value comes from this render's own request, clamped as
    // the CSS will clamp it; only an auto-sized window waits for `onresize`.
    let bounds_now = bounds();
    let (width, height) = match (size(), bounds_now) {
        (Some(requested), Some(bounds)) => bounds.fit(requested),
        _ => measured().unwrap_or_default(),
    };
    let (value_min, value_max) = match bounds_now.filter(|b| b.max.0.is_finite()) {
        Some(b) => (
            Some(b.min.0.round().to_string()),
            Some(b.max.0.max(b.min.0).round().to_string()),
        ),
        None => (None, None),
    };

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
        .event("onfocusin", focus.focusin(0))
        // Pointer too: a drag's pointerdown prevents the focus a click
        // would otherwise bring.
        .event("onpointerdown", move |_: Event<PointerData>| host.raise(id))
        .event("onpointermove", move |event: Event<PointerData>| {
            geometry.onpointermove(event)
        })
        .event("onpointerup", move |event: Event<PointerData>| {
            geometry.onpointerup(event)
        })
        .event("onpointercancel", move |event: Event<PointerData>| {
            geometry.onpointercancel(event)
        })
        .event("onresize", move |event: Event<ResizeData>| {
            if let Ok(size) = event.get_border_box_size() {
                measured.set(Some((size.width, size.height)));
            }
            read_bounds(root, bounds);
        })
        .render(
            HtmlTag::Div,
            Vec::new(),
            rsx! {
                WindowTitleBar {
                    title,
                    title_id,
                    pinned,
                    resizable,
                    labels,
                    close_label: localization.common.close,
                    onclose,
                    onadjust,
                    onpointerdown: move_drag.onpointerdown,
                    onkeydown: move_key,
                }
                {steps}
                div { "data-window-body": "", {props.children} }
                if resizable {
                    div {
                        role: "separator",
                        tabindex: "0",
                        "aria-label": labels.resize_handle,
                        // One handle resizes two axes, so a single number is
                        // a compromise: the width, with the whole size in words.
                        // Real pixels: ARIA's implicit 0..100 clamps the value.
                        "aria-valuemin": value_min,
                        "aria-valuemax": value_max,
                        "aria-valuenow": "{width.round()}",
                        "aria-valuetext": fill(
                            labels.size,
                            &[("width", &width.round()), ("height", &height.round())],
                        ),
                        onpointerdown: move |event| resize_drag.onpointerdown.call(event),
                        onkeydown: move |event| geometry.resize_key(event),
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
            sx: &FLOAT_SX,
            {window}
        }
    }
}

/// Its own scope, so a drag frame or a host re-render skips the heading and
/// the close button: every prop compares equal until the title changes.
#[component]
fn WindowTitleBar(
    title: Option<String>,
    title_id: Signal<String>,
    pinned: bool,
    resizable: bool,
    labels: FloatingWindowLabels,
    close_label: &'static str,
    onclose: Callback<()>,
    onadjust: Callback<Adjust>,
    onpointerdown: Callback<Event<PointerData>>,
    onkeydown: Callback<Event<KeyboardData>>,
) -> Element {
    let hint_id = use_id();
    let menu = use_menu();
    let item = |label: &'static str, adjust: Adjust| -> MenuEntry {
        MenuItem::new(label)
            .onselect(move |_| onadjust.call(adjust))
            .into()
    };
    let items: Vec<MenuEntry> = [
        (!pinned).then(|| item(labels.move_item, Adjust::Move)),
        resizable.then(|| item(labels.resize_item, Adjust::Resize)),
        (!pinned || resizable).then(|| item(labels.reset_item, Adjust::Reset)),
    ]
    .into_iter()
    .flatten()
    .collect();
    let mut trigger = menu.a11y_attributes();
    trigger.push(Attribute::new("data-window-menu", "", None, false));
    let (move_label, move_hint) = (labels.move_handle, labels.move_hint);
    rsx! {
        div { "data-window-title-bar": "",
            div {
                "data-window-handle": "",
                // Focusable because it is the keyboard move handle, and named
                // for that job; the heading inside it is the window's name.
                role: if !pinned { "group" },
                tabindex: if !pinned { "0" },
                "aria-label": if !pinned { move_label },
                "aria-describedby": if !pinned { hint_id() },
                onpointerdown: move |event| {
                    if !pinned {
                        onpointerdown.call(event);
                    }
                },
                onkeydown: move |event| {
                    if !pinned {
                        onkeydown.call(event);
                    }
                },
                if let Some(title) = title {
                    Title { id: title_id(), component: "h2", size: "sm", "{title}" }
                }
            }
            if !pinned {
                span { id: hint_id(), hidden: true, "{move_hint}" }
            }
            if !items.is_empty() {
                Menu { state: menu, items,
                    ActionIcon {
                        variant: "standard",
                        color: "muted",
                        size: "sm",
                        aria_label: labels.menu,
                        attributes: trigger,
                        ChevronDownIcon {}
                    }
                }
            }
            ActionIcon {
                variant: "standard",
                color: "muted",
                size: "sm",
                aria_label: close_label,
                onclick: move |_| onclose.call(()),
                CloseIcon {}
            }
        }
    }
}

/// The step buttons Move or Resize in the title-bar menu shows: a click moves
/// or resizes by the theme's step, with no drag. Focus starts on the first;
/// Done or Escape hides them.
#[component]
fn WindowSteps(
    adjust: Adjust,
    labels: FloatingWindowLabels,
    onstep: Callback<(f64, f64)>,
    ondone: Callback<()>,
) -> Element {
    let group = use_element();
    // Again when the menu switches Move to Resize: focus is on its trigger.
    use_effect(use_reactive!(|adjust| {
        let _ = adjust;
        if group.is_mounted()
            && let Ok(first) = group.query_selector("button")
        {
            let _ = first.focus();
        }
    }));
    let (name, [up, down, left, right]) = match adjust {
        Adjust::Resize => (
            labels.resize_handle,
            [labels.shorter, labels.taller, labels.narrower, labels.wider],
        ),
        _ => (
            labels.move_handle,
            [
                labels.move_up,
                labels.move_down,
                labels.move_left,
                labels.move_right,
            ],
        ),
    };
    let step = |label: &'static str, delta: (f64, f64), icon: Element| {
        rsx! {
            ActionIcon {
                variant: "standard",
                size: "sm",
                aria_label: label,
                onclick: move |_| onstep.call(delta),
                {icon}
            }
        }
    };
    rsx! {
        div {
            "data-window-steps": "",
            role: "group",
            "aria-label": name,
            onmounted: group.mount(),
            // Escape leaves the buttons, not the window.
            onkeydown: move |event: Event<KeyboardData>| {
                if escape_closes(&event) {
                    event.stop_propagation();
                    event.prevent_default();
                    ondone.call(());
                }
            },
            {step(up, (0.0, -1.0), rsx! { ChevronUpIcon {} })}
            {step(down, (0.0, 1.0), rsx! { ChevronDownIcon {} })}
            {step(left, (-1.0, 0.0), rsx! { ChevronLeftIcon {} })}
            {step(right, (1.0, 0.0), rsx! { ChevronRightIcon {} })}
            Button { variant: "outlined", size: "xs", onclick: move |_| ondone.call(()), "{labels.done}" }
        }
    }
}

/// A floating window's geometry: where it is, how big it is, and the two
/// drags that move and resize it. One `Copy` argument, so the title bar, the
/// corner handle and the window itself all read the same state.
#[derive(Clone, Copy)]
struct WindowGeometry {
    root: ElementHandle,
    /// Top-left in viewport pixels once moved; `None` while `placement`
    /// decides.
    position: Signal<Option<(f64, f64)>>,
    /// What the resize handle asked for; `None` is the content's own size.
    size: Signal<Option<(f64, f64)>>,
    /// The rendered border box, for the clamp. Kept current by `onresize`.
    measured: Signal<Option<(f64, f64)>>,
    /// The caller's size bounds in pixels; `None` until read, and where there
    /// is no computed style.
    bounds: Signal<Option<WindowBounds>>,
    /// A keyboard or button move or resize reports once the new geometry has
    /// rendered.
    owed: Signal<Vec<Callback<WindowRect>>>,
    move_drag: Drag,
    resize_drag: Drag,
    onmove: Option<Callback<WindowRect>>,
    onresize: Option<Callback<WindowRect>>,
    move_step: f64,
    resize_step: f64,
}

/// The window's own geometry state, the two drags over it, and the effect that
/// pays back a report owed to a keyboard move.
fn use_window_geometry(
    root: ElementHandle,
    move_step: f64,
    resize_step: f64,
    onmove: Option<Callback<WindowRect>>,
    onresize: Option<Callback<WindowRect>>,
) -> WindowGeometry {
    let mut position = use_signal(|| None::<(f64, f64)>);
    let mut size = use_signal(|| None::<(f64, f64)>);
    let measured = use_signal(|| None::<(f64, f64)>);
    let bounds = use_signal(|| None::<WindowBounds>);
    use_effect(move || {
        if root.is_mounted() {
            read_bounds(root, bounds);
        }
    });
    // Where a pointer drag started, read once at pointerdown.
    let mut move_origin = use_signal(|| None::<(f64, f64)>);
    let mut size_origin = use_signal(|| None::<(f64, f64)>);

    // A keyboard move or resize reports once the new geometry has rendered:
    // reading the rect in the same task as the write would report the old one.
    // An effect runs after the render commits, and the read forces layout.
    let mut owed = use_signal(Vec::<Callback<WindowRect>>::new);
    use_effect(move || {
        let callbacks = owed();
        if !callbacks.is_empty() {
            owed.set(Vec::new());
            for callback in callbacks {
                report(root, Some(callback));
            }
        }
    });

    // Focus the window itself on open, which is what APG asks of a non-modal
    // dialog. Once: a later re-render must not pull focus back from the page.
    let mut focused = use_signal(|| false);
    use_effect(move || {
        if root.is_mounted() && !*focused.peek() {
            focused.set(true);
            let _ = root.focus();
        }
    });

    let move_drag = use_drag(DragOptions {
        capture: root,
        onstart: use_callback(move |_: DragStart| {
            move_origin.set(None);
            let offset = root.client_offset();
            spawn(async move {
                if let Ok(offset) = offset.await {
                    move_origin.set(Some(offset));
                }
            });
        }),
        onmove: use_callback(move |event: DragMove| {
            if let Some((x, y)) = move_origin() {
                let delta = event.delta();
                position.set(Some((x + delta.x, y + delta.y)));
            }
        }),
        onend: use_callback(move |()| report(root, onmove)),
    });

    let resize_drag = use_drag(DragOptions {
        capture: root,
        onstart: use_callback(move |_: DragStart| {
            size_origin.set(None);
            let dimensions = root.dimensions();
            spawn(async move {
                if let Ok(dimensions) = dimensions.await {
                    size_origin.set(Some((dimensions.width, dimensions.height)));
                }
            });
        }),
        onmove: use_callback(move |event: DragMove| {
            if let Some((width, height)) = size_origin() {
                let delta = event.delta();
                size.set(Some((
                    (width + delta.x).max(0.0),
                    (height + delta.y).max(0.0),
                )));
            }
        }),
        onend: use_callback(move |()| report(root, onresize)),
    });

    WindowGeometry {
        root,
        position,
        size,
        measured,
        bounds,
        owed,
        move_drag,
        resize_drag,
        onmove,
        onresize,
        move_step,
        resize_step,
    }
}

impl WindowGeometry {
    /// Both drags capture on the root, so it receives every move; only the one
    /// that started reacts, because `use_drag` filters on its own pointer id.
    fn onpointermove(self, event: Event<PointerData>) {
        match (self.move_drag.dragging)() {
            true => self.move_drag.onpointermove.call(event),
            false if (self.resize_drag.dragging)() => self.resize_drag.onpointermove.call(event),
            false => {}
        }
    }

    fn onpointerup(self, event: Event<PointerData>) {
        match (self.move_drag.dragging)() {
            true => self.move_drag.onpointerup.call(event),
            false if (self.resize_drag.dragging)() => self.resize_drag.onpointerup.call(event),
            false => {}
        }
    }

    fn onpointercancel(self, event: Event<PointerData>) {
        match (self.move_drag.dragging)() {
            true => self.move_drag.onpointercancel.call(event),
            false if (self.resize_drag.dragging)() => self.resize_drag.onpointercancel.call(event),
            false => {}
        }
    }

    /// The title bar is the keyboard move handle: Arrow moves by a step,
    /// Shift+Arrow by a pixel.
    fn handle_key(self, event: Event<KeyboardData>) {
        // Alt+ArrowLeft is Back: a chord is the browser's, not a move.
        if has_shortcut_modifier(&event) {
            return;
        }
        let step = if event.modifiers().shift() {
            1.0
        } else {
            self.move_step
        };
        let Some((dx, dy)) = arrow_delta(&event.key(), step) else {
            return;
        };
        event.prevent_default();
        event.stop_propagation();
        self.move_by(dx, dy);
    }

    /// Moves by a pixel delta from where the window is drawn, and reports it.
    fn move_by(self, dx: f64, dy: f64) {
        let (mut position, mut owed, onmove) = (self.position, self.owed, self.onmove);
        let offset = self.root.client_offset();
        spawn(async move {
            if let Ok((x, y)) = offset.await {
                position.set(Some((x + dx, y + dy)));
                owed.write().extend(onmove);
            }
        });
    }

    /// Back to `placement` and the content's own size, reporting both.
    fn reset(self) {
        let (mut position, mut size, mut owed) = (self.position, self.size, self.owed);
        position.set(None);
        size.set(None);
        owed.write()
            .extend(self.onmove.into_iter().chain(self.onresize));
    }

    /// The corner handle: Arrow resizes by a step, Shift+Arrow by a pixel, and
    /// Home/End ask for nothing and for everything, which the window's
    /// min/max constraints then clamp.
    fn resize_key(self, event: Event<KeyboardData>) {
        if has_shortcut_modifier(&event) {
            return;
        }
        let key = event.key();
        let request = match key {
            Key::Home => Some(Err((0.0, 0.0))),
            Key::End => Some(Err((f64::from(u16::MAX), f64::from(u16::MAX)))),
            _ => {
                let step = if event.modifiers().shift() {
                    1.0
                } else {
                    self.resize_step
                };
                arrow_delta(&key, step).map(Ok)
            }
        };
        let Some(request) = request else { return };
        event.prevent_default();
        event.stop_propagation();
        self.resize_to(request);
    }

    /// `Ok` grows the drawn size by a delta, `Err` asks for an absolute size;
    /// the window's min/max constraints clamp either. Reports it.
    fn resize_to(self, request: Result<(f64, f64), (f64, f64)>) {
        let (mut size, mut owed, onresize) = (self.size, self.owed, self.onresize);
        let dimensions = self.root.dimensions();
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
            owed.write().extend(onresize);
        });
    }
}

#[cfg(test)]
mod tests {
    use dioxus::prelude::*;

    use super::{FloatingWindow, FloatingWindowOptions, WindowBounds};
    use crate::{
        LiberoProvider,
        components::{Dialog, Modal},
    };

    /// Todo 527: rendered inline in a modal, not portaled, the window still
    /// cuts its content off from the modal's close.
    #[test]
    fn a_window_inside_a_modal_is_not_modal() {
        fn app() -> Element {
            let onclose = use_callback(|()| {});
            rsx! {
                LiberoProvider {
                    Modal {
                        FloatingWindow {
                            options: FloatingWindowOptions {
                                title: Some("Window".into()),
                                ..Default::default()
                            },
                            onclose,
                            onmount: |_| {},
                            opener: |()| None,
                            Dialog { title: "Inner", "inner content" }
                        }
                    }
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("inner content"), "{html}");
        assert!(!html.contains("aria-modal=\"true\""), "{html}");
    }

    const BOUNDS: WindowBounds = WindowBounds {
        min: (240.0, 120.0),
        max: (480.0, 360.0),
    };

    /// Home and End ask for `0x0` and `u16::MAX`; the separator has to say
    /// what the CSS draws, not what was asked for.
    #[test]
    fn a_request_is_clamped_as_the_css_clamps_it() {
        assert_eq!(BOUNDS.fit((0.0, 0.0)), (240.0, 120.0));
        assert_eq!(BOUNDS.fit((65535.0, 65535.0)), (480.0, 360.0));
        assert_eq!(BOUNDS.fit((300.0, 200.0)), (300.0, 200.0));
    }

    /// CSS lets `min-width` win over `max-width`.
    #[test]
    fn a_min_above_the_max_wins() {
        let crossed = WindowBounds {
            min: (500.0, 0.0),
            max: (480.0, 360.0),
        };
        assert_eq!(crossed.fit((65535.0, 10.0)).0, 500.0);
    }
}
