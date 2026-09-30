use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use super::hover_intent::{HoverIntent, TRIGGER_WRAPPER_SX, use_hover_intent};
use crate::{
    CssLayer,
    components::{
        common::{HtmlTag, Input, States, base_props, input_from_str, variables},
        layout::use_box,
    },
    hooks::{
        Align, ElementHandle, PopoverOptions, Rect, escape_closes, place, use_css, use_element,
        use_escape_dismiss, use_focus_within, use_popover_on, use_portal_slot, use_theme,
    },
    platform::{Dimensions, ElementApi, document, keyboard, when_laid_out},
    sx::{REDUCED_MOTION, StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        CssVar, POPOVER_PADDING, Size, SizeCss, TOOLTIP_DURATION, TOOLTIP_IN, TooltipDefaults,
        Z_INDEX_POPOVER,
    },
};

use crate::theme::Side;

input_from_str!(Side);

const TOOLTIP_GAP_VAR: CssVar = CssVar::new("--lsx-tooltip-gap");

/// The bubble's widest, unless the viewport is narrower.
const MAX_WIDTH: &str = "20rem";

/// Milliseconds a touch must hold to open it, and it stays after the release (MUI's).
const LONG_PRESS: u32 = 500;
const TOUCH_LINGER: u32 = 1500;

/// Bridges `gap` on the landed side, so the pointer can reach the bubble (WCAG 1.4.13).
fn bridge_sx(side: Side) -> Sx {
    let gap = TOOLTIP_GAP_VAR.value();
    let (bridge, rtl) = match side {
        Side::Top => (sx().top("100%").left("0").right("0").height(gap), None),
        Side::Bottom => (sx().bottom("100%").left("0").right("0").height(gap), None),
        Side::Start => (
            sx().left("100%").top("0").bottom("0").width(gap.clone()),
            Some(sx().left("auto").right("100%")),
        ),
        Side::End => (
            sx().right("100%").top("0").bottom("0").width(gap.clone()),
            Some(sx().right("auto").left("100%")),
        ),
    };
    let base = sx().selector("&::before", bridge);
    match rtl {
        Some(rtl) => base.rtl(sx().selector("&::before", rtl)),
        None => base,
    }
}

fn bubble_sx() -> Sx {
    TooltipDefaults::theme_vars()
        .z_index(Z_INDEX_POPOVER.overridable())
        .max_width(format!(
            "min({MAX_WIDTH}, calc(100vw - 2 * {}))",
            POPOVER_PADDING.value()
        ))
        .selector("&::before", sx().content("\"\"").position("absolute"))
        .animation(format!("{TOOLTIP_IN} {} ease", TOOLTIP_DURATION.value()))
        .media(REDUCED_MOTION, sx().animation("none"))
}

static TOOLTIP_BUBBLE_SX: StaticSx = StaticSx::new(|| {
    Side::ALL.iter().fold(bubble_sx(), |base, &side| {
        base.when(side.state_name(), bridge_sx(side))
    })
});

/// `flex: none`: a 0x0 point would shrink it to its longest word.
static TOOLTIP_PINNED_SX: StaticSx = StaticSx::new(|| {
    bubble_sx()
        .flex("none")
        .width("max-content")
        .pointer_events("none")
});

/// The 0x0 point a pinned bubble grows from; overflowing it keeps a label that
/// changes width centred, no `transform` (Blitz's client rect ignores one).
static TOOLTIP_POINT_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .width("0")
        .height("0")
        .display("flex")
        .pointer_events("none")
        .z_index(Z_INDEX_POPOVER.overridable())
});

/// Provided above a `Tooltip` whose trigger its caller focuses from code after
/// a press outside it (a slider's track, todo 476): the next focus is a press's.
///
/// Its caller draws a drag's bubble itself, [`TooltipPinned`]: a touch's long
/// press opens none (1058).
#[derive(Clone, Default)]
pub(crate) struct PressFocus(Rc<Cell<bool>>);

impl PressFocus {
    pub(crate) fn mark(&self) {
        self.0.set(true);
    }

