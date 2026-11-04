use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, States, Variables, Variant, VariantVars, base_color, base_props,
            contrast_color, fill_color, label_color, literal_contrast, variables,
            variant_chrome_sx, variant_colors, with_own,
        },
        data_display::responsive_props::{plain_var, var_sx},
        layout::use_box,
    },
    hooks::{use_gradient_style, use_range_css, use_theme},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        BADGE_BOX, BADGE_RADII, BADGE_RADIUS, BadgeDefaults, CssVar, Gradient, Responsive, Size,
        SizeCss,
    },
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
        .cursor("default")
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
            .padding_right("0")
            .selector("& > * > *", sx().margin("0")),
    )
    // Outside the label, so its ellipsis never cuts the icon and the root's gap spaces it.
    .selector(
        format!("& > [data-slot='{TRAILING_ICON_SLOT}']"),
        sx().display("inline-flex").flex_shrink("0"),
    )
});

// A block, as `text-overflow` is dead on the flex root: a long label ends in an ellipsis
// (todo 4539). Inline here, an icon takes the root's gap as a margin.
static BADGE_LABEL_SX: StaticSx = StaticSx::new(|| {
    let step = SizeCss::SPACING.value(Size::Xs);
    sx().display("block")
        .min_width("0")
        .overflow("hidden")
        .text_overflow("ellipsis")
        .white_space("nowrap")
        .selector("& > *", sx().vertical_align("middle"))
        .selector(
            "& > :first-child",
            sx().with("margin-inline-end", step.clone()),
        )
        .selector(
            "& > * ~ :last-child",
            sx().with("margin-inline-start", step),
        )
});

const TRAILING_ICON_SLOT: &str = "badge-trailing-icon";

fn badge_variables(props: &BadgeProps, color: &ThemeAwareValue, variant: Variant) -> Variables {
    let base = base_color("Badge", Some(color));
    let contrast = contrast_color(&base);
    let colors = variant_colors(variant, &base);

    variables()
        .with(BADGE_COLOR_VAR, label_color(&base))
        .with(BADGE_FILL_VAR, fill_color(&base))
        .with(
            BADGE_CONTRAST_VAR,
            contrast
                .and_then(|c| c.resolve(None))
                .or_else(|| literal_contrast(&base)),
        )
        .with(BADGE_CONTAINER_VAR, colors.container)
        .with(BADGE_ON_CONTAINER_VAR, colors.on_container)
        .with(
            BADGE_RADIUS.override_var(),
            plain_var(&props.radius, Some(BADGE_RADII)),
        )
}

base_props! {
    pub struct BadgeProps {
        /// Per breakpoint too: `responsive(Variant::Outlined).md(Variant::Filled)`.
        #[props(default, into)]
        variant: Input<Responsive<Variant>>,
        /// Second stop and angle of `variant: "gradient"`, whose first stop is `color`:
        /// `("secondary", 45)` or a [`Gradient`]. Unset keys take the theme's.
        #[props(default, into)]
        gradient: Option<Gradient>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Per breakpoint too: `responsive(Size::Sm).md(Size::Lg)`.
        #[props(default, into)]
        size: Input<Responsive<Size>>,
        /// A step on the badge's own radius scale, or any CSS, as `radius: "0"`; the default is a pill.
        /// Per breakpoint too: `responsive(Size::Xs).md(Size::Xxl)`.
        #[props(default, into)]
        radius: Input<Responsive<ThemeAwareValue>>,
        /// A circle for a one- or two-character count; with a `trailing_icon` it stays a pill.
        #[props(default)]
        circle: Option<bool>,
        /// Drawn after the label, as `Icon { .. }` after "Verified".
        #[props(default)]
        trailing_icon: Option<Element>,
        /// The label.
        children: Element,
    }
}

/// A short status label: a pill of uppercase text. Too wide, it ends in an ellipsis.
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
    let variants = props.variant.copied_or(theme.badge.variant.into());
    let sizes = props.size.copied_or(theme.badge.size.into());
    // Only `tonal` reads the container colours: computed if any range is tonal.
    let variant = match variants
        .breakpoints()
        .any(|(_, variant)| variant == Variant::Tonal)
    {
        true => Variant::Tonal,
        false => variants.base(),
    };
    let color = props
        .color
        .as_ref()
        .cloned()
        .unwrap_or_else(|| ThemeAwareValue::from(theme.badge.color));
    let variables: Input<Variables> = badge_variables(&props, &color, variant).into();
    // A themed color seeds the first stop of an unset one (todo 2699).
    let seed = props
        .color
        .as_ref()
        .or_else(|| (theme.badge.color != BadgeDefaults::DEFAULT.color).then_some(&color));
    let gradient = use_gradient_style(
        props.gradient.as_ref(),
        seed,
        variants.base() == Variant::Gradient
            || variants
                .breakpoints()
                .any(|(_, variant)| variant == Variant::Gradient),
        false,
    );

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .responsive(&variants, Variant::state_name)
        .responsive(&sizes, |size| size.state_name())
        // The box can't be round with an icon beside the label: a pill keeps its padding (5449).
        .with(
            "circle",
            props.circle.unwrap_or(false) && props.trailing_icon.is_none(),
        )
        .into();
    let mut copies = sizes.range_copies(|size| size.state_name(), BadgeDefaults::size_sx);
    copies.extend(variants.range_copies(Variant::state_name, |variant| {
        variant_chrome_sx(variant, &BADGE_VARS)
    }));
    use_range_css(&BADGE_BASE_SX, copies);
    let sx = with_own(
        var_sx(
            &props.radius,
            BADGE_RADIUS.override_var(),
            Some(BADGE_RADII),
        ),
        &props.sx,
    );

    let label = use_box()
        .framework_sx(&BADGE_LABEL_SX)
        .focus_ring(false)
        .prepare()
        .render(HtmlTag::Span, Vec::new(), props.children);
    let content = rsx! {
        {label}
        if let Some(icon) = props.trailing_icon {
            span { "data-slot": TRAILING_ICON_SLOT, {icon} }
        }
    };

    use_box()
        .framework_sx(&BADGE_BASE_SX)
        .class(&props.class)
        .sx(&sx)
        .visibility(&props.hidden_from, &props.visible_from)
        .states(&states)
        .variables(&variables)
        .style(gradient)
        .prepare()
        .render(HtmlTag::Span, props.attributes, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Todo 5449: with an icon beside the label the box can't be round, so it stays a pill.
    #[test]
    fn a_circle_with_a_trailing_icon_stays_a_pill() {
        let circle = |icon: bool| {
            let html = dioxus_ssr::render_element(rsx! {
                crate::LiberoProvider {
                    Badge { circle: true, trailing_icon: icon.then(|| rsx! { span {} }), "3" }
                }
            });
            html.split("data-state=\"").skip(1).any(|state| {
                state[..state.find('"').unwrap()]
                    .split(' ')
                    .any(|s| s == "circle")
            })
        };
        assert!(circle(false));
        assert!(!circle(true));
    }
}
