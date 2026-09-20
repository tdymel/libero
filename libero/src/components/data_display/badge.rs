use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, States, Variables, Variant, VariantVars, base_color, base_props,
            contrast_color, fill_color, text_color, variables, variant_chrome_sx, variant_colors,
        },
        layout::use_box,
    },
    hooks::{use_gradient_style, use_theme},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{BADGE_BOX, BADGE_RADII, BADGE_RADIUS, BadgeDefaults, CssVar, Gradient, Size, SizeCss},
};

const BADGE_COLOR_VAR: CssVar = CssVar::new("--lsx-badge-color");
const BADGE_FILL_VAR: CssVar = CssVar::new("--lsx-badge-fill");
const BADGE_CONTRAST_VAR: CssVar = CssVar::new("--lsx-badge-contrast");
const BADGE_CONTAINER_VAR: CssVar = CssVar::new("--lsx-badge-container");
const BADGE_ON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-badge-on-container");

const BADGE_VARS: VariantVars<'static> = VariantVars {
    color: &BADGE_COLOR_VAR,
    fill: &BADGE_FILL_VAR,
    contrast: &BADGE_CONTRAST_VAR,
    container: &BADGE_CONTAINER_VAR,
    on_container: &BADGE_ON_CONTAINER_VAR,
};

static BADGE_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = BadgeDefaults::theme_vars()
        .display("inline-flex")
        .align_items("center")
        // So `Badge { Icon { .. } "Verified" }` needs no `Flex`.
        .gap(SizeCss::SPACING.value(Size::Xs))
        .border_style("solid")
        .border_width("1px")
        .white_space("nowrap")
        .user_select("none")
        .cursor("default")
        // Too wide cuts the label: `text-overflow` is dead on a flex root, and
        // an inner span for it would cost a second element.
        .width("fit-content")
        .max_width("100%")
        .overflow("hidden");

    // Chrome only, no `:hover`: a static label must not look interactive.
    let base = Variant::ALL.iter().fold(base, |base, &variant| {
        base.when(
            variant.state_name(),
            variant_chrome_sx(variant, &BADGE_VARS),
        )
    });

    // A count badge floors its width at `BADGE_BOX`, the active size's height.
    // Centred only here: an overflowing pill label must clip at its end, not both.
    base.when(
        "circle",
        sx().min_width(BADGE_BOX.value())
            .justify_content("center")
            .padding_left("0")
            .padding_right("0"),
    )
});

fn badge_variables(props: &BadgeProps, variant: Variant) -> Variables {
    let base = base_color(props.color.as_ref());
    let contrast = contrast_color(&base);
    let colors = variant_colors(variant, &base);

    variables()
        .with(BADGE_COLOR_VAR, text_color(&base))
        .with(BADGE_FILL_VAR, fill_color(&base))
        .with(BADGE_CONTRAST_VAR, contrast.and_then(|c| c.resolve(None)))
        .with(BADGE_CONTAINER_VAR, colors.container)
        .with(BADGE_ON_CONTAINER_VAR, colors.on_container)
        .with(
            BADGE_RADIUS.override_var(),
            props
                .radius
                .as_ref()
                .map(|radius| BADGE_RADII.value(*radius)),
        )
}

base_props! {
    pub struct BadgeProps {
        #[props(default, into)]
        variant: Input<Variant>,
        /// The stops and angle of `variant: "gradient"`, over the theme's.
        #[props(default)]
        gradient: Option<Gradient>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<Size>,
        /// A step on the badge's own radius scale; the default is a pill.
        #[props(default, into)]
        radius: Input<Size>,
        /// A circle for a one- or two-character count.
        #[props(default)]
        circle: Option<bool>,
        /// The label.
        children: Element,
    }
}

/// A short status label: a pill of uppercase text.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Badge;
/// # fn app() -> Element {
/// rsx! {
///     Badge { variant: "outlined", color: "success", "Active" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/badge>
#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let theme = use_theme();
    let variant = props.variant.copied_or(theme.badge.variant);
    let size = props.size.copied_or(theme.badge.size);
    let variables: Input<Variables> = badge_variables(&props, variant).into();
    let gradient = use_gradient_style(props.gradient.as_ref(), variant == Variant::Gradient, false);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(variant.state_name(), true)
        .with(size.state_name(), true)
        .with("circle", props.circle.unwrap_or(false))
        .into();

    use_box()
        .framework_sx(&BADGE_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .style(gradient)
        .prepare()
        .render(HtmlTag::Span, props.attributes, props.children)
}
