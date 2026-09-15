use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables, Variant,
        common::{base_color, base_props, contrast_color, fill_color, names_itself, text_color},
        inputs::{VariantVars, variant_chrome_sx, variant_colors},
        layout::use_box,
        variables,
    },
    hooks::{use_cache, use_theme},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{CssVar, ICON_SIZE, Size, SizeCss},
};

pub(crate) const ICON_COLOR_VAR: CssVar = CssVar::new("--lsx-icon-color");
pub(crate) const ICON_FILL_VAR: CssVar = CssVar::new("--lsx-icon-fill");
pub(crate) const ICON_CONTRAST_VAR: CssVar = CssVar::new("--lsx-icon-contrast");
const ICON_RADIUS_VAR: CssVar = CssVar::new("--lsx-icon-radius");
const ICON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-icon-container");
const ICON_ON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-icon-on-container");

pub(crate) const ICON_VARS: VariantVars<'static> = VariantVars {
    color: &ICON_COLOR_VAR,
    fill: &ICON_FILL_VAR,
    contrast: &ICON_CONTRAST_VAR,
    container: &ICON_CONTAINER_VAR,
    on_container: &ICON_ON_CONTAINER_VAR,
};

static ICON_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = sx()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .flex_shrink("0")
        .width(ICON_SIZE.overridable(Size::Md))
        .height(ICON_SIZE.overridable(Size::Md))
        .border_radius(ICON_RADIUS_VAR.value_or(SizeCss::RADIUS.value(Size::Sm)))
        .selector("& svg", sx().width("100%").height("100%"));

    // Chrome only: a badge is not interactive, so it takes no hover response.
    Variant::ALL.iter().fold(base, |base, &variant| {
        base.when(variant.state_name(), variant_chrome_sx(variant, &ICON_VARS))
    })
});

fn icon_variables(props: &IconProps, variant: Variant) -> Variables {
    let base = base_color(props.color.as_ref());
    let contrast = contrast_color(&base);
    let colors = variant_colors(variant, &base);

    variables()
        .with(ICON_COLOR_VAR, text_color(&base))
        .with(ICON_FILL_VAR, fill_color(&base))
        .with(ICON_CONTRAST_VAR, contrast.and_then(|c| c.resolve(None)))
        .with(ICON_CONTAINER_VAR, colors.container)
        .with(ICON_ON_CONTAINER_VAR, colors.on_container)
        .with(
            ICON_SIZE.override_var(),
            props.size.resolve(Some(ICON_SIZE)),
        )
        .with(ICON_RADIUS_VAR, props.radius.resolve(Some(SizeCss::RADIUS)))
}

base_props! {
    pub struct IconProps {
        /// Element to render as; `span` by default.
        #[props(default, into)]
        component: Input<HtmlTag>,
        #[props(default, into)]
        variant: Input<Variant>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        children: Element,
    }
}

/// Wraps an svg child in a sized, colored badge. `color` sets the container's
/// CSS `color`, which a `currentColor` svg then inherits.
///
/// Decorative (`aria-hidden`) unless named: pass `aria_label` or
/// `aria_labelledby` and it becomes `role="img"` under that name.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Icon;
/// # fn app() -> Element {
/// rsx! {
///     Icon { aria_label: "Verified", "✓" }
/// }
/// # }
/// ```
#[component]
pub fn Icon(props: IconProps) -> Element {
    let component = props.component.copied_or(HtmlTag::Span);
    let variant = props.variant.copied_or(use_theme().icon.variant);
    // Colour resolution is most of an icon's render, and its inputs rarely change.
    let style = use_cache(
        (
            variant,
            props.color.clone(),
            props.size.clone(),
            props.radius.clone(),
        ),
        |_| icon_variables(&props, variant).render(),
    );
    let style = Some(style).filter(|style| !style.is_empty());

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(variant.state_name(), true)
        .into();
    let named = names_itself(&props.attributes);

    use_box()
        .framework_sx(&ICON_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .style(style)
        .prepare()
        .attr_default("role", named.then_some("img"))
        .attr_default("aria-hidden", (!named).then_some("true"))
        .render(component, props.attributes, props.children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::inputs::interactive_variant_sx;
    use crate::tokens::{Color, ColorShade, ColorValue};

    fn icon_props(color: Input<ThemeAwareValue>) -> IconProps {
        IconProps {
            component: Input::None,
            class: Default::default(),
            sx: Default::default(),
            states: Input::None,
            attributes: Vec::new(),
            variant: Input::None,
            color,
            size: Input::None,
            radius: Input::None,
            children: rsx! {},
        }
    }

    #[test]
    fn a_bare_theme_color_becomes_a_shade_plus_its_contrast() {
        let variables =
            icon_variables(&icon_props(Color::Error.into()), Variant::Filled).to_string();

        assert!(variables.contains(&format!(
            "{}:{};",
            ICON_COLOR_VAR.name(),
            ColorValue::Text(Color::Error, ColorShade::S6).value()
        )));
        assert!(variables.contains(ICON_CONTRAST_VAR.name()));
    }

    /// A literal has no theme shade, so no contrast to pair with it.
    #[test]
    fn a_literal_color_emits_no_contrast() {
        let variables = icon_variables(&icon_props("#123456".into()), Variant::Filled).to_string();

        assert!(variables.contains(&format!("{}:#123456;", ICON_COLOR_VAR.name())));
        assert!(!variables.contains(ICON_CONTRAST_VAR.name()));
    }

    /// Chrome only - a badge that changed colour under the pointer would be
    /// claiming to be interactive.
    #[test]
    fn a_badge_renders_variant_chrome_without_a_hover() {
        let class_of = |variant| variant_chrome_sx(variant, &ICON_VARS).class_name();

        let classes: Vec<String> = Variant::ALL.iter().map(|v| class_of(*v)).collect();
        let mut unique = classes.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), Variant::ALL.len());

        // The interactive chrome is the same rules plus a `:hover`, so a
        // matching class name would mean the badge grew one.
        assert_ne!(
            variant_chrome_sx(Variant::Filled, &ICON_VARS).class_name(),
            interactive_variant_sx(
                Variant::Filled,
                &ICON_VARS,
                &ICON_COLOR_VAR,
                &ICON_COLOR_VAR
            )
            .class_name()
        );
    }
}
