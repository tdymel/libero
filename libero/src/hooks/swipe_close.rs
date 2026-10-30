use std::cell::Cell;

use dioxus::core::{Attribute, AttributeValue};
use dioxus::prelude::*;

use crate::{
    hooks::{
        DragPoint, ElementHandle, SwipeOptions, listener, use_scheduled, use_subscription_slot,
    },
    platform::{
        self, ElementApi, SWIPE_CLOSE_ATTR, ScrollSubscription, SheetPan, hold_sheet_pan,
        prefers_reduced_motion, swipe_press,
    },
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
/// Blitz starts a touch pan once the finger is more than 2px from the press.
const PAN_SLOP: f64 = 2.0;

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
    /// Where a move `dy` down puts the box; `None` while it slides off.
    fn follow(self, dy: f64) -> Option<SwipeDrag> {
        match self {
            SwipeDrag::Off => None,
            _ => Some(SwipeDrag::Follow(dy.max(0.0))),
        }
    }

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

/// A press the pointer path follows: Blitz, which runs no script, or a pen on the strip.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Press {
    /// Not yet past the slop; `grip` when it pressed the strip.
    Wait {
        pointer_id: i32,
        start: DragPoint,
        grip: bool,
    },
    /// A scroller under it scrolls; it hands over back at the top, still moving down.
    Scroll { pointer_id: i32, last_y: f64 },
    /// The box follows from `base`.
    Drag { pointer_id: i32, base: DragPoint },
}

impl Press {
    fn pointer_id(self) -> i32 {
        match self {
            Press::Wait { pointer_id, .. }
            | Press::Scroll { pointer_id, .. }
            | Press::Drag { pointer_id, .. } => pointer_id,
        }
    }

    /// The press after a move to `at`; `None` once sideways, a control's or a scroller's.
    fn step(self, at: DragPoint, scrolled: impl Fn() -> bool) -> Option<Press> {
        match self {
            Press::Wait {
                pointer_id,
                start,
                grip,
            } => {
                let (dx, dy) = (at.x - start.x, at.y - start.y);
                if dx.abs().max(dy.abs()) <= PAN_SLOP {
                    Some(self)
                } else if dx.abs() >= dy.abs() {
                    None
                } else if dy > 0.0 && (grip || !scrolled()) {
                    Some(Press::Drag {
                        pointer_id,
                        base: start,
                    })
                } else {
                    Some(Press::Scroll {
                        pointer_id,
                        last_y: at.y,
                    })
                }
            }
            Press::Scroll { pointer_id, last_y } if at.y > last_y && !scrolled() => {
                Some(Press::Drag {
                    pointer_id,
                    base: DragPoint { x: at.x, y: last_y },
                })
            }
            Press::Scroll { pointer_id, .. } => Some(Press::Scroll {
                pointer_id,
                last_y: at.y,
            }),
            Press::Drag { .. } => Some(self),
        }
    }
}

thread_local! {
    static NEXT_MARK: Cell<u64> = const { Cell::new(0) };
}

/// What [`use_swipe_close`] hands out: the strip's press handler, the swiped box's
/// attributes, and the vars for its `style`.
pub(crate) struct SwipeClose {
    pub(crate) onpointerdown: Callback<PointerEvent>,
    pub(crate) listeners: Vec<Attribute>,
    pub(crate) variables: Variables,
}

