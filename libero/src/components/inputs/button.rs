use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States,
        common::{
            base_color, base_props, contrast_color, focus_ring_sx, hover_color, input_from_str,
            selected_color, shade_color, variables,
        },
        layout::use_box,
        navigation::InternalAnchor,
    },
    hooks::{ripple_sx, use_cache, use_ripple, use_theme},
    str_enum::str_enum,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ButtonDefaults, ColorShade, CssVar, Size, SizeCss},
    utils::warn,
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

str_enum! {
    /// Material 3's five button styles, in descending emphasis.
    pub enum ButtonVariant {
        #[default]
        Filled = "filled",
        Tonal = "tonal" | "filled-tonal",
        Elevated = "elevated",
        Outlined = "outlined" | "outline",
        Text = "text",
    }
}

input_from_str!(ButtonVariant);

// What `button_variant_sx` references by name.
pub(crate) const BUTTON_COLOR_VAR: CssVar = CssVar::new("--lsx-button-color");
pub(crate) const BUTTON_CONTRAST_VAR: CssVar = CssVar::new("--lsx-button-contrast");
pub(crate) const BUTTON_HOVER_VAR: CssVar = CssVar::new("--lsx-button-hover");
pub(crate) const BUTTON_SELECTED_VAR: CssVar = CssVar::new("--lsx-button-selected");
pub(crate) const BUTTON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-button-container");
pub(crate) const BUTTON_ON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-button-on-container");

/// The `var()` names the variant chrome reads. `Chip` owns a parallel set
/// under its own prefix, so it renders the same chrome from its own colour.
pub(crate) struct VariantVars<'a> {
    pub color: &'a CssVar,
    pub contrast: &'a CssVar,
    pub hover: &'a CssVar,
    pub container: &'a CssVar,
    pub on_container: &'a CssVar,
}

/// `Button`'s own set, shared with `SegmentedControl`.
pub(crate) const BUTTON_VARS: VariantVars<'static> = VariantVars {
    color: &BUTTON_COLOR_VAR,
    contrast: &BUTTON_CONTRAST_VAR,
    hover: &BUTTON_HOVER_VAR,
    container: &BUTTON_CONTAINER_VAR,
    on_container: &BUTTON_ON_CONTAINER_VAR,
};

// M3's elevated button rests at level 1 and lifts to level 2 on hover.
const ELEVATED_REST: Size = Size::Xs;
const ELEVATED_HOVER: Size = Size::Sm;

/// Structural chrome for `variant`, in `var()` names rather than resolved
/// values - which is what lets a second component reuse it under its own set.
pub(crate) fn button_variant_sx(variant: ButtonVariant, vars: &VariantVars) -> Sx {
    let VariantVars {
        color,
        contrast,
        hover,
        ..
    } = vars;

    match variant {
        ButtonVariant::Filled => sx()
            .background(color.value())
            .border_color(color.value())
            .color(contrast.value_or("inherit"))
            .hover(sx().background(hover.value_or(color.value()))),
        // A light tint of the colour carrying its own dark shade as the
        // label - M3's secondary-container pairing. A literal colour has no
        // ramp, so every fallback here lands back on the filled look.
        ButtonVariant::Tonal => tonal_sx(vars),
        ButtonVariant::Elevated => tonal_sx(vars)
            .box_shadow(SizeCss::SHADOW.value(ELEVATED_REST))
            .hover(
                sx().background(hover.value_or(color.value()))
                    .box_shadow(SizeCss::SHADOW.value(ELEVATED_HOVER)),
            ),
        ButtonVariant::Outlined => sx()
            .background("transparent")
            .border_color(color.value())
            .color(color.value())
            .hover(sx().background(hover.value_or("transparent"))),
        ButtonVariant::Text => sx()
            .background("transparent")
            .border_color("transparent")
            .color(color.value())
            .hover(sx().background(hover.value_or("transparent"))),
    }
}

/// The tinted container `Tonal` and `Elevated` share; they differ only in
/// which shades [`variant_colors`] puts behind the vars, and in the shadow.
fn tonal_sx(vars: &VariantVars) -> Sx {
    sx().background(vars.container.value_or(vars.color.value()))
        .border_color("transparent")
        .color(
            vars.on_container
                .value_or(vars.contrast.value_or("inherit")),
        )
        .hover(sx().background(vars.hover.value_or(vars.color.value())))
}

/// Resolved values for the colour vars `variant` reads, from the caller's
/// `color`. `None` wherever a literal colour has no shade ramp to walk.
pub(crate) struct VariantColors {
    pub container: Option<String>,
    pub on_container: Option<String>,
    pub hover: Option<String>,
    pub selected: Option<String>,
}

