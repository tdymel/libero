use dioxus::prelude::*;

use crate::{
    hooks::{
        DragMove, DragOptions, DragPoint, ElementHandle, PopoverHandle, SwipeOptions, listener,
        use_drag_cancel, use_scheduled, use_subscription_slot,
    },
    platform::{
        ElementApi, SCROLL_MARGIN_BOTTOM_VAR, ScrollSubscription, document, on_viewport_resize,
        prefers_reduced_motion, visible_band,
    },
    sx::{FORCED_COLORS, Sx, sx},
    theme::{Color, ColorShade, ColorValue, CssVar, Size},
    utils::bump,
};

/// Below the theme's `sm`, where a dropdown becomes the phone sheet.
pub(crate) fn narrow_query() -> String {
    format!("not all and (min-width: {})", Size::Sm.breakpoint_value())
}

/// A portaled dropdown's phone layout: below `sm`, a sheet full width at the screen's foot.
/// Its rules override the popover's class placement, which keeps measuring but no longer places.
pub(crate) fn phone_sheet_sx() -> Sx {
    sx().selector(SHEET_HANDLE, sx().display("none")).media(
        narrow_query(),
        sx().left(SHEET_LEFT.value_or("0px"))
            .right(SHEET_RIGHT.value_or("0px"))
            .top("auto")
            .bottom(SHEET_FOOT.value_or("0px"))
            .width("auto")
            .min_width("0")
            .max_width("100%")
            .max_height(format!("min(70vh, {})", SHEET_ROOM.value_or("70vh")))
            .border_bottom_left_radius("0")
            .border_bottom_right_radius("0")
            .padding_bottom("calc(4px + env(safe-area-inset-bottom, 0px))")
            // Follows a swipe down from the strip, slides off past the keyboard's foot or springs back (3957).
            .and(swipe_close_sx())
            .var(SWIPE_BEYOND, SHEET_FOOT.value_or("0px"))
            // Tracks the visible viewport as a phone's URL bar slides in and out.
            .supports(
                "(height: 1dvh)",
                sx().max_height(format!("min(70dvh, {})", SHEET_ROOM.value_or("70dvh"))),
            )
            // The swipe strip over the rows, its grab bar centred, as a bottom `Drawer`'s.
            .selector(
                SHEET_HANDLE,
                sx().display("block")
                    .position("relative")
                    .flex("none")
                    .height("20px")
                    .margin_top("-4px")
                    .touch_action("none"),
            )
            .selector(
                format!("{SHEET_HANDLE}::before"),
                sx().content("\"\"")
                    .position("absolute")
                    .top("50%")
                    .left("50%")
                    .width("2rem")
                    .height("4px")
                    .transform("translate(-50%, -50%)")
                    .border_radius("999px")
                    .background(ColorValue::Shade(Color::Muted, ColorShade::S6).value())
                    .media(FORCED_COLORS, sx().background("CanvasText")),
            ),
    )
}

const SHEET_HANDLE: &str = "& > [data-slot='sheet-handle']";

/// The sheet's swipe strip, first in the box, the box's listeners that follow the finger,
/// and the inline vars for the box's `style` that move it.
pub(crate) struct SheetSwipe {
    pub(crate) strip: Element,
    pub(crate) listeners: Vec<Attribute>,
    pub(crate) style: String,
}

/// A touch swipe down from the sheet's top closes it, as a bottom `Drawer`'s (Maintainer 2026-10-22).
pub(crate) fn use_sheet_swipe(capture: ElementHandle, onclose: Callback<()>) -> SheetSwipe {
    let SwipeClose {
        onpointerdown,
        listeners,
        style,
    } = use_swipe_close(capture, onclose);
    let strip = rsx! {
        div {
            "data-slot": "sheet-handle",
            aria_hidden: "true",
            onpointerdown: move |event| onpointerdown.call(event),
        }
    };
    SheetSwipe {
        strip,
        listeners,
        style,
    }
}

/// How far a swiped box is dragged down, how long it eases there, and how far past its own
/// height it slides to leave the screen.
const SWIPE_DRAG: CssVar = CssVar::new("--lsx-swipe-drag");
const SWIPE_EASE: CssVar = CssVar::new("--lsx-swipe-ease");
const SWIPE_BEYOND: CssVar = CssVar::new("--lsx-swipe-beyond");
/// How long a swiped box slides off or springs back, in ms.
const SWIPE_SLIDE_MS: u64 = 200;

