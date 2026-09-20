use crate::{
    components::common::{
        Variant, contrast_shade_color, focus_ring_sx, hover_color, hover_contrast_color,
        on_state_sx, on_tint_color, selected_color, shade_color, shadow_sx,
    },
    sx::{Sx, ThemeAwareValue, sx},
    theme::{
        ColorShade, CssVar, PAPER_BACKGROUND, PaperDefaults, Size, SizeCss, gradient_fill_sx,
        gradient_hover_sx, gradient_selected_sx,
    },
};

// What `interactive_variant_sx` references by name.
pub(crate) const BUTTON_COLOR_VAR: CssVar = CssVar::new("--lsx-button-color");
pub(crate) const BUTTON_FILL_VAR: CssVar = CssVar::new("--lsx-button-fill");
pub(crate) const BUTTON_CONTRAST_VAR: CssVar = CssVar::new("--lsx-button-contrast");
pub(crate) const BUTTON_HOVER_VAR: CssVar = CssVar::new("--lsx-button-hover");
pub(crate) const BUTTON_SELECTED_VAR: CssVar = CssVar::new("--lsx-button-selected");
pub(crate) const BUTTON_ON_STATE_VAR: CssVar = CssVar::new("--lsx-button-on-state");
pub(crate) const BUTTON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-button-container");
pub(crate) const BUTTON_ON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-button-on-container");

/// The `var()` names the variant chrome reads. `Chip` owns a parallel set
/// under its own prefix, so it renders the same chrome from its own colour.
pub(crate) struct VariantVars<'a> {
    /// The colour in its **text** role: the label of an unfilled variant, and
    /// the outline that matches that label. Not the brand colour itself -
    /// see [`crate::tokens::ColorValue`] for why the two differ.
    pub color: &'a CssVar,
    /// The colour in its **fill** role, under a `contrast` foreground.
    pub fill: &'a CssVar,
    pub contrast: &'a CssVar,
    pub container: &'a CssVar,
    pub on_container: &'a CssVar,
}

/// `Button`'s own set, shared with `SegmentedControl`.
pub(crate) const BUTTON_VARS: VariantVars<'static> = VariantVars {
    color: &BUTTON_COLOR_VAR,
    fill: &BUTTON_FILL_VAR,
    contrast: &BUTTON_CONTRAST_VAR,
    container: &BUTTON_CONTAINER_VAR,
    on_container: &BUTTON_ON_CONTAINER_VAR,
};

// The lightest tint on the ramp. A darker one cannot be labelled legibly:
// black on `S1` clears 12:1 for every palette colour, on `S2` it is closer.
const TONAL_CONTAINER: ColorShade = ColorShade::S1;
const TONAL_SELECTED: ColorShade = ColorShade::S3;

// M3's elevated button rests at level 1 and lifts to level 2 on hover.
const ELEVATED_REST: Size = Size::Xs;
const ELEVATED_HOVER: Size = Size::Sm;

/// Structural chrome for `variant`, in `var()` names rather than resolved
/// values - which is what lets a second component reuse it under its own set.
///
/// The hover response is [`interactive_variant_sx`]'s: `Icon` is a static badge and
/// must not grow one, so it takes the chrome alone.
pub(crate) fn variant_chrome_sx(variant: Variant, vars: &VariantVars) -> Sx {
    let VariantVars {
        color,
        fill,
        contrast,
        container,
        on_container,
    } = vars;
    // A literal colour publishes neither role, so both fall back to the
    // colour var, which holds the literal itself.
    let fill = fill.value_or(color.value());

    match variant {
        Variant::Filled => sx()
            // The rim is the fill, not the accent: a filled control whose
            // border sat one ramp step lighter would show a bright hairline.
            .background(fill.clone())
            .border_color(fill)
            .color(contrast.value_or("inherit")),
        // M3's secondary-container pairing: a light tint of the colour under
        // the label that reads on it. A literal colour has no ramp, so every
        // fallback here lands back on the filled look.
        Variant::Tonal => sx()
            .background(container.value_or(color.value()))
            .border_color("transparent")
            .color(on_container.value_or(contrast.value_or("inherit"))),
        // The *surface*, not a tint of the colour - M3's elevated button is
        // separated from the page by its shadow alone, and the label carries
        // the accent. An opaque background is what the shadow needs to sit on.
        // The shadow stays M3's own pair of steps rather than Paper's resting
        // one: the hover lift is a fixed step above it.
        Variant::Elevated => sx()
            .and(PaperDefaults::background_sx())
            .border_color("transparent")
            .color(color.value())
            .and(shadow_sx(SizeCss::SHADOW.value(ELEVATED_REST))),
        Variant::Outlined => sx()
            .background("transparent")
            .border_color(color.value())
            .color(color.value()),
        Variant::Standard => sx()
            .background("transparent")
            .border_color("transparent")
            .color(color.value()),
        // Reads the gradient vars, not `vars`: the fill is not the colour's.
        Variant::Gradient => gradient_fill_sx().border_color("transparent"),
    }
}