/// Which shades a variant tints with. The tinted variants pin absolute
/// shades rather than stepping from `base`, the same way the hover tint
/// always has.
pub(crate) fn variant_colors(variant: ButtonVariant, base: &ThemeAwareValue) -> VariantColors {
    let filled = variant == ButtonVariant::Filled;
    let (container, on_container) = match variant {
        ButtonVariant::Tonal => (
            shade_color(base, ColorShade::S2),
            shade_color(base, ColorShade::S8),
        ),
        // A near-white container under the colour's own label: the shadow,
        // not the fill, is what separates it from the surface.
        ButtonVariant::Elevated => (shade_color(base, ColorShade::S1), base.resolve(None)),
        _ => (None, None),
    };

    let (hover, selected) = match variant {
        ButtonVariant::Tonal => (
            shade_color(base, ColorShade::S3),
            shade_color(base, ColorShade::S4),
        ),
        ButtonVariant::Elevated => (
            shade_color(base, ColorShade::S2),
            shade_color(base, ColorShade::S3),
        ),
        _ => (hover_color(base, filled), selected_color(base, filled)),
    };

    VariantColors {
        container,
        on_container,
        hover,
        selected,
    }
}

/// The `selected` look for `variant`, nested inside that variant's own block.
/// Ties that block's own `:hover` on specificity, so it has to stay *after* it
/// in `button_variant_sx`'s output - source order is what settles the two.
///
/// Only the background moves: the enclosing variant already sets the label
/// colour every arm here would want, bar `Text`, whose accent label on its own
/// tint is ~2.3:1. A literal `color` has no shade scale and so no selected
/// tint - falling back to the base colour would paint a full-strength
/// background under an `inherit` label, so the untinted variants fall back to
/// nothing at all.
pub(crate) fn button_selected_sx(
    variant: ButtonVariant,
    color_var: &CssVar,
    selected_var: &CssVar,
) -> Sx {
    match variant {
        ButtonVariant::Filled | ButtonVariant::Tonal | ButtonVariant::Elevated => {
            sx().background(selected_var.value_or(color_var.value()))
        }
        // Both label the surface's accent, which is ~2.3:1 on its own tint.
        ButtonVariant::Outlined | ButtonVariant::Text => sx()
            .background(selected_var.value_or("transparent"))
            .color("inherit"),
    }
}

