use dioxus::prelude::*;

use crate::{
    components::{
        Box, IconVariant, Input, States, Variables,
        common::base_props,
        data_display::{icon_base_color, icon_contrast_color, icon_variant_sx, variant_token},
        navigation::InternalAnchor,
        variables,
    },
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ACTION_ICON_RADIUS, ACTION_ICON_SIZE, ColorShade, SizeCss},
};

use super::button::{BUTTON_HOVER_TINT_SHADE, hover_color};

const ACTION_ICON_COLOR_VAR: &str = "--lsx-action-icon-color";
const ACTION_ICON_CONTRAST_VAR: &str = "--lsx-action-icon-contrast";
const ACTION_ICON_HOVER_VAR: &str = "--lsx-action-icon-hover";
const ACTION_ICON_SIZE_VAR: &str = "--lsx-action-icon-size-override";
const ACTION_ICON_RADIUS_VAR: &str = "--lsx-action-icon-radius-override";

/// `Icon`'s own variant chrome plus a hover, referencing `hover_var` (a
/// `var()` name, not a resolved value) - mirrors `Button`'s own hover:
/// darker on `Filled`, a light tint behind the border/text on
/// `Outlined`/`Transparent` - this is an interactive button, unlike the
/// plain, hover-less `Icon` whose variant styling this builds on.
fn action_icon_variant_sx(
    variant: IconVariant,
    color_var: &str,
    contrast_var: &str,
    hover_var: &str,
) -> Sx {
    let hover_fallback = match variant {
        IconVariant::Filled => format!("var({color_var})"),
        IconVariant::Outlined | IconVariant::Transparent => "transparent".to_string(),
    };

    icon_variant_sx(variant, color_var, contrast_var)
        .hover(sx().background(format!("var({hover_var}, {hover_fallback})")))
}

// `Icon`'s own sizing/svg-fit, plus the resets a native `<button>` needs
// that a `<span>` (Icon's default element) never had to worry about - no
// border/background/padding from the browser's own button styling, and no
// default focus ring (the visible one comes from `Box`'s own
// `:focus-visible` handling, same as `Button`). Unlike `Icon`, the
// filled/outlined/transparent variant chrome is only reachable through the
// matching data-state token, which `ActionIcon` only sets when the caller
// actually asks for the badge look via `variant`/`color` - a caller
// supplying its own full `sx` (e.g. `Code`'s copy button, with its own
// hover treatment) needs background/color left alone entirely, and CSS
// layering means a plain (non-hover) declaration here would otherwise
// always beat a `:hover` override from the caller's lower-priority `sx`.
static ACTION_ICON_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .flex_shrink("0")
        .border("none")
        .background("transparent")
        .padding("0")
        .cursor("pointer")
        .outline("none")
        .width(format!(
            "var({ACTION_ICON_SIZE_VAR}, {})",
            ACTION_ICON_SIZE.value()
        ))
        .height(format!(
            "var({ACTION_ICON_SIZE_VAR}, {})",
            ACTION_ICON_SIZE.value()
        ))
        .border_radius(format!(
            "var({ACTION_ICON_RADIUS_VAR}, {})",
            ACTION_ICON_RADIUS.value()
        ))
        .selector("& svg", sx().width("100%").height("100%"))
        .when(
            "filled",
            action_icon_variant_sx(
                IconVariant::Filled,
                ACTION_ICON_COLOR_VAR,
                ACTION_ICON_CONTRAST_VAR,
                ACTION_ICON_HOVER_VAR,
            ),
        )
        .when(
            "outlined",
            action_icon_variant_sx(
                IconVariant::Outlined,
                ACTION_ICON_COLOR_VAR,
                ACTION_ICON_CONTRAST_VAR,
                ACTION_ICON_HOVER_VAR,
            ),
        )
        .when(
            "transparent",
            action_icon_variant_sx(
                IconVariant::Transparent,
                ACTION_ICON_COLOR_VAR,
                ACTION_ICON_CONTRAST_VAR,
                ACTION_ICON_HOVER_VAR,
            ),
        )
        .when(
            "disabled",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .pointer_events("none"),
        )
});

