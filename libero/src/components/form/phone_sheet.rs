use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    hooks::{
        ElementHandle, PopoverHandle, SWIPE_BEYOND, SwipeClose, in_open_popup, swipe_close_sx,
        use_open_popups, use_subscription_slot, use_swipe_close,
    },
    platform::{
        ContentSubscription, ElementApi, PressSubscription, SCROLL_MARGIN_BOTTOM_VAR,
        ScrollSubscription, document, element_contains, observe_resize, on_viewport_resize,
        swallow_outside_clicks, visible_band,
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
            // A swipe in the sheet must not scroll the page behind it (4610).
            .with("overscroll-behavior", "contain")
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
                // Sticky: a sheet that is itself the scroller (`Menu`) keeps its strip in view.
                sx().display("block")
                    .position("sticky")
                    // Stuck where it sits unscrolled: the box's padding offsets it.
                    .top("-4px")
                    .z_index("1")
                    .background("inherit")
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
            )
            // A pressed look also makes Chromium's touch adjustment keep a finger on the strip,
            // not snap it to a button below (3971).
            .selector(format!("{SHEET_HANDLE}:active"), sx().cursor("grabbing"))
            .selector(
                format!("{SHEET_HANDLE}:active::before"),
                sx().background(ColorValue::Shade(Color::Muted, ColorShade::S8).value())
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
        variables,
    } = use_swipe_close(capture, None, onclose);
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
        style: variables.render(),
    }
}

/// The sheet's foot above, and its height within, the visible part of the page.
const SHEET_FOOT: CssVar = CssVar::new("--lsx-sheet-foot");
const SHEET_ROOM: CssVar = CssVar::new("--lsx-sheet-room");
/// Pinch-zoomed and panned sideways, the sheet spans the visible band only (2219).
const SHEET_LEFT: CssVar = CssVar::new("--lsx-sheet-left");
const SHEET_RIGHT: CssVar = CssVar::new("--lsx-sheet-right");

/// A composite of sheet owners, as a `Menubar`'s row: a press inside it keeps its click, so a
/// sibling trigger opens its own sheet (4841).
#[derive(Clone, Copy)]
pub(crate) struct SheetComposite(pub(crate) ElementHandle);

/// A press outside the open sheet only closes it, as a scrim would: its click is swallowed (4826).
/// One on any open popup or its anchor, the sheet's own trigger included, keeps its click.
fn use_sheet_outside_press(placed: bool) {
    let popups = use_open_popups();
    let composite = try_use_context::<SheetComposite>();
    let swallowing = use_subscription_slot::<dyn PressSubscription>();
    use_effect(use_reactive!(|placed| {
        if !placed {
            swallowing.clear();
        } else if !swallowing.is_some() {
            swallowing.set(swallow_outside_clicks(
                &narrow_query(),
                Box::new(move |target| {
                    let in_composite = composite
                        .and_then(|SheetComposite(owner)| owner.try_mounted())
                        .is_some_and(|owner| element_contains(&owner, target));
                    !in_composite && !in_open_popup(popups, target)
                }),
                composite.and_then(|SheetComposite(owner)| owner.tag()),
            ));
        }
    }));
}

/// A phone browser's keyboard shrinks only the visual viewport (2173): the open sheet's foot
/// and room follow it, as inline vars for its `style`. Also swallows an outside press's click.
pub(crate) fn use_sheet_room(sheet: ElementHandle, placed: bool) -> String {
    use_sheet_outside_press(placed);
    let mut room = use_signal(|| None::<SheetBand>);
    let tick = use_signal(|| 0u64);
    let resizing = use_subscription_slot::<dyn ScrollSubscription>();
    use_effect(use_reactive!(|placed| {
        let _ = tick();
        if !placed {
            resizing.clear();
            // The next open starts from the layout viewport's foot, not this one's keyboard band.
            if room.peek().is_some() {
                room.set(None);
            }
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
    // Bumped when the sheet's size changes, so the effect measures again.
    let tick = use_signal(|| 0u64);
    let growing = use_subscription_slot::<dyn ContentSubscription>();
    let observed: Rc<Cell<Option<usize>>> = use_hook(|| Rc::new(Cell::new(None)));
    use_effect(use_reactive!(|(opened, placed)| {
        let _ = tick();
        live.set(opened && placed);
        if !(opened && placed) {
            growing.clear();
            observed.set(None);
            if *clearance.peek() != 0.0 {
                clearance.set(0.0);
            }
            return;
        }
        // Per mount: a remount is a new node to observe.
        let token = sheet.mount_token();
        if observed.replace(token) != token
            && let Some(mounted) = sheet.mounted()
        {
            growing.set(observe_resize(
                &mounted,
                sheet.tag(),
                Box::new(move || bump(tick)),
            ));
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
            if !*live.peek() {
                return;
            }
            let covers = sheet_top < bottom && sheet_top + sheet_size.height > top;
            let next = if covers {
                sheet_size.height + gap + bottom - trigger_bottom
            } else {
                0.0
            };
            if *clearance.peek() != next {
                clearance.set(next);
            }
        });
    }));
    clearance()
}

/// Scrolls the trigger above the sheet once its `scroll-margin-bottom` (the `clearance`) is drawn.
pub(crate) fn use_scroll_clear(anchor: ElementHandle, clearance: f64) {
    use_effect(use_reactive!(|clearance| {
        if clearance > 0.0 {
            let _ = anchor.scroll_into_view(false);
        }
    }));
}
