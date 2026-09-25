use crate::{
    css::CssDeclaration,
    theme::{Color, ColorShade, ColorValue, HexColor, NamedColorCss, Theme},
    tokens::{Ends, HOVER_TINT_SHADE, SELECTED_TINT_SHADE, ShadeRamp, TEXT_CONTRAST},
};

const SHADES: [ColorShade; 9] = [
    ColorShade::S1,
    ColorShade::S2,
    ColorShade::S3,
    ColorShade::S4,
    ColorShade::S5,
    ColorShade::S6,
    ColorShade::S7,
    ColorShade::S8,
    ColorShade::S9,
];

/// The named colours, then each brand ramp measured against `cards`.
pub(super) fn push_palette_declarations(
    declarations: &mut Vec<CssDeclaration>,
    ends: Ends,
    cards: &[HexColor],
    bases: [(Color, HexColor); 8],
) {
    push_named_color_declarations(declarations, ends);
    // The muted ramp's text role, so a re-themed `muted` carries it.
    declarations.push(CssDeclaration::new(
        NamedColorCss::TEXT_DIMMED.name(),
        ColorValue::Text(Color::Muted, ColorShade::DEFAULT).value(),
    ));
    for (color, base) in bases {
        push_color_declarations(declarations, color, base, ends, cards);
    }
}

fn push_named_color_declarations(declarations: &mut Vec<CssDeclaration>, ends: Ends) {
    declarations.push(CssDeclaration::new(
        NamedColorCss::INK.name(),
        ends.ink.to_string(),
    ));
    declarations.push(CssDeclaration::new(
        NamedColorCss::SURFACE.name(),
        ends.surface.to_string(),
    ));
}

pub(super) fn theme_ends(theme: &Theme) -> Ends {
    Ends {
        surface: theme.surface,
        ink: theme.ink,
    }
}

/// The var a black or white `foreground` is spelled as: on a dark theme white is `--lsx-ink`.
pub(super) fn foreground_var(foreground: HexColor, ends: Ends) -> String {
    if ends.foreground_is_ink(foreground) {
        NamedColorCss::INK.value()
    } else {
        NamedColorCss::SURFACE.value()
    }
}

/// The brand ramp, plus `text-N` and `fill-N` re-based on [`HexColor::text_base`] and
/// [`HexColor::fill_base`], and `contrast-N` paired with `fill-N` (todos 69, 314).
/// Re-based, not snapped per step: snapping merged `fill-6` and `fill-7` and lost the hover.
fn push_color_declarations(
    declarations: &mut Vec<CssDeclaration>,
    color: Color,
    base: HexColor,
    ends: Ends,
    cards: &[HexColor],
) {
    let ramp = color.shade_ramp();
    let text_base = base.text_base(ramp, ends, cards);
    let fill_base = base.fill_base(ramp, ends);

    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            ColorValue::Shade(color, shade).var_name(),
            base.shade(shade, ramp, ends).to_string(),
        ));
    }

    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            ColorValue::Text(color, shade).var_name(),
            text_base.shade(shade, ramp, ends).to_string(),
        ));
    }

    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            ColorValue::Fill(color, shade).var_name(),
            fill_base.shade(shade, ramp, ends).to_string(),
        ));
    }

    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            ColorValue::Contrast(color, shade).var_name(),
            foreground_var(
                fill_base.shade(shade, ramp, ends).readable_contrast(ends),
                ends,
            ),
        ));
    }

    push_on_tint_declarations(declarations, color, base, text_base, fill_base, ends);
}

/// An unfilled control's label on its hover and selected tints, mixed until it reads on
/// both. Declared only where the resting label fails; `on_tint_color` falls back to it.
fn push_on_tint_declarations(
    declarations: &mut Vec<CssDeclaration>,
    color: Color,
    base: HexColor,
    text_base: HexColor,
    fill_base: HexColor,
    ends: Ends,
) {
    let ramp = color.shade_ramp();
    // The neutral ramps label in the brand shade, as `ColorValue::as_text` says.
    let label_base = match ramp {
        ShadeRamp::Chromatic => text_base,
        ShadeRamp::Neutral => base,
    };
    let tints = [
        fill_base.shade(HOVER_TINT_SHADE, ramp, ends),
        fill_base.shade(SELECTED_TINT_SHADE, ramp, ends),
    ];

    for shade in SHADES {
        let label = label_base.shade(shade, ramp, ends);
        let on_tint = label.readable_on(&tints, TEXT_CONTRAST, ends);
        if on_tint == label {
            continue;
        }
        if let Some(name) = ColorValue::Shade(color, shade).on_tint_name() {
            declarations.push(CssDeclaration::new(name, on_tint.to_string()));
        }
    }
}