/// A touch drag down anywhere on the `capture` box moves it along, as an Android bottom
/// sheet (4994): from its strip always, from elsewhere unless an inner control owns the drag
/// or a scroller there is scrolled down, which scrolls first and then hands over. Let go far
/// enough, and `allow` (unset, always) lets it, it slides off and `onclose` runs, else it
/// springs back (3957). Give the box [`swipe_close_sx`] and its strip `data-swipe-handle`.
/// A pen on the web swipes from the body only where it fires touch events (Chromium on Android,
/// Safari); elsewhere the browser's pan cancels its pointer, so it swipes from the strip (5054).
pub(crate) fn use_swipe_close(
    capture: ElementHandle,
    allow: Option<Callback<(), bool>>,
    onclose: Callback<()>,
) -> SwipeClose {
    let mut drag = use_signal(|| SwipeDrag::Rest);
    let mut last = use_hook(|| CopyValue::new(DragPoint { x: 0.0, y: 0.0 }));
    let mark = use_hook(|| NEXT_MARK.replace(NEXT_MARK.get() + 1));
    // Blitz fires no `transitionend`: a timer closes once the slide is over.
    let slid = use_scheduled(move |_| {
        drag.set(SwipeDrag::Rest);
        onclose.call(());
    });
    // Sliding off, the box ignores a new touch: it closes all the same (5012).
    let sliding = move || *drag.peek() == SwipeDrag::Off;
    let follow = use_callback(move |delta: DragPoint| {
        let Some(next) = drag.peek().follow(delta.y) else {
            return;
        };
        last.set(delta);
        if *drag.peek() != next {
            drag.set(next);
        }
    });
    let rest = use_callback(move |()| {
        if !sliding() && *drag.peek() != SwipeDrag::Rest {
            drag.set(SwipeDrag::Rest);
        }
    });
    let finish = use_callback(move |()| {
        if sliding() {
            return;
        }
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
    });

    // The web and a WebView: a script holds the touch, so a scroller can hand it over.
    let holding = use_subscription_slot::<dyn ScrollSubscription>();
    let mounted = capture.is_mounted();
    use_effect(use_reactive!(|mounted| {
        if !mounted {
            holding.clear();
        } else if !holding.is_some() {
            holding.set(hold_sheet_pan(
                mark,
                Box::new(move |pan| match pan {
                    SheetPan::Move { dx, dy } => follow.call(DragPoint { x: dx, y: dy }),
                    SheetPan::End => finish.call(()),
                    SheetPan::Cancel => rest.call(()),
                }),
            ));
        }
    }));

    // Blitz, and a pen on the strip elsewhere (one firing no touch events): the box's pointer listeners.
    let mut press = use_hook(|| CopyValue::new(None::<Press>));
    let mut gripped = use_hook(|| CopyValue::new(None::<i32>));
    let release = use_callback(move |(event, ended): (PointerEvent, bool)| {
        let Some(current) = *press.peek() else {
            return;
        };
        if current.pointer_id() != event.pointer_id() {
            return;
        }
        press.set(None);
        if let Press::Drag { .. } = current {
            if ended {
                finish.call(());
            } else {
                rest.call(());
            }
        }
    });
    let up = use_callback(move |event: PointerEvent| release.call((event, true)));
    // The moves Blitz reports from outside the box.
    let track = use_callback(move |event: PointerEvent| {
        if let Some(Press::Drag { pointer_id, base }) = *press.peek()
            && pointer_id == event.pointer_id()
        {
            let at = event.client_coordinates();
            follow.call(DragPoint {
                x: at.x - base.x,
                y: at.y - base.y,
            });
        }
    });
    let onmove = use_callback(move |event: PointerEvent| {
        let Some(current) = *press.peek() else {
            return;
        };
        if current.pointer_id() != event.pointer_id() {
            return;
        }
        let at = event.client_coordinates();
        let at = DragPoint { x: at.x, y: at.y };
        let next = current.step(at, || {
            swipe_press(&event, mark).is_some_and(|hit| hit.scrolled)
        });
        if next != Some(current) {
            press.set(next);
        }
        let Some(Press::Drag { base, .. }) = next else {
            return;
        };
        // Blitz pans the scrolling box unless the move is prevented.
        event.prevent_default();
        if !matches!(current, Press::Drag { .. })
            && capture.set_pointer_capture(event.pointer_id()).is_err()
            && let Some(within) = capture.mounted()
        {
            platform::follow_pointer(&event, &within, track, up);
        }
        follow.call(DragPoint {
            x: at.x - base.x,
            y: at.y - base.y,
        });
    });
    let listeners = vec![
        Attribute::new(
            SWIPE_CLOSE_ATTR,
            AttributeValue::Text(mark.to_string()),
            None,
            false,
        ),
        listener("onpointerdown", move |event: PointerEvent| {
            let grip = gripped.replace(None) == Some(event.pointer_id());
            // Touch and pen only: a mouse has Escape and the box's own close (WCAG 2.5.1).
            if !event.is_primary() || event.pointer_type() == "mouse" {
                return;
            }
            let takes = !sliding()
                && match swipe_press(&event, mark) {
                    Some(hit) => hit.shown && !hit.refused,
                    None => grip && event.pointer_type() == "pen",
                };
            if takes {
                let at = event.client_coordinates();
                press.set(Some(Press::Wait {
                    pointer_id: event.pointer_id(),
                    start: DragPoint { x: at.x, y: at.y },
                    grip,
                }));
            }
        }),
        listener("onpointermove", move |event: PointerEvent| {
            onmove.call(event)
        }),
        listener("onpointerup", move |event: PointerEvent| up.call(event)),
        listener("onpointercancel", move |event: PointerEvent| {
            release.call((event, false))
        }),
    ];
    // Fires before the box's own, which it bubbles to. A press on the strip moves no focus:
    // a blur would close a field's sheet.
    let onpointerdown = use_callback(move |event: PointerEvent| {
        if event.is_primary() && event.pointer_type() != "mouse" {
            event.prevent_default();
        }
        gripped.set(Some(event.pointer_id()));
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

    #[test]
    fn a_box_sliding_off_follows_no_new_touch() {
        assert_eq!(SwipeDrag::Rest.follow(-5.0), Some(SwipeDrag::Follow(0.0)));
        assert_eq!(
            SwipeDrag::Follow(4.0).follow(9.0),
            Some(SwipeDrag::Follow(9.0))
        );
        assert_eq!(SwipeDrag::Off.follow(30.0), None);
    }

    const START: DragPoint = DragPoint { x: 100.0, y: 100.0 };

    fn wait(grip: bool) -> Press {
        Press::Wait {
            pointer_id: 1,
            start: START,
            grip,
        }
    }

    fn at(x: f64, y: f64) -> DragPoint {
        DragPoint { x, y }
    }

    #[test]
    fn a_press_at_the_top_drags_once_it_goes_down() {
        assert_eq!(
            wait(false).step(at(101.0, 102.0), || false),
            Some(wait(false))
        );
        assert_eq!(
            wait(false).step(at(101.0, 106.0), || false),
            Some(Press::Drag {
                pointer_id: 1,
                base: START
            })
        );
        // Sideways is a horizontal scroller's or a slider's.
        assert_eq!(wait(false).step(at(108.0, 104.0), || false), None);
    }

    #[test]
    fn a_scrolled_body_scrolls_first_and_hands_over_at_its_top() {
        let scrolling = wait(false).step(at(100.0, 106.0), || true);
        assert_eq!(
            scrolling,
            Some(Press::Scroll {
                pointer_id: 1,
                last_y: 106.0
            })
        );
        let still = scrolling.and_then(|press| press.step(at(100.0, 120.0), || true));
        assert_eq!(
            still,
            Some(Press::Scroll {
                pointer_id: 1,
                last_y: 120.0
            })
        );
        assert_eq!(
            still.and_then(|press| press.step(at(100.0, 130.0), || false)),
            Some(Press::Drag {
                pointer_id: 1,
                base: at(100.0, 120.0)
            })
        );
    }

    #[test]
    fn the_strip_drags_even_over_a_scrolled_body() {
        assert!(matches!(
            wait(true).step(at(100.0, 106.0), || true),
            Some(Press::Drag { .. })
        ));
    }
}