    pub(crate) fn clear(&self) {
        self.0.set(false);
    }

    fn take(&self) -> bool {
        self.0.replace(false)
    }
}

base_props! {
    pub struct TooltipProps {
        /// The bubble's content.
        label: Element,
        /// The preferred side. The bubble flips when that side has no room.
        #[props(default, into)]
        side: Input<Side>,
        /// Distance to the trigger, bridged so the pointer can cross it.
        #[props(default, into)]
        gap: Input<Size>,
        #[props(default, into)]
        size: Input<Size>,
        #[props(default, into)]
        z_index: Input<ThemeAwareValue>,
        /// Milliseconds the pointer must rest before the bubble appears.
        #[props(default)]
        open_delay: Option<u32>,
        #[props(default)]
        close_delay: Option<u32>,
        /// Forces the bubble open or closed; `None` leaves it to hover and focus.
        #[props(default)]
        open: Option<bool>,
        /// Renders `children` bare: no wrapper, no bubble.
        #[props(default)]
        disabled: Option<bool>,
        /// The bubble's `id`, for the trigger's `aria-describedby`.
        #[props(default, into)]
        label_id: Option<String>,
        /// The trigger. `class`/`sx`/`states`/`attributes` style the *bubble*.
        children: Element,
    }
}

/// A hover and keyboard-focus label for its `children`.
///
/// The bubble carries `data-closing` while the pointer has left and `close_delay` counts down;
/// the pointer coming back or the close firing removes it.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, Tooltip};
/// # fn app() -> Element {
/// # rsx! {
/// Tooltip { label: rsx! { "Save changes" },
///     Button { "Save" }
/// }
/// # } }
/// ```
///
/// Docs: <https://libero-ui.dev/overlay/tooltip>
#[component]
pub fn Tooltip(props: TooltipProps) -> Element {
    let theme = use_theme();
    let hover = use_hover_intent();
    let mut focused = use_signal(|| false);
    // Opens for keyboard focus only: `:focus-visible` on the web, a press heuristic elsewhere.
    let pressed = use_hook(|| Rc::new(Cell::new(false)));
    // The last press was a touch, until the pointer leaves: a long press opens it (996).
    let touch = use_hook(|| Rc::new(Cell::new(false)));
    let press_focus = use_hook(try_consume_context::<PressFocus>);
    let owned = press_focus.is_some();
    let marked = move || press_focus.as_ref().is_some_and(PressFocus::take);
    // Where the document hears Escape, the open bubble's `use_dismiss` does.
    let global_escape = use_hook(|| keyboard().is_some());
    let anchor = use_element();
    // Re-places the open bubble every render: a slider thumb moves its anchor.
    let generation = use_hook(|| Rc::new(Cell::new(0u64)));
    generation.set(generation.get().wrapping_add(1));

    // Escape, or a forced-open bubble released: closed until the pointer
    // comes back or focus arrives again.
    let onclose = use_callback(move |()| {
        hover.set(false);
        focused.set(false);
    });
    let focus = use_focus_within(move || vec![anchor.mounted()], {
        let pressed = pressed.clone();
        move |change| {
            let mut focused = focused;
            if !change.within {
                focused.set(false);
                return;
            }
            // The web answers itself, except for a focus from code after a
            // press outside this wrapper, which its caller marks (todo 476).
            let pointer = pressed.replace(false);
            if !marked() && change.focus_visible().unwrap_or(!pointer) {
                focused.set(true);
            }
        }
    });

    // Every hook above the branch - `prepare()` is the hook.
    let wrapper = use_box().framework_sx(&TRIGGER_WRAPPER_SX).prepare();

    if props.disabled.unwrap_or(false) {
        return props.children;
    }

    let open_delay = props.open_delay.unwrap_or(theme.tooltip.open_delay);
    let close_delay = props.close_delay.unwrap_or(theme.tooltip.close_delay);
    let open = props.open.unwrap_or(hover.get() || focused());
    let dismissible = props.open.is_none();

    let bubble = match open {
        true => rsx! {
            TooltipBubble {
                tooltip: props.clone(),
                anchor,
                hover,
                generation: generation.get(),
                onclose,
            }
        },
        // A hidden element still describes the trigger that points at it.
        false => match props.label_id.clone() {
            Some(id) => rsx! {
                span { id, role: "tooltip", hidden: true, {props.label.clone()} }
            },
            None => VNode::empty(),
        },
    };

    let leave = pressed.clone();
    let (enter, leave_touch, press_touch) = (touch.clone(), touch.clone(), touch.clone());
    let mut wrapper = wrapper
        .element(&anchor)
        // A touch's compatibility mouse events are no hover: a tap never opens it.
        .event("onmouseenter", move |_: MouseEvent| {
            if !enter.get() {
                hover.hover(true, open_delay)
            }
        })
        .event("onmouseleave", move |_: MouseEvent| {
            leave.set(false);
            if !leave_touch.replace(false) {
                hover.hover(false, close_delay);
            }
        })
        .event("onpointerdown", move |event: PointerEvent| {
            pressed.set(true);
            let is_touch = event.data().pointer_type() == "touch";
            press_touch.set(is_touch);
            if is_touch && !owned {
                hover.hover(true, LONG_PRESS);
            }
        })
        // Released before the long press: nothing. After: it lingers, then closes.
        .event("onpointerup", move |event: PointerEvent| {
            if event.data().pointer_type() == "touch" && !owned {
                hover.hover(false, TOUCH_LINGER);
            }
        })
        .event("onpointercancel", move |event: PointerEvent| {
            if event.data().pointer_type() == "touch" && !owned {
                hover.hover(false, TOUCH_LINGER);
            }
        })
        .event("onfocusin", focus.focusin(0))
        .event("onfocusout", focus.focusout(0));
    // Off the web only the focused element hears Escape. Absent while closed,
    // so it never swallows Escape for an enclosing `Modal`.
    if open && dismissible && !global_escape {
        wrapper = wrapper.event("onkeydown", move |event: KeyboardEvent| {
            if escape_closes(&event) {
                event.prevent_default();
                event.stop_propagation();
                onclose.call(());
            }
        });
    }
    wrapper.render(HtmlTag::Span, Vec::new(), vec![props.children, bubble])
}

