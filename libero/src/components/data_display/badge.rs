use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{base_color, base_props, contrast_color, variables},
        inputs::{ButtonVariant, VariantVars, variant_chrome_sx, variant_colors},
        layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{BADGE_BOX, BADGE_RADIUS, BadgeDefaults, CssVar, Size, SizeCss},
};

const BADGE_COLOR_VAR: CssVar = CssVar::new("--lsx-badge-color");
const BADGE_CONTRAST_VAR: CssVar = CssVar::new("--lsx-badge-contrast");
const BADGE_CONTAINER_VAR: CssVar = CssVar::new("--lsx-badge-container");
const BADGE_ON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-badge-on-container");

const BADGE_VARS: VariantVars<'static> = VariantVars {
    color: &BADGE_COLOR_VAR,
    contrast: &BADGE_CONTRAST_VAR,
    container: &BADGE_CONTAINER_VAR,
    on_container: &BADGE_ON_CONTAINER_VAR,
};

static BADGE_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = BadgeDefaults::theme_vars()
        .display("inline-flex")
        .align_items("center")
        // So `Badge { Icon { .. } "Verified" }` reads as two things, without
        // the caller reaching for a `Flex`.
        .gap(SizeCss::SPACING.value(Size::Xs))
        .border_style("solid")
        .border_width("1px")
        .white_space("nowrap")
        .user_select("none")
        // A label, never a target: no I-beam, no drag.
        .cursor("default")
        // Shrink-to-fit even as a stretched flex item, and never wider than
        // what holds it - at which point the label is cut, not wrapped, since
        // the height is fixed. No `text-overflow: ellipsis` to soften that:
        // the property wants a block container, and a flex root is not one, so
        // it is dead here. Mantine gets its ellipsis from the inner label span
        // its grid needs; one element is the trade.
        .width("fit-content")
        .max_width("100%")
        .overflow("hidden");

    // Chrome only, no `:hover` - `Icon`'s rule and the same reason: a static
    // label that changed colour under the pointer would be claiming to be
    // interactive.
    let base = ButtonVariant::ALL.iter().fold(base, |base, &variant| {
        base.when(
            variant.state_name(),
            variant_chrome_sx(variant, &BADGE_VARS),
        )
    });

    // A count badge: the pill becomes a circle by taking its own height as a
    // floor for its width. `BADGE_BOX` is why - the active step's height,
    // republished unsuffixed, so this rule needs to know nothing about which
    // size token is on.
    // `justify-content` only here: a circle's label has room around it and has
    // to sit in the middle of it. On a normal badge, centring an *overflowing*
    // label clips it at both ends and shows the middle - the start is what a
    // reader needs.
    base.when(
        "circle",
        sx().min_width(BADGE_BOX.value())
            .justify_content("center")
            .padding_left("0")
            .padding_right("0"),
    )
});

fn badge_variables(props: &BadgeProps) -> Variables {
    let base = base_color(props.color.as_ref());
    let contrast = contrast_color(&base);
    let colors = variant_colors(props.variant.copied_or_default(), &base);

    variables()
        .with(BADGE_COLOR_VAR, base.resolve(None))
        .with(BADGE_CONTRAST_VAR, contrast.and_then(|c| c.resolve(None)))
        .with(BADGE_CONTAINER_VAR, colors.container)
        .with(BADGE_ON_CONTAINER_VAR, colors.on_container)
        .with(
            BADGE_RADIUS.override_var(),
            props.radius.resolve(Some(SizeCss::RADIUS)),
        )
}

base_props! {
    pub struct BadgeProps {
        /// The five M3 arms, shared with `Button` and `Chip` - minus their
        /// hover response.
        #[props(default, into)]
        variant: Input<ButtonVariant>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<Size>,
        /// A size step or any CSS length. The theme's own default is a pill.
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Squares the padding away and floors the width at the height, for a
        /// one- or two-character count.
        #[props(default)]
        circle: Option<bool>,
        /// The label.
        children: Element,
    }
}

/// A short status label: a pill of uppercase text, sized under a control.
///
/// Renders one `<span>` with no role and no ARIA - a badge is visible text,
/// read in document order, and its content is its accessible name. Something
/// that must announce a *change* is the caller's own `role="status"` region
/// around it, not this.
#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let theme = use_theme();
    let variant = props.variant.copied_or_default();
    let size = props.size.copied_or(theme.badge.size);
    let variables: Input<Variables> = badge_variables(&props).into();

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
        .prepare()
        .render(HtmlTag::Span, props.attributes, props.children)
}