/// The `transform` and `transition` that move a box by [`use_swipe_close`]'s style.
pub(crate) fn swipe_close_sx() -> Sx {
    sx().transform(format!("translateY({})", SWIPE_DRAG.value_or("0px")))
        .transition(format!("transform {} ease-out", SWIPE_EASE.value_or("0ms")))
}

/// Where a swiped box is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SwipeDrag {
    /// At rest, or springing back to it.
    Rest,
    /// Under the finger, this many px down.
    Follow(f64),
    /// Sliding off the screen, then closing.
    Off,
}

impl SwipeDrag {
    /// The drag and ease vars; always printed, as a renderer keeps a dropped declaration.
    pub(crate) fn style(self, reduced_motion: bool) -> String {
        let ease = match (self, reduced_motion) {
            (SwipeDrag::Follow(_), _) | (_, true) => 0,
            _ => SWIPE_SLIDE_MS,
        };
        let drag = match self {
            SwipeDrag::Rest => "0px".to_string(),
            SwipeDrag::Follow(dy) => format!("{dy}px"),
            SwipeDrag::Off => format!("calc(100% + {})", SWIPE_BEYOND.value_or("0px")),
        };
        format!(
            "{}:{drag};{}:{ease}ms;",
            SWIPE_DRAG.name(),
            SWIPE_EASE.name()
        )
    }
}

/// Whether a release `dy` down and `dx` across swipes the box away: as far as a swipe, mostly down.
pub(crate) fn swiped_off(dx: f64, dy: f64) -> bool {
    dy >= SwipeOptions::default().distance && dy > dx.abs()
}

/// What [`use_swipe_close`] hands out: the strip's press handler, the swiped box's
/// listeners, and the vars for its `style`.
pub(crate) struct SwipeClose {
    pub(crate) onpointerdown: Callback<PointerEvent>,
    pub(crate) listeners: Vec<Attribute>,
    pub(crate) style: String,
}

