use crate::{
    css::CssDeclaration,
    theme::{Color, ColorShade, ColorValue, HexColor, NamedColorCss, Theme},
    tokens::{Ends, HOVER_TINT_SHADE, SELECTED_TINT_SHADE, ShadeRamp, TEXT_CONTRAST},
};

const ROW_HOVER_CONTRAST: f32 = 1.15;
const ROW_STRIPE_CONTRAST: f32 = 1.05;

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
        if color == Color::Muted {
            push_row_hover_declaration(declarations, base, ends, cards);
        }
    }
}

/// The muted fill step a hovered row takes: the first that shows 1.15:1 on every card (todo 2683).
/// A striped row takes a lower step at 1.05:1, so a hovered stripe still changes (todo 4555).
fn push_row_hover_declaration(
    declarations: &mut Vec<CssDeclaration>,
    base: HexColor,
    ends: Ends,
    cards: &[HexColor],
) {
    let ramp = Color::Muted.shade_ramp();
    let fill_base = base.fill_base(ramp, ends);
    let shows = |color: HexColor, floor: f32| {
        cards
            .iter()
            .chain([&ends.surface])
            .all(|card| color.contrast_ratio(*card) >= floor)
    };
    let step = |shade| fill_base.shade(shade, ramp, ends);
    let hover = SHADES
        .into_iter()
        .position(|shade| shows(step(shade), ROW_HOVER_CONTRAST))
        .unwrap_or(SHADES.len() - 1);
    declarations.push(CssDeclaration::new(
        NamedColorCss::ROW_HOVER.name(),
        ColorValue::Fill(Color::Muted, SHADES[hover]).value(),
    ));

    let stripe = match SHADES[..hover]
        .iter()
        .find(|shade| shows(step(**shade), ROW_STRIPE_CONTRAST))
    {
        Some(shade) => ColorValue::Fill(Color::Muted, *shade).value(),
        // No step below the hover shows: the first mix of the page into it that does.
        None => {
            let to = step(SHADES[hover]);
            (1..100)
                .map(|weight| mix(to, ends.surface, weight))
                .find(|color| shows(*color, ROW_STRIPE_CONTRAST))
                .unwrap_or(to)
                .to_string()
        }
    };
    declarations.push(CssDeclaration::new(
        NamedColorCss::ROW_STRIPE.name(),
        stripe,
    ));
}

/// `a` at `percent` over `b`, as `color-mix(in srgb, …)` paints it.
fn mix(a: HexColor, b: HexColor, percent: u8) -> HexColor {
    let share = f32::from(percent) / 100.0;
    let mix = |x: u8, y: u8| (f32::from(x) * share + f32::from(y) * (1.0 - share)).round() as u32;
    HexColor::new((mix(a.r(), b.r()) << 16) | (mix(a.g(), b.g()) << 8) | mix(a.b(), b.b()))
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

/// The theme's own end that reads on `fill`, else true black or white, as Mantine's
/// `autoContrast`: a grey ink reaches no 4.5:1 on a mid yellow (todo 4554).
fn fill_label(fill: HexColor, ends: Ends) -> String {
    let pick = fill.readable_contrast(ends);
    if fill.contrast_ratio(ends.foreground(pick)) >= TEXT_CONTRAST {
        return foreground_var(pick, ends);
    }
    let (black, white) = (HexColor::new(0x00_00_00), HexColor::new(0xFF_FF_FF));
    match fill.contrast_ratio(black) >= fill.contrast_ratio(white) {
        true => black.to_string(),
        false => white.to_string(),
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
            fill_label(fill_base.shade(shade, ramp, ends), ends),
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