/// A forced-open bubble on a rect its caller works out each render, a slider
/// thumb's from its value: portaled, but measured once per open (1065).
#[component]
pub(crate) fn TooltipPinned(
    label: Element,
    /// The trigger, in viewport coordinates.
    anchor: Rect,
    size: Size,
    gap: Size,
    id: Option<String>,
    rtl: bool,
) -> Element {
    let theme = use_theme();
    let options = PopoverOptions::new(theme.spacing.get(gap).into(), theme.popover.padding)
        .side(theme.tooltip.side)
        .align(Align::Center);
    let floating = use_element();
    let slot = use_portal_slot();
    // The bubble's size and the viewport, which a drag does not change.
    let mut measured = use_signal(|| None::<(Dimensions, Dimensions)>);
    let retry = use_signal(|| 0u8);
    use_effect(move || {
        let tries = retry();
        if floating.mount_token().is_none() || measured.peek().is_some() {
            return;
        }
        let Some(document) = document() else {
            return;
        };
        let (size, viewport) = (floating.dimensions(), document.viewport());
        spawn(async move {
            let (Ok(size), Ok(viewport)) = (size.await, viewport.await) else {
                return;
            };
            // Not laid out yet (native shell), as `use_popover_on` waits (todo 896).
            if size.width == 0.0 && size.height == 0.0 && tries < 3 {
                when_laid_out(move || {
                    let mut retry = retry;
                    retry.set(tries + 1);
                });
                return;
            }
            measured.set(Some((size, viewport)));
        });
    });

    let placed = measured().map(|(size, viewport)| {
        let placed = place(anchor, size, viewport, &options, rtl);
        (placed, size)
    });
    let side = placed.map_or(options.side, |(placed, _)| placed.placement.side);
    // The point on the landed side the bubble grows away from.
    let point = placed.map(|(placed, size)| {
        let (x, y, width, height) = (placed.x, placed.y, size.width, size.height);
        match side {
            Side::Top => (x + width / 2.0, y + height, "center", "flex-end"),
            Side::Bottom => (x + width / 2.0, y, "center", "flex-start"),
            side if (side == Side::Start) != rtl => {
                (x + width, y + height / 2.0, "flex-end", "center")
            }
            _ => (x, y + height / 2.0, "flex-start", "center"),
        }
    });
    let style = match point {
        Some((x, y, justify, align)) => format!(
            "left:{x}px;top:{y}px;justify-content:{justify};align-items:{align};visibility:visible;"
        ),
        None => String::from(
            "left:0px;top:0px;justify-content:flex-start;align-items:flex-start;visibility:hidden;",
        ),
    };

    let states: Input<States> = States::default()
        .with(side.state_name(), true)
        .with(size.state_name(), true)
        .into();
    let point_class = use_css(Some(&TOOLTIP_POINT_SX), CssLayer::Framework);
    let bubble = use_box()
        .framework_sx(&TOOLTIP_PINNED_SX)
        .states(&states)
        .prepare()
        .element(&floating)
        .attr("role", "tooltip")
        .attr("id", id)
        .render(HtmlTag::Span, Vec::new(), label);
    slot.show(Some(rsx! {
        span { class: point_class, style, {bubble} }
    }));

    rsx! {}
}

