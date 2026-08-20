use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{
            base_color, base_props, contrast_color, focus_ring_sx, hover_color, input_from_str,
            variables,
        },
        layout::use_box,
        navigation::InternalAnchor,
    },
    hooks::use_theme,
    str_enum::str_enum,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{BUTTON_RIPPLE_ANIMATION, ButtonDefaults, CssVar, Size},
};

// Optional, since a Button is only a link when set. Costs the direct
// `to: Route::Foo {}` that `Anchor` gets: `#[props(into)]` can't chain a
// foreign conversion through a wrapper, so callers need
// `NavigationTarget::from(..)`.
input_from_str!(NavigationTarget);

impl From<NavigationTarget> for Input<NavigationTarget> {
    fn from(value: NavigationTarget) -> Self {
        Input::Value(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Ripple {
    id: u64,
    x: f64,
    y: f64,
}

str_enum! {
    pub enum ButtonVariant {
        Filled = "filled",
        #[default]
        Outlined = "outlined" | "outline",
        Text = "text",
    }
}

input_from_str!(ButtonVariant);

// What `button_variant_sx` references by name.
const BUTTON_COLOR_VAR: CssVar = CssVar::new("--lsx-button-color");
const BUTTON_CONTRAST_VAR: CssVar = CssVar::new("--lsx-button-contrast");
const BUTTON_HOVER_VAR: CssVar = CssVar::new("--lsx-button-hover");

/// Structural chrome for `variant`. The three arguments are `var()` names,
/// not resolved values, so `ActionIcon` reuses this under its own.
pub(crate) fn button_variant_sx(
    variant: ButtonVariant,
    color_var: &CssVar,
    contrast_var: &CssVar,
    hover_var: &CssVar,
) -> Sx {
    match variant {
        ButtonVariant::Filled => sx()
            .background(color_var.value())
            .border_color(color_var.value())
            .color(contrast_var.value_or("inherit"))
            .hover(sx().background(hover_var.value_or(color_var.value()))),
        ButtonVariant::Outlined => sx()
            .background("transparent")
            .border_color(color_var.value())
            .color(color_var.value())
            .hover(sx().background(hover_var.value_or("transparent"))),
        ButtonVariant::Text => sx()
            .background("transparent")
            .border_color("transparent")
            .color(color_var.value())
            .hover(sx().background(hover_var.value_or("transparent"))),
    }
}

static BUTTON_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = ButtonDefaults::theme_vars()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .position("relative")
        .overflow("hidden")
        .border_style("solid")
        .border_width("1px")
        .font_weight("600")
        .cursor("pointer")
        .user_select("none")
        .white_space("nowrap")
        .text_decoration("none")
        .outline("none");

    ButtonVariant::ALL
        .iter()
        .fold(base, |base, &variant| {
            base.when(
                variant.state_name(),
                button_variant_sx(
                    variant,
                    &BUTTON_COLOR_VAR,
                    &BUTTON_CONTRAST_VAR,
                    &BUTTON_HOVER_VAR,
                ),
            )
        })
        .when(
            "disabled",
            // Also stops the variant's `:hover` from ever triggering.
            sx().opacity("0.5")
                .cursor("not-allowed")
                .pointer_events("none"),
        )
        .when("full-width", sx().width("100%"))
        // The base outline is suppressed above and re-added only here.
        .focus_visible(focus_ring_sx())
});

fn button_variables(variant: ButtonVariant, base: &ThemeAwareValue) -> Variables {
    let contrast = contrast_color(base);
    let hover = hover_color(base, variant == ButtonVariant::Filled);

    variables()
        .with(BUTTON_COLOR_VAR, base.resolve(None))
        .with(BUTTON_CONTRAST_VAR, contrast.and_then(|c| c.resolve(None)))
        .with(BUTTON_HOVER_VAR, hover)
}

base_props! {
    pub struct ButtonProps {
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        variant: Input<ButtonVariant>,
        /// Corner radius, independent of `size`.
        #[props(default, into)]
        radius: Input<Size>,
        #[props(default, into)]
        size: Input<Size>,
        #[props(default)]
        full_width: Option<bool>,
        #[props(default)]
        disabled: Option<bool>,
        #[props(default)]
        onclick: EventHandler<MouseEvent>,
        /// Renders a router-aware link instead of a `<button>`. No
        /// ripple/`onclick` then - see `is_link` below.
        #[props(default, into)]
        to: Input<NavigationTarget>,
        #[props(default)]
        target: Option<String>,
        children: Element,
    }
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let theme = use_theme();
    let variant = props.variant.copied_or_default();
    let color = base_color(props.color.as_ref());
    let disabled = props.disabled.unwrap_or(false);
    let full_width = props.full_width.unwrap_or(false);

    // One at a time; a new click overrides the last.
    let mut ripple_signal = use_signal(|| None::<Ripple>);
    let mut next_ripple_id = use_signal(|| 0u64);

    let size = props.size.copied_or(theme.button.size);
    let radius = props.radius.copied_or(theme.button.radius);

    let variables: Input<Variables> = button_variables(variant, &color).into();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("disabled", disabled)
        .with("full-width", full_width)
        .with(variant.state_name(), true)
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .into();

    let handle_click = move |event: Event<MouseData>| {
        let point = event.element_coordinates();
        let id = next_ripple_id();
        next_ripple_id += 1;
        ripple_signal.set(Some(Ripple {
            id,
            x: point.x,
            y: point.y,
        }));
        props.onclick.call(event);
    };

    // A plain `span`, not a `Box`: the ripple has no class, `sx`, states or
    // theming - only a position and an animation - so a `Box` would buy it a
    // scope and a styling pass for nothing.
    //
    // `key` is load-bearing: a new ripple must be a *new* element, or the diff
    // reuses the old one and the CSS animation never restarts.
    let ripple_span = rsx! {
        if let Some(ripple) = ripple_signal() {
            span {
                key: "{ripple.id}",
                style: "position:absolute;left:{ripple.x}px;top:{ripple.y}px;width:300%;height:300%;border-radius:50%;background:currentColor;opacity:0.3;transform:translate(-50%, -50%) scale(0);animation:{BUTTON_RIPPLE_ANIMATION} 550ms ease-out forwards;pointer-events:none;",
                onanimationend: move |_| ripple_signal.set(None),
            }
        }
    };

    // One hook for every path, above the branch: `prepare` is where
    // `use_style_attributes` runs, and hook order has to be the same on every
    // render. The renders below are pure.
    let boxed = use_box()
        .framework_sx(&BUTTON_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .prepare();

    // `InternalAnchor` has no `onclick`: a link-mode Button navigates for
    // real and loses the ripple, which only makes sense on a `<button>`.
    if let Some(to) = props.to.as_ref().cloned() {
        // `<a>` has no native `disabled`: dropping `to` stops navigation,
        // `aria-disabled`/`tabindex` handle the a11y tree and tab order.
        // `InternalAnchor` can't do this - it always resolves a real link.
        if disabled {
            return boxed
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
                framework_sx: &BUTTON_BASE_SX,
                states,
                variables,
                attributes: props.attributes,
                {props.children}
            }
        };
    }

    boxed
        .event("onclick", handle_click)
        .attr("disabled", disabled)
        .attr("type", "button")
        .render(
            HtmlTag::Button,
            props.attributes,
            rsx! {
                {ripple_span}
                {props.children}
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::{Color, ColorShade, ColorValue};

    /// `Button` passes `base_color`'s output, already an explicit shade.
    #[test]
    fn a_filled_button_darkens_on_hover_where_an_outlined_one_tints() {
        let base = base_color(Some(&ThemeAwareValue::Color(Color::Primary)));
        let filled = button_variables(ButtonVariant::Filled, &base).to_string();
        let outlined = button_variables(ButtonVariant::Outlined, &base).to_string();

        assert!(filled.contains(&format!(
            "{}:{};",
            BUTTON_HOVER_VAR.name(),
            ColorValue::Shade(Color::Primary, ColorShade::S6.darker()).value()
        )));
        assert!(outlined.contains(&format!(
            "{}:{};",
            BUTTON_HOVER_VAR.name(),
            ColorValue::Shade(Color::Primary, ColorShade::S1).value()
        )));
    }

    #[test]
    fn the_color_variable_is_the_base_color_itself() {
        let base = ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Error, ColorShade::S7));
        let variables = button_variables(ButtonVariant::Filled, &base).to_string();

        assert!(variables.starts_with(&format!(
            "{}:{};",
            BUTTON_COLOR_VAR.name(),
            ColorValue::Shade(Color::Error, ColorShade::S7).value()
        )));
    }

    /// No computable contrast, so the var stays unset and the class's own
    /// fallback applies.
    #[test]
    fn an_unparseable_color_emits_no_contrast() {
        let base = ThemeAwareValue::String("gold".to_string());
        let variables = button_variables(ButtonVariant::Filled, &base).to_string();

        assert!(!variables.contains(BUTTON_CONTRAST_VAR.name()));
    }

    #[test]
    fn each_variant_renders_its_own_css() {
        let class_of = |variant| {
            button_variant_sx(
                variant,
                &BUTTON_COLOR_VAR,
                &BUTTON_CONTRAST_VAR,
                &BUTTON_HOVER_VAR,
            )
            .class_name()
        };

        let filled = class_of(ButtonVariant::Filled);
        let outlined = class_of(ButtonVariant::Outlined);
        let text = class_of(ButtonVariant::Text);

        assert_ne!(filled, outlined);
        assert_ne!(outlined, text);
        assert_ne!(filled, text);
        assert_eq!(filled, class_of(ButtonVariant::Filled));
    }
}
