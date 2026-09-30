use std::{
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::prelude::*;

use super::{
    geometry::{WindowGeometry, read_bounds, use_window_geometry},
    options::{FloatingWindowOptions, FloatingWindowPart},
    page_switch::use_page_switch,
    steps::WindowSteps,
    styles::{FLOAT_SX, WINDOW_SX},
    title_bar::WindowTitleBar,
};
use crate::{
    components::{
        common::{HtmlTag, Input, Part},
        layout::{Float, Placement, use_box},
    },
    context::{ModalContext, WindowHost},
    hooks::{ElementHandle, escape_closes, use_element, use_focus_within, use_id},
    localization::fill,
    platform::{ElementApi, PlatformError},
};

static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(0);

/// The title-bar menu's single-pointer move and resize (WCAG 2.5.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Adjust {
    Move,
    Resize,
    Reset,
}

/// Clamped into the viewport by CSS, so it re-clamps with no listener. A window
/// larger than the viewport pins top-left.
fn clamped(at: f64, size: Option<f64>, viewport: &str) -> String {
    let size = size.unwrap_or(0.0);
    format!("clamp(0px, {at}px, calc({viewport} - {size}px))")
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

/// The window the hook portals. It owns its geometry.
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
        parts,
        menu_parts,
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
        if let Ok(trigger) = root.query_selector("[data-slot='title-bar'] [data-slot='menu']") {
            let _ = trigger.focus();
        }
    });
    let steps = adjusting().map(|adjust| {
        rsx! {
            WindowSteps { adjust, labels, onstep, ondone }
        }
    });

    // Not a dismiss layer, so only presses inside it; one Escape closes one layer,
    // as in `Modal`, and a held or composing Escape closes nothing.
    let onkeydown = move |event: Event<KeyboardData>| {
        if escape_closes(&event) {
            event.stop_propagation();
            onclose.call(());
        }
    };

    let (float_placement, offset_x, offset_y) = match position() {
        Some((x, y)) => {
            let measured = measured();
            // The offsets are physical, from the left: under RTL that corner is the end.
            let left = match root.is_rtl() {
                true => Placement::TopEnd,
                false => Placement::TopStart,
            };
            (
                Input::<Placement>::Value(left),
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

    // Inline, so a resize beats the caller's `sx` size (todo 922). Width capped
    // here as a caller's `max_width` replaces ours; height by the wrapper.
    let resized_style =
        size().map(|(width, height)| format!("width:min({width}px, 100dvw);height:{height}px;"));

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

    let resized = move |event: Event<ResizeData>| {
        if let Ok(size) = event.get_border_box_size() {
            measured.set(Some((size.width, size.height)));
        }
        read_bounds(root, bounds);
    };
    crate::hooks::use_resize_fallback(root, resized);

    let window = use_box()
        .framework_sx(&WINDOW_SX)
        .sx(&user_sx)
        .parts(&parts)
        .style(resized_style)
        .states(
            &crate::components::common::States::default()
                .active(defaults.radius.radius_state_name())
                .active(defaults.shadow.shadow_state_name())
                .with("bordered", true)
                .with("resizable", resizable)
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
        .event("onresize", resized)
        .render(
            HtmlTag::Div,
            // The WebView's `computed_px` finds the root by its tag (1007).
            root.attributes(),
            rsx! {
                WindowTitleBar {
                    title,
                    title_id,
                    pinned,
                    resizable,
                    labels,
                    menu_parts,
                    close_label: localization.common.close,
                    onclose,
                    onadjust,
                    onpointerdown: move_drag.onpointerdown,
                    onkeydown: move_key,
                }
                {steps}
                div { "data-slot": FloatingWindowPart::Body.slot(), {props.children} }
                if resizable {
                    div {
                        "data-slot": FloatingWindowPart::Resize.slot(),
                        role: "separator",
                        tabindex: "0",
                        "aria-label": labels.resize_handle,
                        // The width in real pixels (ARIA's implicit 0..100 would
                        // clamp), with the whole size in words.
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
            // As wide as the window (`left: 50%` would wrap it at half the
            // viewport), and the viewport cap no caller `sx` reaches.
            sx: &FLOAT_SX,
            {window}
        }
    }
}
