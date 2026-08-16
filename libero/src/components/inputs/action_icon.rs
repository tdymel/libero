use dioxus::prelude::*;

use crate::{
    components::{
        Box, IconVariant, Input, States,
        common::class_list,
        data_display::{icon_base_color, icon_size, icon_variant_sx},
    },
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ColorShade, Size, SizeCss},
};

use super::button::{BUTTON_HOVER_TINT_SHADE, button_hover_sx};

// `Icon`'s own sizing/svg-fit, plus the resets a native `<button>` needs
// that a `<span>` (Icon's default element) never had to worry about - no
// border/background/padding from the browser's own button styling, and no
// default focus ring (the visible one comes from `Box`'s own
// `:focus-visible` handling, same as `Button`).
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
        .width(SizeCss::ICON_SIZE.value(Size::Md))
        .height(SizeCss::ICON_SIZE.value(Size::Md))
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        .selector("& svg", sx().width("100%").height("100%"))
        .when(
            "disabled",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .pointer_events("none"),
        )
});

fn action_icon_dynamic_sx(
    variant: Input<IconVariant>,
    color: Input<ThemeAwareValue>,
    size: Input<ThemeAwareValue>,
    radius: Input<ThemeAwareValue>,
) -> Sx {
    // Unlike `Icon`, background/color (and their hover) are only touched
    // when the caller actually asks for the badge look via `variant`/
    // `color` - a caller supplying its own full `sx` (e.g. `Code`'s copy
    // button, with its own hover treatment) needs those properties left
    // alone, and CSS layering means a plain (non-hover) declaration here
    // would otherwise always beat a `:hover` override from the caller's
    // lower-priority `sx`.
    let variant_sx = match (variant.as_ref(), color.as_ref()) {
        (None, None) => sx(),
        _ => {
            let variant = variant.as_ref().copied().unwrap_or_default();
            let base = icon_base_color(color.as_ref());
            // Mirrors `Button`'s own hover: darker on `Filled`, a light tint
            // behind the border/text on `Outlined`/`Transparent` - this is
            // an interactive button, unlike the plain, hover-less `Icon`
            // whose variant styling this builds on.
            let hover = match variant {
                IconVariant::Filled => button_hover_sx(&base, ColorShade::darker),
                IconVariant::Outlined | IconVariant::Transparent => {
                    button_hover_sx(&base, |_| BUTTON_HOVER_TINT_SHADE)
                }
            };
            let result = icon_variant_sx(variant, base);
            match hover {
                Some(hover) => result.hover(hover),
                None => result,
            }
        }
    };

    variant_sx
        .apply_if(size.as_ref().map(icon_size), |sx, size| {
            sx.width(size.clone()).height(size)
        })
        .apply_if(radius.as_ref(), |sx, radius| {
            sx.border_radius(radius.clone())
        })
}

#[derive(Props, Clone, PartialEq)]
pub struct ActionIconProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
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
    #[props(default)]
    onclick: EventHandler<MouseEvent>,
    #[props(default)]
    onmouseleave: EventHandler<MouseEvent>,
    children: Element,
}

/// `Icon`'s sized/colored/variant-shaped badge, rendered as a real `<button>`
/// instead of a `span` - `Icon` plus `Button`'s click handling and a11y,
/// for icon-only actions like a copy or close button.
#[component]
pub fn ActionIcon(props: ActionIconProps) -> Element {
    let disabled = props.disabled.unwrap_or(false);
    let dynamic_class = crate::hooks::use_css(
        &action_icon_dynamic_sx(props.variant, props.color, props.size, props.radius),
        crate::CssLayer::UserDynamic,
    );
    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("disabled", disabled);

    rsx! {
        Box {
            component: "button",
            class: class_list([props.class, dynamic_class]),
            sx: props.sx,
            states,
            framework_sx: &ACTION_ICON_BASE_SX,
            "aria-label": props.aria_label,
            r#type: "button",
            disabled,
            onclick: props.onclick,
            onmouseleave: props.onmouseleave,
            attributes: props.attributes,
            {props.children}
        }
    }
}
