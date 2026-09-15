//! The picker's 2D panel: saturation left to right, value bottom to top, at
//! the current hue.

use dioxus::prelude::*;

use super::ColorCode;
use crate::{
    components::{
        HtmlTag, Input, Variables,
        common::{has_shortcut_modifier, shadow_sx},
        form::SliderChangeEvent,
        layout::use_box,
        variables,
    },
    hooks::{
        DragMove, DragOptions, DragStart, ElementHandle, drag_handle_sx, use_drag, use_element,
        use_local_state,
    },
    platform::ElementApi,
    sx::{StaticSx, sx},
    theme::{COLOR_PICKER_SATURATION_HEIGHT, COLOR_PICKER_THUMB, CssVar, Size, SizeCss},
};

/// Both 0-1, measured from the panel's left and top edges.
const SATURATION_X: CssVar = CssVar::new("--lsx-color-picker-saturation-x");
const SATURATION_Y: CssVar = CssVar::new("--lsx-color-picker-saturation-y");
/// The fully saturated hue the panel's gradient runs towards. Set on the
/// picker's root, which redraws on every change anyway.
pub(super) const SATURATION_HUE: CssVar = CssVar::new("--lsx-color-picker-saturation-hue");
const SATURATION_COLOR: CssVar = CssVar::new("--lsx-color-picker-saturation-color");

/// Arrow keys move this far, Shift+arrow and Page Up/Down ten times as far.
const KEY_STEP: f64 = 0.01;

static PANEL_SX: StaticSx = StaticSx::new(|| {
    drag_handle_sx()
        .position("relative")
        .height(COLOR_PICKER_SATURATION_HEIGHT.value())
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        // Black rising from the bottom over white-to-hue from the left: the
        // two gradients multiply out to every saturation and value.
        .background(format!(
            "linear-gradient(to top, #000000, rgba(0, 0, 0, 0)), linear-gradient(to right, #ffffff, {})",
            SATURATION_HUE.value()
        ))
        .cursor("crosshair")
        .user_select("none")
});

static THUMB_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .left(format!("calc({} * 100%)", SATURATION_X.value_or("0")))
        .top(format!("calc({} * 100%)", SATURATION_Y.value_or("0")))
        .width(COLOR_PICKER_THUMB.value())
        .height(COLOR_PICKER_THUMB.value())
        .transform("translate(-50%, -50%)")
        .border_radius("50%")
        .border_style("solid")
        .border_width("2px")
        // White, then a 60% black ring: one of the two is at least 3.4:1 on
        // any colour of the pad, in either scheme (WCAG 1.4.11, todo 554).
        .border_color("#ffffff")
        .and(shadow_sx(
            "0 0 0 1px rgba(0, 0, 0, 0.6), inset 0 0 0 1px rgba(0, 0, 0, 0.3)".to_string(),
        ))
        .background(SATURATION_COLOR.value())
        .cursor("grab")
});

/// The panel and its handlers. `value` is the picker's own signal and only
/// [`SaturationThumb`] reads it while rendering, so a drag frame redraws the
/// thumb and skips this scope.
#[derive(Props, Clone, PartialEq)]
pub(super) struct SaturationProps {
    value: Signal<ColorCode>,
    oninput: Option<EventHandler<SliderChangeEvent<ColorCode>>>,
    aria_label: Option<String>,
    focusable: bool,
}

