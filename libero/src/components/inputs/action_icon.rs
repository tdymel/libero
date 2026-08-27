use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::base_props,
        common::{base_color, contrast_color},
        inputs::{ButtonVariant, VariantVars, button_variant_sx, variant_colors},
        layout::use_box,
        navigation::InternalAnchor,
        variables,
    },
    hooks::{ripple_sx, use_ripple},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{ACTION_ICON_RADIUS, ACTION_ICON_SIZE, CssVar, ICON_SIZE, SizeCss},
};

const ACTION_ICON_COLOR_VAR: CssVar = CssVar::new("--lsx-action-icon-color");
const ACTION_ICON_CONTRAST_VAR: CssVar = CssVar::new("--lsx-action-icon-contrast");
const ACTION_ICON_HOVER_VAR: CssVar = CssVar::new("--lsx-action-icon-hover");
const ACTION_ICON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-action-icon-container");
const ACTION_ICON_ON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-action-icon-on-container");

const ACTION_ICON_VARS: VariantVars<'static> = VariantVars {
    color: &ACTION_ICON_COLOR_VAR,
    contrast: &ACTION_ICON_CONTRAST_VAR,
    container: &ACTION_ICON_CONTAINER_VAR,
    on_container: &ACTION_ICON_ON_CONTAINER_VAR,
};

// `Icon`'s sizing plus the resets a native `<button>` needs that its `<span>`
// never did.
//
// The variant chrome sits behind a data-state token that `ActionIcon` sets
// only when the caller asks for it via `variant`/`color`. A caller with its
// own `sx` (`Code`'s copy button) needs background/color untouched: layering
// means a plain declaration here would beat a `:hover` rule from their
// lower-priority `sx`.
static ACTION_ICON_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = ripple_sx(sx())
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .flex_shrink("0")
        .border("none")
        .background("transparent")
        .padding("0")
        .cursor("pointer")
        .outline("none")
        .width(ACTION_ICON_SIZE.overridable())
        .height(ACTION_ICON_SIZE.overridable())
        .border_radius(ACTION_ICON_RADIUS.overridable())
        .selector("& svg", sx().width("100%").height("100%"));

    let base = ButtonVariant::ALL.iter().fold(base, |base, &variant| {
        base.when(
            variant.state_name(),
            // The ungated base is `border: none` for a caller with its own
            // `sx`, so the width the variants' `border-color` needs joins
            // here rather than out there.
            button_variant_sx(variant, &ACTION_ICON_VARS, &ACTION_ICON_HOVER_VAR)
                .border_style("solid")
                .border_width("1px"),
        )
    });

    // After the variants, so it also stops their `:hover` from triggering.
    base.when(
        "disabled",
        sx().opacity("0.5")
            .cursor("not-allowed")
            .pointer_events("none"),
    )
});

fn action_icon_variables(props: &ActionIconProps, has_variant_styling: bool) -> Variables {
    let result = variables()
        .with(
            ACTION_ICON_SIZE.override_var(),
            props.size.resolve(Some(ICON_SIZE)),
        )
        .with(
            ACTION_ICON_RADIUS.override_var(),
            props.radius.resolve(Some(SizeCss::RADIUS)),
        );

    if !has_variant_styling {
        return result;
    }

    let base = base_color(props.color.as_ref());
    let contrast = contrast_color(&base);
    let colors = variant_colors(props.variant.copied_or_default(), &base);

    result
        .with(ACTION_ICON_COLOR_VAR, base.resolve(None))
        .with(
            ACTION_ICON_CONTRAST_VAR,
            contrast.and_then(|c| c.resolve(None)),
        )
        .with(ACTION_ICON_HOVER_VAR, colors.hover)
        .with(ACTION_ICON_CONTAINER_VAR, colors.container)
        .with(ACTION_ICON_ON_CONTAINER_VAR, colors.on_container)
}