/// [`variant_chrome_sx`] plus the hover response an interactive control needs:
/// a filled or tinted variant darkens, an unfilled one tints, and `Elevated`
/// lifts a level rather than changing its fill.
///
/// `on_state` is the label over the hover and selected fills, where the
/// resting one does not read on them (an outlined `primary` label was 3.52:1
/// on its hover tint, todo 452). `Tonal`'s hover keeps its container's label.
pub(crate) fn interactive_variant_sx(
    variant: Variant,
    vars: &VariantVars,
    hover: &CssVar,
    on_state: &CssVar,
) -> Sx {
    let color = vars.color;

    let fallback = match variant {
        Variant::Filled | Variant::Tonal => vars.fill.value_or(color.value()),
        Variant::Elevated => PAPER_BACKGROUND.value(),
        Variant::Outlined | Variant::Standard => "transparent".to_string(),
        // The state layer over the image, not a flat colour.
        Variant::Gradient => {
            return variant_chrome_sx(variant, vars).selector(HOVER, gradient_hover_sx());
        }
    };

    let mut hovered = sx().background(hover.value_or(fallback));
    match variant {
        Variant::Filled => {
            hovered = hovered.color(on_state.value_or(vars.contrast.value_or("inherit")));
        }
        Variant::Tonal | Variant::Gradient => {}
        Variant::Elevated | Variant::Outlined | Variant::Standard => {
            hovered = hovered.color(on_state.value_or(color.value()));
        }
    }
    if variant == Variant::Elevated {
        hovered = hovered.and(shadow_sx(SizeCss::SHADOW.value(ELEVATED_HOVER)));
    }

    variant_chrome_sx(variant, vars).selector(HOVER, hovered)
}

// Skips a disabled control, so it needs no `pointer-events: none` and can
// show `not-allowed` (todo 586). `:where` keeps the specificity of `:hover`.
const HOVER: &str = "&:hover:not(:where(:disabled, [data-state~=\"disabled\"]))";

/// Resolved values for the colour vars `variant` reads, from the caller's
/// `color`. `None` wherever a literal colour has no shade ramp to walk.
pub(crate) struct VariantColors {
    pub container: Option<String>,
    pub on_container: Option<String>,
    pub hover: Option<String>,
    pub selected: Option<String>,
    pub on_state: Option<String>,
}

/// Which shades a variant tints with. The tinted variants pin absolute
/// shades rather than stepping from `base`, the same way the hover tint
/// always has.
pub(crate) fn variant_colors(variant: Variant, base: &ThemeAwareValue) -> VariantColors {
    let filled = variant == Variant::Filled;
    // Only `Tonal` paints a container; `Elevated` sits on the surface itself.
    let (container, on_container) = match variant {
        Variant::Tonal => (
            shade_color(base, TONAL_CONTAINER),
            contrast_shade_color(base, TONAL_CONTAINER),
        ),
        _ => (None, None),
    };

    let (hover, selected) = match variant {
        Variant::Tonal => (
            shade_color(base, ColorShade::S2),
            shade_color(base, TONAL_SELECTED),
        ),
        // A faint tint over the surface, the way the other unfilled variants
        // hover - the shadow is what actually moves.
        Variant::Elevated => (
            shade_color(base, ColorShade::S1),
            shade_color(base, ColorShade::S2),
        ),
        Variant::Gradient => (None, None),
        _ => (hover_color(base, filled), selected_color(base, filled)),
    };

    // `Tonal`'s only for its selected step: its hover keeps the container's.
    let on_state = match variant {
        Variant::Filled => hover_contrast_color(base),
        Variant::Tonal => contrast_shade_color(base, TONAL_SELECTED),
        Variant::Elevated | Variant::Outlined | Variant::Standard => on_tint_color(base),
        // The gradient's own vars carry every state.
        Variant::Gradient => None,
    };

    VariantColors {
        container,
        on_container,
        hover,
        selected,
        on_state,
    }
}

/// The `selected` look for `variant`, nested inside that variant's own block.
/// Ties that block's own `:hover` on specificity, so it has to stay *after* it
/// in `interactive_variant_sx`'s output - source order is what settles the two.
///
/// The label is `on_state`, as on hover: the resting one need not read on the
/// selected fill. A literal `color` has no shade scale and so no selected
/// tint - falling back to the base colour would paint a full-strength
/// background under the resting label, so the untinted variants fall back to
/// the ring of [`on_state_sx`] alone, which every variant carries.
pub(crate) fn variant_selected_sx(
    variant: Variant,
    vars: &VariantVars,
    selected_var: &CssVar,
    on_state: &CssVar,
) -> Sx {
    let color = vars.color;
    // The house on-state ring; `Elevated` keeps its resting lift under it.
    let marker = match variant {
        Variant::Elevated => on_state_sx(Some(&SizeCss::SHADOW.value(ELEVATED_REST))),
        _ => on_state_sx(None),
    };
    let selected = match variant {
        Variant::Filled => sx()
            .background(selected_var.value_or(color.value()))
            .color(on_state.value_or(vars.contrast.value_or("inherit"))),
        Variant::Tonal => sx().background(selected_var.value_or(color.value())).color(
            on_state.value_or(
                vars.on_container
                    .value_or(vars.contrast.value_or("inherit")),
            ),
        ),
        Variant::Elevated => sx()
            .background(selected_var.value_or(color.value()))
            .color(on_state.value_or(color.value())),
        Variant::Outlined | Variant::Standard => sx()
            .background(selected_var.value_or("transparent"))
            .color(on_state.value_or(color.value())),
        Variant::Gradient => gradient_selected_sx(),
    };
    // Outranks the plain focus rule, which would lose to the ring's
    // `box-shadow` here; the focus ring composes the marker back in.
    selected.and(marker).focus_visible(focus_ring_sx())
}
