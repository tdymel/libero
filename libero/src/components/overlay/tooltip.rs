use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use super::hover_intent::{HoverIntent, TRIGGER_WRAPPER_SX, use_hover_intent};
use crate::{
    components::{
        HtmlTag, Input, States,
        common::{base_props, input_from_str, variables},
        layout::use_box,
    },
    hooks::{
        Align, ElementHandle, PopoverOptions, escape_closes, use_element, use_escape_dismiss,
        use_focus_within, use_popover_on, use_theme,
    },
    platform::keyboard,
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

/// `gap` is transparent padding, not empty space: the pointer crossing it
/// never leaves the bubble, so it can reach it (WCAG 2.1 SC 1.4.13). On the
/// side the bubble actually landed on, after any flip.
fn bridge_sx(side: Side) -> Sx {
    let gap = TOOLTIP_GAP_VAR.value();
    // A start bubble sits left of the trigger under LTR, so its bridge leaves
    // its right edge; `:dir(rtl)` mirrors that.
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

static TOOLTIP_BUBBLE_SX: StaticSx = StaticSx::new(|| {
    let base = TooltipDefaults::theme_vars()
        .z_index(Z_INDEX_POPOVER.overridable())
        .max_width(format!(
            "min({MAX_WIDTH}, calc(100vw - 2 * {}))",
            POPOVER_PADDING.value()
        ))
        .selector("&::before", sx().content("\"\"").position("absolute"))
        .animation(format!("{TOOLTIP_IN} {} ease", TOOLTIP_DURATION.value()))
        .media(REDUCED_MOTION, sx().animation("none"));

    Side::ALL.iter().fold(base, |base, &side| {
        base.when(side.state_name(), bridge_sx(side))
    })
});

/// Provided above a `Tooltip` whose trigger its caller focuses from code after
/// a press outside it (a slider's track, todo 476): the next focus is a press's.
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
        /// Forces the bubble open or closed; `None` leaves it to hover/focus.
        /// A bubble forced open ignores Escape.
        #[props(default)]
        open: Option<bool>,
        /// Renders `children` bare - no wrapper, no bubble.
        #[props(default)]
        disabled: Option<bool>,
        /// The bubble's `id`, so the trigger can carry `aria-describedby`.
        /// It resolves while the bubble is closed, too.
        #[props(default, into)]
        label_id: Option<String>,
        /// The trigger. `class`/`sx`/`states`/`attributes` style the *bubble*.
        children: Element,
    }
}

/// A hover and keyboard-focus label for its `children`.
///
/// The bubble is portaled, so an `overflow: hidden` ancestor cannot clip it,
/// flips when its side has no room, and closes on Escape (WCAG 1.4.13). Until
/// it opens, it costs a wrapper and its listeners and nothing else.
#[component]
pub fn Tooltip(props: TooltipProps) -> Element {
    let theme = use_theme();
    let hover = use_hover_intent();
    let mut focused = use_signal(|| false);
    // A click focuses the trigger too, and the bubble opens for keyboard focus
    // only - `:focus-visible`, asked on the web, a press heuristic elsewhere.
    let pressed = use_hook(|| Rc::new(Cell::new(false)));
    let press_focus = use_hook(try_consume_context::<PressFocus>);
    let marked = move || press_focus.as_ref().is_some_and(PressFocus::take);
    // Where the document hears Escape, the open bubble's `use_dismiss` does.
    let global_escape = use_hook(|| keyboard().is_some());
    let anchor = use_element();
    // Re-places the open bubble on every render here: a slider's thumb moves
    // its anchor while its value bubble stays open.
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
    let mut wrapper = wrapper
        .element(&anchor)
        .event("onmouseenter", move |_: MouseEvent| {
            hover.hover(true, open_delay)
        })
        .event("onmouseleave", move |_: MouseEvent| {
            leave.set(false);
            hover.hover(false, close_delay);
        })
        .event("onpointerdown", move |_: PointerEvent| pressed.set(true))
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

/// The open bubble. Its own component, so a closed tooltip runs none of the
/// popover's and dismissal's hooks and effects - a page may hold hundreds.
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
    // Focus never enters the bubble, so nothing to hand back and no focus to
    // lose: Escape alone.
    let dismiss = use_escape_dismiss(true, tooltip.open.is_none(), onclose);

    let states: Input<States> = tooltip
        .states
        .unwrap_or_default()
        .with(popover.placement().side.state_name(), true)
        .with(size.state_name(), true)
        .into();
    let variables: Input<crate::components::Variables> = variables()
        .with(TOOLTIP_GAP_VAR, SizeCss::SPACING.value(gap))
        .with(
            Z_INDEX_POPOVER.override_var(),
            tooltip.z_index.resolve(None),
        )
        .into();
    // The popover caps the box inline, where no caller `sx` could change it;
    // the class carries the same cap instead.
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
            .event("onmouseenter", move |_: MouseEvent| hover.hover(true, 0))
            .event("onmouseleave", move |_: MouseEvent| {
                hover.hover(false, close_delay)
            })
            .render(HtmlTag::Span, attributes, tooltip.label.clone()),
    ));

    rsx! {}
}