static BUTTON_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = ripple_sx(ButtonDefaults::theme_vars())
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
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
                button_variant_sx(variant, &BUTTON_VARS)
                    // After the variant's `:hover`, which it ties on specificity.
                    .when(
                        "checked",
                        button_selected_sx(variant, &BUTTON_COLOR_VAR, &BUTTON_SELECTED_VAR),
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

/// The colour half of the `style` attribute, already rendered. Depends on
/// `(variant, base)` alone - see [`use_button_variables`], which is what keeps
/// it off the render path.
pub(crate) fn button_variables(
    variant: ButtonVariant,
    base: &ThemeAwareValue,
    selectable: bool,
) -> String {
    let contrast = contrast_color(base);
    let colors = variant_colors(variant, base);
    // Only a toggle button ever reads it, and every other button would pay
    // for the extra declaration in its `style` attribute.
    let selected = selectable.then_some(colors.selected).flatten();

    variables()
        .with(BUTTON_COLOR_VAR, base.resolve(None))
        .with(BUTTON_CONTRAST_VAR, contrast.and_then(|c| c.resolve(None)))
        .with(BUTTON_HOVER_VAR, colors.hover)
        .with(BUTTON_SELECTED_VAR, selected)
        .with(BUTTON_CONTAINER_VAR, colors.container)
        .with(BUTTON_ON_CONTAINER_VAR, colors.on_container)
        .render()
}

base_props! {
    extends(button);
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
        /// Toggle button: renders `aria-pressed` and the selected look.
        /// `None` leaves the button a plain action.
        #[props(default)]
        selected: Option<bool>,
        #[props(default)]
        disabled: Option<bool>,
        /// `Option`, not a bare `EventHandler`: a defaulted one allocates a
        /// `GenerationalBox` on every render of every button - ~300 ns - even
        /// where no caller ever passes a handler.
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
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
    let selectable = props.selected.is_some();
    let selected = props.selected.unwrap_or(false);

    if selectable && props.to.as_ref().is_some() {
        warn(
            "Button: a link keeps the `selected` look but not `aria-pressed`, which `<a>` has no use for.",
        );
    }

    let ripple = use_ripple();

    let size = props.size.copied_or(theme.button.size);
    let radius = props.radius.copied_or(theme.button.radius);

    let showing = ripple.showing();
    // Colour resolution plus rendering is ~790 ns, and `(variant, color)` is
    // the same on almost every render of almost every button.
    let style = use_cache(
        (variant, color, selectable),
        |(variant, color, selectable)| button_variables(*variant, color, *selectable),
    );
    let style = match showing.as_ref() {
        Some(ripple) => ripple.with_point(style),
        None => style,
    };
    let style = Some(style).filter(|style| !style.is_empty());

    // Built in one allocation rather than through `.with()`, which is a
    // `retain` scan and a possible regrow per state. The caller's own states,
    // which are usually absent, keep the merging path.
    let mut own = Vec::with_capacity(7);
    own.extend([
        ("disabled", disabled),
        ("full-width", full_width),
        ("checked", selected),
        (variant.state_name(), true),
        (size.state_name(), true),
        (radius.radius_state_name(), true),
    ]);
    if let Some(ripple) = showing.as_ref() {
        own.push((ripple.state(), true));
    }

    let states: Input<States> = match props.states.as_ref() {
        Some(caller) => own
            .into_iter()
            .fold(caller.clone(), |states, (state, active)| {
                states.with(state, active)
            })
            .into(),
        None => States::from(own).into(),
    };

    let handle_click = move |event: Event<MouseData>| {
        ripple.press(&event);
        if let Some(onclick) = &props.onclick {
            onclick.call(event);
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
        .style(style.clone())
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
                style,
                attributes: props.attributes,
                {props.children}
            }
        };
    }

    // `<button>` defaults to `submit`, which submits an enclosing form; a
    // `Button` defaults to `button`. `attr_default`, so a caller asking for
    // `submit` or `reset` still gets it.
    boxed
        .event("onclick", handle_click)
        .attr("disabled", disabled)
        .attr_default("type", "button")
        .attr_default(
            "aria-pressed",
            selectable.then_some(if selected { "true" } else { "false" }),
        )
        .render(HtmlTag::Button, props.attributes, props.children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::{Color, ColorShade, ColorValue};

    /// `Button` passes `base_color`'s output, already an explicit shade.
    #[test]
    fn a_filled_button_darkens_on_hover_where_an_outlined_one_tints() {
        let base = base_color(Some(&ThemeAwareValue::Color(Color::Primary)));
        let filled = button_variables(ButtonVariant::Filled, &base, false);
        let outlined = button_variables(ButtonVariant::Outlined, &base, false);

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
        let variables = button_variables(ButtonVariant::Filled, &base, false);

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
        let variables = button_variables(ButtonVariant::Filled, &base, false);

        assert!(!variables.contains(BUTTON_CONTRAST_VAR.name()));
    }

    #[test]
    fn each_variant_renders_its_own_css() {
        let class_of = |variant| button_variant_sx(variant, &BUTTON_VARS).class_name();

        let classes: Vec<String> = ButtonVariant::ALL.iter().map(|v| class_of(*v)).collect();
        let mut unique = classes.clone();
        unique.sort();
        unique.dedup();

        assert_eq!(unique.len(), ButtonVariant::ALL.len());
        assert_eq!(classes[0], class_of(ButtonVariant::ALL[0]));
    }

    /// The tinted variants pin absolute shades; the plain ones step from the
    /// base the way they always have.
    #[test]
    fn the_tinted_variants_emit_a_container() {
        let base = base_color(None);

        let tonal = button_variables(ButtonVariant::Tonal, &base, false);
        assert!(tonal.contains(BUTTON_CONTAINER_VAR.name()));
        assert!(tonal.contains(BUTTON_ON_CONTAINER_VAR.name()));

        let outlined = button_variables(ButtonVariant::Outlined, &base, false);
        assert!(!outlined.contains(BUTTON_CONTAINER_VAR.name()));
    }

    /// A literal color has no ramp, so the vars stay unset and the CSS falls
    /// back to the filled look rather than painting an invisible label.
    #[test]
    fn a_literal_color_gets_no_container() {
        let base = ThemeAwareValue::String("gold".to_string());
        let variables = button_variables(ButtonVariant::Tonal, &base, false);

        assert!(!variables.contains(BUTTON_CONTAINER_VAR.name()));
        assert!(!variables.contains(BUTTON_ON_CONTAINER_VAR.name()));
    }
}
