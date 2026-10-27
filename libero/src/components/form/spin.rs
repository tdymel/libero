//! The gestures a spin column turns by: wheel, a vertical drag, a press on a neighbour, and
//! the arrow and Page keys. A digital clock's columns and a number's spinner share them.

use dioxus::prelude::*;

use crate::{
    hooks::{DragMove, DragOptions, DragStart, ElementHandle, use_drag_cancel},
    platform,
};

/// The wheel travel, in pixels, that turns a column one step.
const WHEEL_STEP: f64 = 40.0;
/// The drag travel, in pixels, that turns a column one step.
const DRAG_STEP: f64 = 24.0;

/// What a gesture asks of the column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Turn<A> {
    /// Steps from the current value: a key, a wheel notch, a press on a neighbour.
    By(isize),
    /// Steps from `A`, the value a drag started on; `0` is back on it.
    From(A, isize),
}

/// The steps an arrow or Page key takes, `page` for a Page key.
pub(crate) fn key_steps(key: &Key, page: usize) -> Option<isize> {
    match key {
        Key::Named(NamedKey::ArrowUp) => Some(1),
        Key::Named(NamedKey::ArrowDown) => Some(-1),
        Key::Named(NamedKey::PageUp) => Some(page as isize),
        Key::Named(NamedKey::PageDown) => Some(-(page as isize)),
        _ => None,
    }
}

/// The travel left over and the steps a wheel event takes: a notch is one step,
/// a trackpad's small deltas add up to one, a page delta (`paged`) takes its whole travel.
pub(crate) fn wheel_steps(held: f64, travel: f64, paged: bool) -> (f64, isize) {
    // A turn back drops what was held the other way.
    let total = if held * travel < 0.0 {
        travel
    } else {
        held + travel
    };
    match total.abs() >= WHEEL_STEP {
        // Down the page brings the value below up: the next one.
        true if paged => (0.0, (total / WHEEL_STEP).round() as isize),
        true => (0.0, total.signum() as isize),
        false => (total, 0),
    }
}

/// The column's listeners; `press` goes on each neighbour's `onpointerdown`.
#[derive(Clone, Copy)]
pub(crate) struct SpinGestures {
    pub(crate) onwheel: Callback<Event<WheelData>>,
    pub(crate) onpointerdown: Callback<Event<PointerData>>,
    pub(crate) onpointermove: Callback<Event<PointerData>>,
    pub(crate) onpointerup: Callback<Event<PointerData>>,
    pub(crate) onpointercancel: Callback<Event<PointerData>>,
    pressed: CopyValue<Option<isize>>,
}

impl SpinGestures {
    /// A press on the neighbour `delta` steps away; `None` for an empty or disabled one.
    pub(crate) fn press(self, delta: Option<isize>) {
        let mut pressed = self.pressed;
        pressed.set(delta);
    }
}

/// Wheel, drag and neighbour presses on the column `handle`, which shows `anchor` now.
/// Disabled, none of them turns it, and the wheel scrolls the page.
pub(crate) fn use_spin_gestures<A: Copy + 'static>(
    handle: ElementHandle,
    anchor: A,
    disabled: bool,
    page: usize,
    onturn: Callback<Turn<A>>,
) -> SpinGestures {
    let mut wheel = use_hook(|| CopyValue::new(0.0_f64));
    // Where a drag started and the steps it took, and the neighbour a press landed on.
    let mut drag_from = use_hook(|| CopyValue::new(None::<(A, isize)>));
    let mut pressed = use_hook(|| CopyValue::new(None::<isize>));

    let onwheel = use_callback(move |event: Event<WheelData>| {
        if disabled {
            return;
        }
        event.prevent_default();
        let travel = platform::wheel_travel_y(&event.data(), WHEEL_STEP, page as f64);
        let paged = platform::wheel_is_paged(&event.data());
        let (total, delta) = wheel_steps(*wheel.peek(), travel, paged);
        wheel.set(total);
        if delta != 0 {
            onturn.call(Turn::By(delta));
        }
    });
    let onstart = use_callback(move |_: DragStart| drag_from.set(Some((anchor, 0))));
    let onmove = use_callback(move |step: DragMove| {
        let Some((from, taken)) = *drag_from.peek() else {
            return;
        };
        // Up brings the value below into the middle, as a wheel's rim.
        let steps = (-step.delta().y / DRAG_STEP).round() as isize;
        if steps == taken {
            return;
        }
        drag_from.set(Some((from, steps)));
        onturn.call(Turn::From(from, steps));
    });
    let onend = use_callback(move |()| {
        let dragged = drag_from.peek().is_some_and(|(_, taken)| taken != 0);
        drag_from.set(None);
        if let Some(delta) = pressed.take().filter(|_| !dragged) {
            onturn.call(Turn::By(delta));
        }
    });
    // A revoked pointer steps nothing: the press and the drag are dropped.
    let oncancel = use_callback(move |()| {
        drag_from.set(None);
        pressed.set(None);
    });
    let drag = use_drag_cancel(
        DragOptions {
            capture: handle,
            onstart,
            onmove,
            onend,
        },
        oncancel,
    );
    // Disabled, no drag: its `prevent_default` would cancel the press's focus (3883).
    let onpointerdown = use_callback(move |event: Event<PointerData>| {
        if !disabled {
            drag.onpointerdown.call(event);
        }
        // A press that started no drag (a right-click) never ends: it must not step later.
        if !*drag.dragging.peek() {
            pressed.set(None);
        }
    });
    SpinGestures {
        onwheel,
        onpointerdown,
        onpointermove: drag.onpointermove,
        onpointerup: drag.onpointerup,
        onpointercancel: drag.onpointercancel,
        pressed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wheel_notch_takes_one_step_and_small_deltas_add_up() {
        assert_eq!(wheel_steps(0.0, 100.0, false), (0.0, 1));
        assert_eq!(wheel_steps(0.0, -360.0, false), (0.0, -1));
        assert_eq!(wheel_steps(0.0, 15.0, false), (15.0, 0));
        assert_eq!(wheel_steps(30.0, 15.0, false), (0.0, 1));
        assert_eq!(wheel_steps(30.0, -15.0, false), (-15.0, 0));
    }

    #[test]
    fn a_page_delta_takes_the_steps_of_a_page() {
        assert_eq!(wheel_steps(0.0, 120.0, true), (0.0, 3));
        assert_eq!(wheel_steps(0.0, -400.0, true), (0.0, -10));
    }
}
