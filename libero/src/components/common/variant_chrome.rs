use crate::{
    components::common::{
        Variant, contrast_shade_color, focus_ring_sx, hover_color, hover_contrast_color,
        on_state_sx, on_tint_color, selected_color, shade_color, shadow_sx, tint_color,
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

/// The `var()` names the variant chrome reads; `Chip` owns a parallel set.
pub(crate) struct VariantVars<'a> {
    /// The colour in its text role: an unfilled variant's label and outline.
    pub color: &'a CssVar,
    /// The colour in its fill role, under a `contrast` foreground.
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

// The lightest tint: black on `S1` clears 12:1 for every palette colour.
const TONAL_CONTAINER: ColorShade = ColorShade::S1;
const TONAL_SELECTED: ColorShade = ColorShade::S3;

// M3's elevated button rests at level 1 and lifts to level 2 on hover.
const ELEVATED_REST: Size = Size::Xs;
const ELEVATED_HOVER: Size = Size::Sm;

/// Structural chrome for `variant`, in `var()` names. No hover: static `Icon` takes it alone;
/// see [`interactive_variant_sx`].
pub(crate) fn variant_chrome_sx(variant: Variant, vars: &VariantVars) -> Sx {
    let VariantVars {
        color,
        fill,
        contrast,
        container,
        on_container,
    } = vars;
    // A literal publishes neither role, so both fall back to the colour var.
    let fill = fill.value_or(color.value());

    match variant {
        Variant::Filled => sx()
            // The rim is the fill: a lighter border would show a bright hairline.
            .background(fill.clone())
            .border_color(fill)
            .color(contrast.value_or("inherit")),
        // M3's container pairing. A literal has no ramp, so it falls back to the filled look.
        Variant::Tonal => sx()
            .background(container.value_or(color.value()))
            .border_color("transparent")
            .color(on_container.value_or(contrast.value_or("inherit"))),
        // The surface, as M3: the shadow separates it and the label carries the accent.
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

/// [`variant_chrome_sx`] plus a hover response. `on_state` is the label over hover and
/// selected fills, where the resting one may not read (todo 452).
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

// Skips a disabled control, so it can show `not-allowed` (todo 586). `:where` keeps specificity.
const HOVER: &str = "&:hover:not(:where(:disabled, [data-state~=\"disabled\"]))";

/// Resolved colour vars for a variant; `None` where a literal has no ramp.
pub(crate) struct VariantColors {
    pub container: Option<String>,
    pub on_container: Option<String>,
    pub hover: Option<String>,
    pub selected: Option<String>,
    pub on_state: Option<String>,
}

/// [`VariantColors`]' `container` and `on_container` alone, for a surface without states.
pub(crate) fn variant_container_colors(
    variant: Variant,
    base: &ThemeAwareValue,
) -> (Option<String>, Option<String>) {
    // Only `Tonal` paints a container; `Elevated` sits on the surface itself.
    match variant {
        Variant::Tonal => (
            shade_color(base, TONAL_CONTAINER),
            contrast_shade_color(base, TONAL_CONTAINER),
        ),
        _ => (None, None),
    }
}

/// Which shades a variant tints with; tinted variants pin absolute shades.
pub(crate) fn variant_colors(variant: Variant, base: &ThemeAwareValue) -> VariantColors {
    let filled = variant == Variant::Filled;
    let (container, on_container) = variant_container_colors(variant, base);

    let (hover, selected) = match variant {
        Variant::Tonal => (
            shade_color(base, ColorShade::S2),
            shade_color(base, TONAL_SELECTED),
        ),
        // A faint tint; the shadow is what actually moves.
        Variant::Elevated => (
            tint_color(base, ColorShade::S1),
            tint_color(base, ColorShade::S2),
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

/// The `selected` look; it ties the variant's `:hover`, so it must come after it.
/// A literal `color` has no selected tint and keeps [`on_state_sx`]'s ring alone.
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
    // Outranks the plain focus rule; the focus ring composes the marker back in.
    selected.and(marker).focus_visible(focus_ring_sx())
}
