use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States,
        common::{base_props, input_from_str, variables},
        layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{CssVar, Size, SizeCss, TOOLTIP_DURATION, TooltipDefaults, Z_INDEX_FLOAT},
};

pub use crate::theme::TooltipPlacement;

input_from_str!(TooltipPlacement);

const TOOLTIP_GAP_VAR: CssVar = CssVar::new("--lsx-tooltip-gap");
const TOOLTIP_OPEN_DELAY_VAR: CssVar = CssVar::new("--lsx-tooltip-open-delay");
const TOOLTIP_CLOSE_DELAY_VAR: CssVar = CssVar::new("--lsx-tooltip-close-delay");

/// The bubble, addressed from the wrapper's rules. Its own class is generated,
/// so `role` - which it needs anyway - is what the wrapper can name.
const BUBBLE: &str = "> [role=\"tooltip\"]";

fn gap() -> String {
    TOOLTIP_GAP_VAR.value_or(SizeCss::SPACING.value(Size::Xs))
}

/// `visibility` flips at the *end* of the fade out, and at the start of the
/// wait going in - so a bubble on its way out stays hoverable until it is gone.
fn closed_transition() -> String {
    let delay = TOOLTIP_CLOSE_DELAY_VAR.value_or("0ms");
    let duration = TOOLTIP_DURATION.value();

    format!("opacity {duration} ease {delay}, visibility 0s linear calc({delay} + {duration})")
}

fn closed_sx() -> Sx {
    sx().opacity("0")
        .visibility("hidden")
        .transition(closed_transition())
}

fn open_sx() -> Sx {
    let delay = TOOLTIP_OPEN_DELAY_VAR.value_or("0ms");
    let duration = TOOLTIP_DURATION.value();

    sx().opacity("1").visibility("visible").transition(format!(
        "opacity {duration} ease {delay}, visibility 0s linear {delay}"
    ))
}

static TOOLTIP_WRAPPER_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative")
        .display("inline-block")
        // Not `auto`, so a flex/grid parent's `align-items: stretch` cannot
        // widen the wrapper past its trigger - which would centre the bubble
        // on the container instead of on the trigger.
        .width("max-content")
        .max_width("100%")
        .selector(
            // `:has(:focus-visible)`, not `:focus-within`: a click focuses the
            // trigger too, and the bubble would then stay up after it.
            format!("&:hover {BUBBLE}, &:has(:focus-visible) {BUBBLE}"),
            open_sx(),
        )
        // Folded after the hover rules: equal specificity, so source order is
        // what makes an explicitly controlled tooltip win.
        .when("opened", sx().selector(format!("& {BUBBLE}"), open_sx()))
        .when("closed", sx().selector(format!("& {BUBBLE}"), closed_sx()))
});

/// `gap` is transparent padding, not empty space: a real gap would drop
/// `:hover` the moment the pointer left the trigger, and the bubble could
/// never be reached (WCAG 2.1 SC 1.4.13). Same reason there is no
/// `pointer-events: none`.
fn placement_sx(placement: TooltipPlacement) -> Sx {
    let gap = gap();
    let offset = format!("calc(100% + {gap})");

    match placement {
        TooltipPlacement::Top => sx()
            .bottom(offset)
            .left("50%")
            .transform("translateX(-50%)")
            .selector(
                "&::before",
                sx().top("100%").left("0").right("0").height(gap),
            ),
        TooltipPlacement::Bottom => sx()
            .top(offset)
            .left("50%")
            .transform("translateX(-50%)")
            .selector(
                "&::before",
                sx().bottom("100%").left("0").right("0").height(gap),
            ),
        TooltipPlacement::Left => sx()
            .right(offset)
            .top("50%")
            .transform("translateY(-50%)")
            .selector(
                "&::before",
                sx().left("100%").top("0").bottom("0").width(gap),
            ),
        TooltipPlacement::Right => sx()
            .left(offset)
            .top("50%")
            .transform("translateY(-50%)")
            .selector(
                "&::before",
                sx().right("100%").top("0").bottom("0").width(gap),
            ),
    }
}

static TOOLTIP_BUBBLE_SX: StaticSx = StaticSx::new(|| {
    let base = TooltipDefaults::theme_vars()
        .position("absolute")
        .z_index(Z_INDEX_FLOAT.overridable())
        .width("max-content")
        .max_width("min(20rem, 100vw)")
        .selector("&::before", sx().content("\"\"").position("absolute"))
        .opacity("0")
        .visibility("hidden")
        .transition(closed_transition());

    TooltipPlacement::ALL.iter().fold(base, |base, &placement| {
        base.when(placement.state_name(), placement_sx(placement))
    })
});

base_props! {
    pub struct TooltipProps {
        /// The bubble's content.
        label: Element,
        #[props(default, into)]
        placement: Input<TooltipPlacement>,
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
        #[props(default)]
        opened: Option<bool>,
        /// Renders `children` bare - no wrapper, no bubble.
        #[props(default)]
        disabled: Option<bool>,
        /// The bubble's `id`, so the trigger can carry `aria-describedby`.
        #[props(default, into)]
        label_id: Option<String>,
        /// The trigger. `class`/`sx`/`states`/`attributes` style the *bubble*.
        children: Element,
    }
}

/// A hover/focus label for its `children`. CSS-only: no callbacks, no
/// viewport flipping, and an `overflow: hidden` ancestor clips it.
#[component]
pub fn Tooltip(props: TooltipProps) -> Element {
    let theme = use_theme();
    let placement = props.placement.copied_or(theme.tooltip.placement);
    let size = props.size.copied_or(theme.tooltip.size);
    let gap = props.gap.copied_or(theme.tooltip.gap);

    let variables: Input<crate::components::Variables> = variables()
        .with(TOOLTIP_GAP_VAR, SizeCss::SPACING.value(gap))
        .with(
            TOOLTIP_OPEN_DELAY_VAR,
            format!("{}ms", props.open_delay.unwrap_or(theme.tooltip.open_delay)),
        )
        .with(
            TOOLTIP_CLOSE_DELAY_VAR,
            format!(
                "{}ms",
                props.close_delay.unwrap_or(theme.tooltip.close_delay)
            ),
        )
        .with(Z_INDEX_FLOAT.override_var(), props.z_index.resolve(None))
        .into();

    let wrapper_states: Input<States> = States::default()
        .with("opened", props.opened == Some(true))
        .with("closed", props.opened == Some(false))
        .into();

    let bubble_states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(placement.state_name(), true)
        .with(size.state_name(), true)
        .into();

    // Every hook above the branch - `prepare()` is the hook.
    let wrapper = use_box()
        .framework_sx(&TOOLTIP_WRAPPER_SX)
        .states(&wrapper_states)
        .variables(&variables)
        .prepare();
    let bubble = use_box()
        .framework_sx(&TOOLTIP_BUBBLE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&bubble_states)
        .prepare();

    if props.disabled.unwrap_or(false) {
        return props.children;
    }

    let bubble = bubble
        .attr("role", "tooltip")
        .attr("id", props.label_id)
        .render(HtmlTag::Span, props.attributes, props.label);

    wrapper.render(HtmlTag::Span, Vec::new(), vec![props.children, bubble])
}