fn action_icon_variables(props: &ActionIconProps, has_variant_styling: bool) -> Variables {
    let result = variables()
        .with(
            ACTION_ICON_SIZE_VAR,
            props
                .size
                .as_ref()
                .and_then(|v| v.resolve(Some(SizeCss::ICON_SIZE))),
        )
        .with(
            ACTION_ICON_RADIUS_VAR,
            props
                .radius
                .as_ref()
                .and_then(|v| v.resolve(Some(SizeCss::RADIUS))),
        );

    if !has_variant_styling {
        return result;
    }

    let variant = props.variant.as_ref().copied().unwrap_or_default();
    let base = icon_base_color(props.color.as_ref());
    let contrast = icon_contrast_color(&base);
    let hover = match variant {
        IconVariant::Filled => hover_color(&base, ColorShade::darker),
        IconVariant::Outlined | IconVariant::Transparent => {
            hover_color(&base, |_| BUTTON_HOVER_TINT_SHADE)
        }
    };

    result
        .with(ACTION_ICON_COLOR_VAR, base.resolve(None))
        .with(
            ACTION_ICON_CONTRAST_VAR,
            contrast.and_then(|c| c.resolve(None)),
        )
        .with(ACTION_ICON_HOVER_VAR, hover)
}

base_props! {
    pub struct ActionIconProps {
        #[props(default, into)]
        variant: Input<IconVariant>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Required, not optional - an icon-only button has no visible text for
        /// a screen reader to announce, so it needs an accessible name from
        /// somewhere.
        aria_label: String,
        #[props(default)]
        disabled: Option<bool>,
        /// Renders as a link (router-aware, like `Anchor`/`Button`) instead of a
        /// `<button>` when set. No `onclick`/`onmouseleave` in that case, same
        /// tradeoff `Button` makes for its own link mode - real navigation
        /// happens instead.
        #[props(default, into)]
        to: Input<NavigationTarget>,
        #[props(default)]
        target: Option<String>,
        children: Element,
    }
}

/// `Icon`'s sized/colored/variant-shaped badge, rendered as a real `<button>`
/// instead of a `span` - `Icon` plus `Button`'s click handling and a11y,
/// for icon-only actions like a copy or close button.
#[component]
pub fn ActionIcon(props: ActionIconProps) -> Element {
    let disabled = props.disabled.unwrap_or(false);
    let has_variant_styling = props.variant.as_ref().is_some() || props.color.as_ref().is_some();
    let variant = props.variant.as_ref().copied().unwrap_or_default();
    let variables = action_icon_variables(&props, has_variant_styling);

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("disabled", disabled)
        .with(variant_token(variant), has_variant_styling);

    if let Some(to) = props.to.as_ref().cloned() {
        // A disabled link keeps looking/behaving like a disabled control (it
        // just doesn't natively support the `disabled` attribute like
        // <button> does): no `to` at all stops navigation entirely,
        // `aria-disabled`/`tabindex` keep it out of the a11y tree and tab
        // order. Can't go through `InternalAnchor` for this - it always
        // resolves to a real, working link.
        if disabled {
            return rsx! {
                Box {
                    component: "a",
                    class: props.class,
                    sx: props.sx,
                    states,
                    variables,
                    framework_sx: &ACTION_ICON_BASE_SX,
                    "aria-label": props.aria_label,
                    "aria-disabled": "true",
                    tabindex: "-1",
                    attributes: props.attributes,
                    {props.children}
                }
            };
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

    rsx! {
        Box {
            component: "button",
            class: props.class,
            sx: props.sx,
            states,
            variables,
            framework_sx: &ACTION_ICON_BASE_SX,
            "aria-label": props.aria_label,
            r#type: "button",
            disabled,
            attributes: props.attributes,
            {props.children}
        }
    }
}
