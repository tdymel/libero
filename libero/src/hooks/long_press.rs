use dioxus::{html::input_data::MouseButton, prelude::*};

use super::use_timeout;

/// What [`use_long_press`] takes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LongPressOptions {
    /// How long the pointer must stay down, in milliseconds.
    pub ms: u64,
    /// How far, in CSS px, the pointer may move before the press is a drag.
    pub move_tolerance: f64,
}

impl Default for LongPressOptions {
    fn default() -> Self {
        Self {
            ms: 400,
            move_tolerance: 10.0,
        }
    }
}

/// Handlers to spread onto the pressed element.
#[derive(Clone, Copy)]
pub struct LongPress {
    /// Starts the press timer.
    pub onpointerdown: Callback<PointerEvent>,
    /// Drops the press once the pointer moves past the tolerance.
    pub onpointermove: Callback<PointerEvent>,
    /// Drops a press not fired yet.
    pub onpointerup: Callback<PointerEvent>,
    /// Drops a press not fired yet.
    pub onpointerleave: Callback<PointerEvent>,
    /// Drops the press, as a touch that starts to scroll does.
    pub onpointercancel: Callback<PointerEvent>,
    /// Suppresses the browser's own long-press menu, only after the press fired.
    pub oncontextmenu: Callback<MouseEvent>,
    /// Call it first in your own `onclick`: it answers `true` for the click a
    /// fired press ends with, which you should skip, and stops that click
    /// from reaching ancestors.
    pub onclick: Callback<MouseEvent, bool>,
    /// `true` from the pointer going down until the press fires or is dropped.
    pub pressing: ReadSignal<bool>,
}

#[derive(Clone, Copy)]
struct Press {
    pointer_id: i32,
    x: f64,
    y: f64,
}

/// Calls `on_long_press` once a pointer stays down for `options.ms`.
///
/// The press is dropped when the pointer moves beyond the tolerance, leaves,
/// is released or cancelled (a touch that starts to scroll), or a second finger
/// lands. A tap that ends first runs no callback and keeps its click.
/// `pressing` is `true` while a press is held and not yet fired, for a hold cue.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::{LongPressOptions, use_long_press};
/// # fn app() -> Element {
/// let mut pressed = use_signal(|| false);
/// let press = use_long_press(
///     Callback::new(move |()| pressed.set(true)),
///     LongPressOptions::default(),
/// );
///
/// rsx! {
///     button {
///         onpointerdown: move |event| press.onpointerdown.call(event),
///         onpointermove: move |event| press.onpointermove.call(event),
///         onpointerup: move |event| press.onpointerup.call(event),
///         onpointerleave: move |event| press.onpointerleave.call(event),
///         onpointercancel: move |event| press.onpointercancel.call(event),
///         oncontextmenu: move |event| press.oncontextmenu.call(event),
///         onclick: move |event| {
///             if !press.onclick.call(event) {
///                 // A plain click.
///             }
///         },
///         "Hold me"
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/hooks/use-long-press>
pub fn use_long_press(on_long_press: Callback, options: LongPressOptions) -> LongPress {
    let LongPressOptions { ms, move_tolerance } = options;
    let mut press = use_signal(|| None::<Press>);
    let mut fired = use_signal(|| false);
    let pressing = use_memo(move || press.read().is_some());
    let timeout = use_timeout(
        move || {
            if press.peek().is_some() {
                press.set(None);
                fired.set(true);
                on_long_press.call(());
            }
        },
        ms,
    );
    let cancel = use_callback(move |()| {
        timeout.stop();
        if press.peek().is_some() {
            press.set(None);
        }
    });

    let onpointerdown = use_callback(move |event: PointerEvent| {
        if *fired.peek() {
            fired.set(false);
        }
        // A second finger makes it a pinch or a scroll, not a press.
        if !event.is_primary() || press.peek().is_some() {
            cancel.call(());
            return;
        }
        if matches!(event.trigger_button(), Some(button) if button != MouseButton::Primary) {
            return;
        }
        let at = event.client_coordinates();
        press.set(Some(Press {
            pointer_id: event.pointer_id(),
            x: at.x,
            y: at.y,
        }));
        timeout.start();
    });
    let onpointermove = use_callback(move |event: PointerEvent| {
        let Some(start) = *press.peek() else {
            return;
        };
        if event.pointer_id() != start.pointer_id {
            return;
        }
        let at = event.client_coordinates();
        if (at.x - start.x).hypot(at.y - start.y) > move_tolerance {
            cancel.call(());
        }
    });
    let end = use_callback(move |_: PointerEvent| cancel.call(()));

    let oncontextmenu = use_callback(move |event: MouseEvent| {
        if *fired.peek() {
            event.prevent_default();
        }
    });
    let onclick = use_callback(move |event: MouseEvent| {
        let ended_press = *fired.peek();
        if ended_press {
            fired.set(false);
            event.prevent_default();
            event.stop_propagation();
        }
        ended_press
    });

    LongPress {
        onpointerdown,
        onpointermove,
        onpointerup: end,
        onpointerleave: end,
        onpointercancel: end,
        oncontextmenu,
        onclick,
        pressing: pressing.into(),
    }
}