/// Its own component, so a closed tooltip runs no popover hooks: a page may hold hundreds.
#[component]
fn TooltipBubble(
    tooltip: TooltipProps,
    anchor: ElementHandle,
    hover: HoverIntent,
    generation: u64,
    onclose: Callback<()>,
) -> Element {
    let theme = use_theme();
    let side = tooltip.side.copied_or(theme.tooltip.side);
    let size = tooltip.size.copied_or(theme.tooltip.size);
    let gap = tooltip.gap.copied_or(theme.tooltip.gap);
    let close_delay = tooltip.close_delay.unwrap_or(theme.tooltip.close_delay);
    let padding = theme.popover.padding;

    let popover = use_popover_on(
        anchor,
        use_element(),
        true,
        PopoverOptions::new(theme.spacing.get(gap).into(), padding)
            .side(side)
            .align(Align::Center)
            .remeasure(generation),
    );
    let floating = *popover.floating();
    // Focus never enters the bubble: Escape alone.
    let dismiss = use_escape_dismiss(true, tooltip.open.is_none(), onclose);

    let states: Input<States> = tooltip
        .states
        .unwrap_or_default()
        .with(popover.placement().side.state_name(), true)
        .with(size.state_name(), true)
        .into();
    let variables: Input<crate::components::common::Variables> = variables()
        .with(TOOLTIP_GAP_VAR, SizeCss::SPACING.value(gap))
        .with(
            Z_INDEX_POPOVER.override_var(),
            tooltip.z_index.resolve(None),
        )
        .into();
    // The class carries the popover's inline cap, so a caller `sx` can change it.
    let inline_cap = format!("max-width:calc(100vw - {}px);", 2.0 * padding);
    let style = popover.style().map(|style| style.replace(&inline_cap, ""));

    let bubble = use_box()
        .framework_sx(&TOOLTIP_BUBBLE_SX)
        .class(&tooltip.class)
        .sx(&tooltip.sx)
        .states(&states)
        .variables(&variables)
        .style(style)
        .prepare();

    let mut attributes = tooltip.attributes.clone();
    attributes.extend(dismiss.floating_events());
    popover.show(Some(
        bubble
            .element(&floating)
            .attr("role", "tooltip")
            .attr("id", tooltip.label_id.clone())
            .attr("data-closing", hover.closing())
            .event("onmouseenter", move |_: MouseEvent| hover.hover(true, 0))
            .event("onmouseleave", move |_: MouseEvent| {
                hover.hover(false, close_delay)
            })
            .render(HtmlTag::Span, attributes, tooltip.label.clone()),
    ));

    rsx! {}
}