/// A touch drag down from a strip moves the `capture` box along; let go far enough, it slides
/// off and `onclose` runs, else it springs back (3957). Give the box [`swipe_close_sx`].
pub(crate) fn use_swipe_close(capture: ElementHandle, onclose: Callback<()>) -> SwipeClose {
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
                if !swiped_off(delta.x, delta.y) {
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
    // Touch and pen only: a mouse has Escape and the field's own close (WCAG 2.5.1).
    let onpointerdown = use_callback(move |event: PointerEvent| {
        if event.is_primary() && event.pointer_type() != "mouse" {
            drags.onpointerdown.call(event);
        }
    });
    SwipeClose {
        onpointerdown,
        listeners,
        style: drag().style(prefers_reduced_motion()),
    }
}

/// The sheet's foot above, and its height within, the visible part of the page.
const SHEET_FOOT: CssVar = CssVar::new("--lsx-sheet-foot");
const SHEET_ROOM: CssVar = CssVar::new("--lsx-sheet-room");
/// Pinch-zoomed and panned sideways, the sheet spans the visible band only (2219).
const SHEET_LEFT: CssVar = CssVar::new("--lsx-sheet-left");
const SHEET_RIGHT: CssVar = CssVar::new("--lsx-sheet-right");

/// A phone browser's keyboard shrinks only the visual viewport (2173): the open sheet's foot
/// and room follow it, as inline vars for its `style`.
pub(crate) fn use_sheet_room(sheet: ElementHandle, placed: bool) -> String {
    let mut room = use_signal(|| None::<SheetBand>);
    let tick = use_signal(|| 0u64);
    let resizing = use_subscription_slot::<dyn ScrollSubscription>();
    use_effect(use_reactive!(|placed| {
        let _ = tick();
        if !placed {
            resizing.clear();
            return;
        }
        if !resizing.is_some() {
            resizing.set(on_viewport_resize(Box::new(move || bump(tick))));
        }
        let Some(document) = document() else {
            return;
        };
        let (offset, size, viewport, band) = (
            sheet.client_offset(),
            sheet.dimensions(),
            document.viewport(),
            visible_band(),
        );
        spawn(async move {
            let (Ok((_, top)), Ok(size), Ok(viewport), Ok(band)) =
                (offset.await, size.await, viewport.await, band.await)
            else {
                return;
            };
            // Placed at the foot, the sheet's bottom plus its foot is the layout viewport's.
            let foot = room.peek().map_or(0.0, |band| band.foot);
            let layout_bottom = top + size.height + foot;
            let next = Some(SheetBand {
                foot: (layout_bottom - viewport.height).max(0.0),
                room: viewport.height - band.top,
                left: band.left,
                right: (viewport.width - band.right).max(0.0),
            });
            if *room.peek() != next {
                room.set(next);
            }
        });
    }));
    room().map_or(String::new(), |band| {
        format!(
            "{}:{}px;{}:{}px;{}:{}px;{}:{}px;",
            SHEET_FOOT.name(),
            band.foot,
            SHEET_ROOM.name(),
            band.room,
            SHEET_LEFT.name(),
            band.left,
            SHEET_RIGHT.name(),
            band.right
        )
    })
}

/// The visible part of the page the sheet keeps to, in layout px.
#[derive(Clone, Copy, PartialEq)]
struct SheetBand {
    foot: f64,
    room: f64,
    left: f64,
    right: f64,
}

/// The trigger's `scroll-margin-bottom` (and Blitz's twin var) clearing the sheet.
/// Always printed: a renderer never removes a declaration that stops being printed.
pub(crate) fn sheet_clearance_style(clearance: f64) -> String {
    format!("scroll-margin-bottom:{clearance}px;{SCROLL_MARGIN_BOTTOM_VAR}:{clearance}px;")
}

/// The sheet may cover the field (WCAG 2.4.11): once placed over it, the trigger's
/// `scroll-margin-bottom` clears the sheet plus the helper and error under the trigger.
pub(crate) fn use_sheet_clearance(
    anchor: ElementHandle,
    field: Option<ElementHandle>,
    popover: PopoverHandle,
    opened: bool,
    gap: f64,
) -> f64 {
    let mut clearance = use_signal(|| 0.0);
    let placed = popover.placed();
    let sheet = *popover.floating();
    let mut live = use_signal(|| false);
    use_effect(use_reactive!(|(opened, placed)| {
        live.set(opened && placed);
        if !(opened && placed) {
            if *clearance.peek() != 0.0 {
                clearance.set(0.0);
            }
            return;
        }
        spawn(async move {
            let (Ok((_, top)), Ok(size), Ok((_, sheet_top)), Ok(sheet_size)) = (
                anchor.client_offset().await,
                anchor.dimensions().await,
                sheet.client_offset().await,
                sheet.dimensions().await,
            ) else {
                return;
            };
            // A placed popover keeps its placed top and covers what a dropdown covers; only
            // the sheet, moved by its narrow rules, covers the field.
            if popover
                .placed_top()
                .is_none_or(|placed_top| (placed_top - sheet_top).abs() < 1.0)
            {
                return;
            }
            let trigger_bottom = top + size.height;
            let mut bottom = trigger_bottom;
            if let Some(field) = field
                && let (Ok((_, field_top)), Ok(field_size)) =
                    (field.client_offset().await, field.dimensions().await)
            {
                bottom = bottom.max(field_top + field_size.height);
            }
            // The list may have closed during the measures above.
            if *live.peek() && sheet_top < bottom && sheet_top + sheet_size.height > top {
                clearance.set(sheet_size.height + gap + bottom - trigger_bottom);
            }
        });
    }));
    use_effect(move || {
        if clearance() > 0.0 {
            let _ = anchor.scroll_into_view(false);
        }
    });
    clearance()
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
        assert_eq!(
            SwipeDrag::Follow(12.0).style(false),
            "--lsx-swipe-drag:12px;--lsx-swipe-ease:0ms;"
        );
        assert_eq!(
            SwipeDrag::Rest.style(false),
            "--lsx-swipe-drag:0px;--lsx-swipe-ease:200ms;"
        );
        assert!(SwipeDrag::Off.style(false).contains("calc(100% + "));
        assert!(
            SwipeDrag::Off
                .style(true)
                .ends_with("--lsx-swipe-ease:0ms;")
        );
    }
}