#[component]
pub(super) fn Saturation(props: SaturationProps) -> Element {
    let panel_element = use_element();
    let thumb_element = use_element();

    let value = props.value;
    let interactive = props.oninput.is_some();
    let focusable = props.focusable;

    // `(left, top, width, height)`, measured at pointerdown.
    let rect = use_local_state(|| (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64));
    // What a move is applied to and what `End` reports: the drag's own last
    // color, which a controlled parent may not have echoed back yet.
    let latest = use_local_state(|| *value.peek());

    let oninput = props.oninput;
    let emit = {
        let latest = latest.clone();
        use_callback(move |event: SliderChangeEvent<ColorCode>| {
            latest.set(event.value());
            if let Some(oninput) = &oninput {
                oninput.call(event);
            }
        })
    };

    // Saturation and value under a client-space point, clamped to the panel.
    let at = {
        let rect = rect.clone();
        use_callback(move |(x, y): (f64, f64)| {
            let (left, top, width, height) = rect.get();
            if width <= 0.0 || height <= 0.0 {
                return None;
            }
            Some((
                ((x - left) / width).clamp(0.0, 1.0),
                (1.0 - (y - top) / height).clamp(0.0, 1.0),
            ))
        })
    };

    let grab = use_callback(move |(saturation, brightness): (f64, f64)| {
        emit.call(SliderChangeEvent::Start(
            value.peek().with_saturation_value(saturation, brightness),
        ));
    });
    let slide = {
        let latest = latest.clone();
        use_callback(move |(saturation, brightness): (f64, f64)| {
            emit.call(SliderChangeEvent::Change(
                latest.get().with_saturation_value(saturation, brightness),
            ));
        })
    };

    let onstart = use_callback(move |event: DragStart| {
        if !interactive {
            event.cancel.call(());
            return;
        }
        let rect = rect.clone();
        // Started here, awaited in the task - see `SliderCore`: under
        // Blitz a read resolves where it is called.
        let size = panel_element.dimensions();
        let offset = panel_element.client_offset();
        spawn(async move {
            let (Ok(size), Ok((left, top))) = (size.await, offset.await) else {
                event.cancel.call(());
                return;
            };
            rect.set((left, top, size.width, size.height));
            match at.call((event.client.x, event.client.y)) {
                Some(point) => {
                    grab.call(point);
                    // The drag cancels the pointerdown, and with it the
                    // browser's own focus - the keys would be unreachable.
                    if focusable {
                        let _ = thumb_element.focus();
                    }
                }
                None => event.cancel.call(()),
            }
        });
    });
    let onmove = use_callback(move |event: DragMove| {
        if let Some(point) = at.call((event.client.x, event.client.y)) {
            slide.call(point);
        }
    });
    let onend = use_callback(move |_| {
        emit.call(SliderChangeEvent::End(latest.get()));
    });
    let drag = use_drag(DragOptions {
        capture: panel_element,
        onstart,
        onmove,
        onend,
    });

    let onkeydown = use_callback(move |event: Event<KeyboardData>| {
        // Alt+ArrowLeft is Back: a chord is the browser's, not a step.
        if !interactive || has_shortcut_modifier(&event) {
            return;
        }
        let step = match event.modifiers().shift() {
            true => KEY_STEP * 10.0,
            false => KEY_STEP,
        };
        let value = *value.peek();
        let (saturation, brightness) = (value.saturation(), value.value());
        let moved = match event.key() {
            Key::ArrowRight => (saturation + step, brightness),
            Key::ArrowLeft => (saturation - step, brightness),
            Key::ArrowUp => (saturation, brightness + step),
            Key::ArrowDown => (saturation, brightness - step),
            // APG slider: Home/End take `aria-valuenow`, the saturation, to
            // its ends; Page Up/Down step the brightness ten.
            Key::Home => (0.0, brightness),
            Key::End => (1.0, brightness),
            Key::PageUp => (saturation, brightness + KEY_STEP * 10.0),
            Key::PageDown => (saturation, brightness - KEY_STEP * 10.0),
            _ => return,
        };
        event.prevent_default();
        // `Change` then `End`, as everywhere a key settles a value at once:
        // a caller that commits on `End` must not miss a keyboard edit.
        let moved = value.with_saturation_value(moved.0, moved.1);
        emit.call(SliderChangeEvent::Change(moved));
        emit.call(SliderChangeEvent::End(moved));
    });

    let panel_style = use_box().framework_sx(&PANEL_SX).prepare();
    let thumb = rsx! {
        SaturationThumb {
            value,
            element: thumb_element,
            aria_label: props.aria_label,
            tab_stop: interactive && focusable,
            onkeydown,
        }
    };

    panel_style
        .element(&panel_element)
        .event("onpointerdown", drag.onpointerdown)
        .event("onpointermove", drag.onpointermove)
        .event("onpointerup", drag.onpointerup)
        .event("onpointercancel", drag.onpointercancel)
        .render(HtmlTag::Div, Vec::new(), thumb)
}

/// The thumb, the one part a drag frame changes: its position, its color and
/// what it announces.
#[component]
fn SaturationThumb(
    value: Signal<ColorCode>,
    element: ElementHandle,
    aria_label: Option<String>,
    tab_stop: bool,
    onkeydown: Callback<Event<KeyboardData>>,
) -> Element {
    let value = value();
    let thumb_variables: Input<Variables> = variables()
        .with(SATURATION_X, value.saturation().to_string())
        .with(SATURATION_Y, (1.0 - value.value()).to_string())
        .with(SATURATION_COLOR, value.opaque().to_hex())
        .into();
    let thumb_style = use_box()
        .framework_sx(&THUMB_SX)
        .variables(&thumb_variables)
        .prepare();

    // One `role="slider"` for a 2D control: its value is the saturation, and
    // the up and down keys still move the brightness - so the text names
    // both, or an Up/Down press would announce nothing.
    let percent = |fraction: f64| (fraction * 100.0).round();
    let valuetext = format!(
        "Saturation {}%, brightness {}%",
        percent(value.saturation()),
        percent(value.value()),
    );
    thumb_style
        .attr("role", "slider")
        .attr("tabindex", if tab_stop { "0" } else { "-1" })
        .attr("aria-label", aria_label)
        .attr("aria-valuemin", 0)
        .attr("aria-valuemax", 100)
        .attr("aria-valuenow", percent(value.saturation()))
        .attr("aria-valuetext", valuetext)
        .element(&element)
        .event("onkeydown", onkeydown)
        .render(HtmlTag::Div, Vec::new(), ())
}