base_props! {
    pub struct ActionIconProps {
        #[props(default, into)]
        variant: Input<ButtonVariant>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Required: an icon-only button has no text to announce.
        aria_label: String,
        #[props(default)]
        disabled: Option<bool>,
        /// `Option`, not a bare `EventHandler` - see `Button`.
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        /// Renders a router-aware link instead of a `<button>`. No
        /// ripple/`onclick`/`onmouseleave` then - real navigation happens
        /// instead.
        #[props(default, into)]
        to: Input<NavigationTarget>,
        #[props(default)]
        target: Option<String>,
        children: Element,
    }
}

/// `Icon`'s badge as a real `<button>`, with `Button`'s click handling and
/// a11y - for icon-only actions like copy or close.
#[component]
pub fn ActionIcon(props: ActionIconProps) -> Element {
    let disabled = props.disabled.unwrap_or(false);
    let has_variant_styling = props.variant.as_ref().is_some() || props.color.as_ref().is_some();
    let variant = props.variant.copied_or_default();
    let variables: Input<Variables> = action_icon_variables(&props, has_variant_styling).into();

    let ripple = use_ripple();
    let showing = ripple.showing();

    let states = props
        .states
        .unwrap_or_default()
        .with("disabled", disabled)
        .with(variant.state_name(), has_variant_styling);
    let states: Input<States> = match showing.as_ref() {
        Some(ripple) => states.with(ripple.state(), true),
        None => states,
    }
    .into();

    let handle_click = move |event: Event<MouseData>| {
        ripple.press(&event);
        if let Some(onclick) = &props.onclick {
            onclick.call(event);
        }
    };

    // One hook for every path, above the branch - see `Button`.
    let boxed = use_box()
        .framework_sx(&ACTION_ICON_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        // Only a button that has been clicked pays for this.
        .style(showing.map(|ripple| ripple.with_point(String::new())))
        .prepare();

    if let Some(to) = props.to.as_ref().cloned() {
        // `<a>` has no native `disabled`: dropping `to` stops navigation,
        // `aria-disabled`/`tabindex` handle the a11y tree and tab order.
        // `InternalAnchor` can't do this - it always resolves a real link.
        if disabled {
            return boxed
                .attr("aria-label", props.aria_label)
                .attr("aria-disabled", "true")
                .attr("tabindex", "-1")
                .render(HtmlTag::A, props.attributes, props.children);
        }

        return rsx! {
            InternalAnchor {
                to,
                target: props.target,
                class: props.class,
                sx: props.sx,
                framework_sx: &ACTION_ICON_BASE_SX,
                states,
                variables,
                "aria-label": props.aria_label,
                attributes: props.attributes,
                {props.children}
            }
        };
    }

    boxed
        .event("onclick", handle_click)
        .attr("aria-label", props.aria_label)
        .attr("disabled", disabled)
        .attr_default("type", "button")
        .render(HtmlTag::Button, props.attributes, props.children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::{Color, Size};

    fn action_icon_props(color: Input<ThemeAwareValue>) -> ActionIconProps {
        ActionIconProps {
            class: Default::default(),
            sx: Default::default(),
            states: Input::None,
            attributes: Vec::new(),
            variant: Input::None,
            color,
            size: Size::Md.into(),
            radius: Input::None,
            aria_label: "test".to_string(),
            disabled: None,
            onclick: None,
            to: Input::None,
            target: None,
            children: rsx! {},
        }
    }

    /// Sizing is always ActionIcon's; the color vars appear only when it
    /// isn't deferring to a wrapped `Icon`.
    #[test]
    fn without_variant_styling_only_the_size_variables_are_set() {
        let props = action_icon_props(Color::Primary.into());
        let variables = action_icon_variables(&props, false).to_string();

        assert!(variables.contains(ACTION_ICON_SIZE.override_var().name()));
        assert!(!variables.contains(ACTION_ICON_COLOR_VAR.name()));
    }

    #[test]
    fn with_variant_styling_the_colour_variables_are_added() {
        let props = action_icon_props(Color::Primary.into());
        let variables = action_icon_variables(&props, true).to_string();

        assert!(variables.contains(ACTION_ICON_SIZE.override_var().name()));
        assert!(variables.contains(ACTION_ICON_COLOR_VAR.name()));
    }
}
