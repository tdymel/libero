use dioxus::prelude::*;

use crate::{
    hooks::{
        DragMove, DragOptions, DragPoint, ElementHandle, SwipeOptions, listener, use_drag_cancel,
        use_scheduled,
    },
    platform::prefers_reduced_motion,
    sx::{Sx, Variables, sx},
    theme::CssVar,
};

/// How far a swiped box is dragged down, how long it eases there, and how far past its own
/// height it slides to leave the screen.
const SWIPE_DRAG: CssVar = CssVar::new("--lsx-swipe-drag");
const SWIPE_EASE: CssVar = CssVar::new("--lsx-swipe-ease");
pub(crate) const SWIPE_BEYOND: CssVar = CssVar::new("--lsx-swipe-beyond");
/// How long a swiped box slides off or springs back, in ms.
const SWIPE_SLIDE_MS: u64 = 200;

/// The `transform` and `transition` that move a box by [`use_swipe_close`]'s variables.
pub(crate) fn swipe_close_sx() -> Sx {
    sx().transform(format!("translateY({})", SWIPE_DRAG.value_or("0px")))
        .transition(format!("transform {} ease-out", SWIPE_EASE.value_or("0ms")))
}

/// Where a swiped box is.
#[derive(Clone, Copy, Debug, PartialEq)]
enum SwipeDrag {
    /// At rest, or springing back to it.
    Rest,
    /// Under the finger, this many px down.
    Follow(f64),
    /// Sliding off the screen, then closing.
    Off,
}

impl SwipeDrag {
    /// The drag and ease vars; always set, as a renderer keeps a dropped declaration.
    fn variables(self, reduced_motion: bool) -> Variables {
        let ease = match (self, reduced_motion) {
            (SwipeDrag::Follow(_), _) | (_, true) => 0,
            _ => SWIPE_SLIDE_MS,
        };
        let drag = match self {
            SwipeDrag::Rest => "0px".to_string(),
            SwipeDrag::Follow(dy) => format!("{dy}px"),
            SwipeDrag::Off => format!("calc(100% + {})", SWIPE_BEYOND.value_or("0px")),
        };
        Variables::new()
            .with(SWIPE_DRAG, drag)
            .with(SWIPE_EASE, format!("{ease}ms"))
    }
}

/// Whether a release `dy` down and `dx` across swipes the box away: as far as a swipe, mostly down.
fn swiped_off(dx: f64, dy: f64) -> bool {
    dy >= SwipeOptions::default().distance && dy > dx.abs()
}

/// What [`use_swipe_close`] hands out: the strip's press handler, the swiped box's
/// listeners, and the vars for its `style`.
pub(crate) struct SwipeClose {
    pub(crate) onpointerdown: Callback<PointerEvent>,
    pub(crate) listeners: Vec<Attribute>,
    pub(crate) variables: Variables,
}

/// A touch drag down from a strip moves the `capture` box along; let go far enough, and
/// `allow` (unset, always) lets it, it slides off and `onclose` runs, else it springs back
/// (3957). Give the box [`swipe_close_sx`].
pub(crate) fn use_swipe_close(
    capture: ElementHandle,
    allow: Option<Callback<(), bool>>,
    onclose: Callback<()>,
) -> SwipeClose {
    let mut drag = use_signal(|| SwipeDrag::Rest);
    let mut last = use_hook(|| CopyValue::new(DragPoint { x: 0.0, y: 0.0 }));
    // Blitz fires no `transitionend`: a timer closes once the slide is over.
    let slid = use_scheduled(move |_| {
        drag.set(SwipeDrag::Rest);
        onclose.call(());
    });
    let drags = use_drag_cancel(
        DragOptions {
            capture,
            onstart: use_callback(move |_| last.set(DragPoint { x: 0.0, y: 0.0 })),
            onmove: use_callback(move |step: DragMove| {
                let delta = step.delta();
                last.set(delta);
                let next = SwipeDrag::Follow(delta.y.max(0.0));
                if *drag.peek() != next {
                    drag.set(next);
                }
            }),
            onend: use_callback(move |()| {
                let delta = *last.peek();
                if !swiped_off(delta.x, delta.y) || allow.is_some_and(|allow| !allow.call(())) {
                    drag.set(SwipeDrag::Rest);
                } else if prefers_reduced_motion() {
                    drag.set(SwipeDrag::Rest);
                    onclose.call(());
                } else {
                    drag.set(SwipeDrag::Off);
                    slid.after(SWIPE_SLIDE_MS);
                }
            }),
        },
        use_callback(move |()| drag.set(SwipeDrag::Rest)),
    );
    let dragging = drags.dragging;
    let listeners = vec![
        listener("onpointermove", move |event: PointerEvent| {
            // Blitz pans the scrolling box unless the move is prevented.
            if *dragging.peek() {
                event.prevent_default();
            }
            drags.onpointermove.call(event);
        }),
        listener("onpointerup", move |event: PointerEvent| {
            drags.onpointerup.call(event)
        }),
        listener("onpointercancel", move |event: PointerEvent| {
            drags.onpointercancel.call(event)
        }),
    ];
    // Touch and pen only: a mouse has Escape and the box's own close (WCAG 2.5.1).
    let onpointerdown = use_callback(move |event: PointerEvent| {
        if event.is_primary() && event.pointer_type() != "mouse" {
            drags.onpointerdown.call(event);
        }
    });
    SwipeClose {
        onpointerdown,
        listeners,
        variables: drag().variables(prefers_reduced_motion()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_swipe_down_slides_the_box_off() {
        assert!(swiped_off(0.0, 48.0));
        assert!(!swiped_off(0.0, 47.0));
        assert!(!swiped_off(60.0, 50.0));
        assert!(!swiped_off(0.0, -80.0));
    }

    #[test]
    fn the_box_eases_off_the_finger_only_without_reduced_motion() {
        let style = |drag: SwipeDrag, reduced| drag.variables(reduced).render();
        assert_eq!(
            style(SwipeDrag::Follow(12.0), false),
            "--lsx-swipe-drag:12px;--lsx-swipe-ease:0ms;"
        );
        assert_eq!(
            style(SwipeDrag::Rest, false),
            "--lsx-swipe-drag:0px;--lsx-swipe-ease:200ms;"
        );
        assert!(style(SwipeDrag::Off, false).contains("calc(100% + "));
        assert!(style(SwipeDrag::Off, true).ends_with("--lsx-swipe-ease:0ms;"));
    }
}
