use super::paper::{GLASS_SHARE, GLASS_SHEEN, GLASS_SHEEN_MAX};
use crate::css::CssDeclaration;
use crate::sx::{ColorRole, Sx, ThemeAwareValue, sx};
use crate::theme::{
    Color, ColorShade, ColorValue, CssVar, FOCUS_RING_HALO, HexColor, NamedColorCss, Theme,
};
use crate::tokens::{Ends, ShadeRamp, TEXT_CONTRAST};
use crate::utils::warn;

/// The gradient's first stop, and the solid fill wherever the image is dropped.
pub const GRADIENT_FROM: CssVar = CssVar::new("--lsx-gradient-from");
pub const GRADIENT_TO: CssVar = CssVar::new("--lsx-gradient-to");
pub const GRADIENT_ANGLE: CssVar = CssVar::new("--lsx-gradient-angle");
/// The label on the gradient: whichever end of the page reads on every point of it.
pub const GRADIENT_CONTRAST: CssVar = CssVar::new("--lsx-gradient-contrast");
/// The other end: the hover and selected state layers are tinted with it.
pub const GRADIENT_LAYER: CssVar = CssVar::new("--lsx-gradient-layer");
/// A fill's label for an uncoloured `standard` Button on it, set where `ANCHOR_COLOR`
/// follows the fill (todo 1663); `initial` on a plain surface, so the button keeps its own.
pub(crate) const SURFACE_LABEL: CssVar = CssVar::new("--lsx-surface-label");

// A literal gradient does not flip with the scheme, so its label may not either.
const BLACK: HexColor = HexColor::new(0x00_00_00);
const WHITE: HexColor = HexColor::new(0xFF_FF_FF);

// M3's state layer opacities.
const HOVER_LAYER: u8 = 8;
const SELECTED_LAYER: u8 = 12;

/// The theme's gradient, the one every `gradient` surface falls back to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GradientDefaults {
    pub from: Color,
    pub to: Color,
    /// The CSS angle, in degrees: `0` runs upwards, `90` to the right.
    pub deg: u16,
}

impl GradientDefaults {
    pub const DEFAULT: Self = Self {
        from: Color::Primary,
        to: Color::Secondary,
        deg: 45,
    };
}

/// A linear gradient fill from the component's `color` to `to`. Every key
/// left `None` falls back to the theme's [`GradientDefaults`] (an unset
/// `color` to `GradientDefaults::from`), so `Gradient::default()` is the
/// theme's gradient.
///
/// A stop is a palette colour (`"primary"`, `"info.7"`) or a literal CSS one.
/// The label is picked to read on palette and hex stops; any other literal
/// (`"rebeccapurple"`, a `var()`) is the caller's to check, and debug builds warn.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::Button;
/// # use libero::theme::Gradient;
/// # fn app() -> Element {
/// # rsx! {
/// Button { variant: "gradient", color: "info", gradient: ("secondary", 90), "Upgrade" }
/// Button { variant: "gradient", gradient: Gradient::default().to("info"), "Renew" }
/// # } }
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Gradient {
    pub to: Option<ThemeAwareValue>,
    pub deg: Option<u16>,
}

impl Gradient {
    pub fn to(mut self, stop: impl Into<ThemeAwareValue>) -> Self {
        self.to = Some(stop.into());
        self
    }

    pub fn deg(mut self, deg: u16) -> Self {
        self.deg = Some(deg);
        self
    }

    /// `from` is the component's `color`, the first stop.
    fn stops(
        &self,
        from: Option<&ThemeAwareValue>,
        defaults: &GradientDefaults,
    ) -> [ThemeAwareValue; 2] {
        // A bare palette name is its `S6`, as on every variant's `color`.
        let stop = |stop: Option<&ThemeAwareValue>, color| match stop {
            None => ThemeAwareValue::ColorValue(ColorValue::Shade(color, ColorShade::S6)),
            Some(ThemeAwareValue::Color(color)) => {
                ThemeAwareValue::ColorValue(ColorValue::Shade(*color, ColorShade::S6))
            }
            Some(other) => other.clone(),
        };
        [
            stop(from, defaults.from),
            stop(self.to.as_ref(), defaults.to),
        ]
    }

    /// The five vars for this gradient from `from`, measured against every
    /// theme a scheme switch can show without a re-render. `text` resolves
    /// palette stops in the text role, for gradient text on the page.
    pub(crate) fn declarations(
        &self,
        from: Option<&ThemeAwareValue>,
        themes: &[&Theme],
        text: bool,
    ) -> Vec<CssDeclaration> {
        let active = themes[0];
        let stops = self.stops(from, &active.gradient);
        let role = if text {
            ColorRole::Text
        } else {
            ColorRole::Fill
        };
        let css = |stop: &ThemeAwareValue| stop.in_color_role(role).resolve(None);
        let (label, layer) = pick_label(&stops, themes, text);
        if text && cfg!(debug_assertions) {
            check_text_stops(&stops, themes);
        }
        vec![
            GRADIENT_FROM.declare(css(&stops[0]).unwrap_or_default()),
            GRADIENT_TO.declare(css(&stops[1]).unwrap_or_default()),
            GRADIENT_ANGLE.declare(format!("{}deg", self.deg.unwrap_or(active.gradient.deg))),
            GRADIENT_CONTRAST.declare(label),
            GRADIENT_LAYER.declare(layer),
        ]
    }
}

/// `gradient: ("secondary", 45)`: the second stop and the angle.
impl<T: Into<ThemeAwareValue>> From<(T, u16)> for Gradient {
    fn from((to, deg): (T, u16)) -> Self {
        Self::default().to(to).deg(deg)
    }
}

// Sealed: only `#[props(into)]` names it, through the impl below.
#[doc(hidden)]
#[allow(unnameable_types)]
pub struct GradientTupleMarker;

// The tuple on an `Option<Gradient>` prop; `Option` only takes `From<Gradient>` itself.
impl<T: Into<ThemeAwareValue>> dioxus::core::SuperFrom<(T, u16), GradientTupleMarker>
    for Option<Gradient>
{
    fn super_from(value: (T, u16)) -> Self {
        Some(value.into())
    }
}

/// One theme's `:root` vars, from its own [`GradientDefaults`].
pub(crate) fn theme_declarations(theme: &Theme) -> Vec<CssDeclaration> {
    Gradient::default().declarations(None, &[theme], false)
}

/// `linear-gradient(...)` over the gradient vars. `share` mixes each stop
/// with `transparent`, for a glass surface.
pub(crate) fn gradient_image(share: Option<&str>) -> String {
    let stop = |var: CssVar| match share {
        Some(share) => format!("color-mix(in srgb, {} {share}, transparent)", var.value()),
        None => var.value(),
    };
    format!(
        "linear-gradient({}, {}, {})",
        GRADIENT_ANGLE.value(),
        stop(GRADIENT_FROM),
        stop(GRADIENT_TO)
    )
}

/// The gradient as a fill, the first stop under it for a renderer that drops
/// the image (forced colours compute it to `none`), and the label.
pub(crate) fn gradient_fill_sx() -> Sx {
    sx().background_color(GRADIENT_FROM.value())
        .background_image(gradient_image(None))
        .color(GRADIENT_CONTRAST.value())
}

/// A gradient surface (`Paper`, `Header`): the fill, and rings inside take
/// the label, which reads on every point of it.
pub(crate) fn gradient_surface_sx() -> Sx {
    gradient_fill_sx()
        .var(
            CssVar::Owned(NamedColorCss::FOCUS_CONTRAST.name().to_string()),
            GRADIENT_CONTRAST.value(),
        )
        .var(FOCUS_RING_HALO, GRADIENT_FROM.value())
        .var(SURFACE_LABEL, GRADIENT_CONTRAST.value())
}

fn state_layer(percent: u8) -> String {
    let layer = format!(
        "color-mix(in srgb, {} {percent}%, transparent)",
        GRADIENT_LAYER.value()
    );
    format!(
        "linear-gradient({layer}, {layer}), {}",
        gradient_image(None)
    )
}

/// The hover state layer: the far end over the fill, which raises the label's contrast.
pub(crate) fn gradient_hover_sx() -> Sx {
    sx().background_image(state_layer(HOVER_LAYER))
}

pub(crate) fn gradient_selected_sx() -> Sx {
    sx().background_image(state_layer(SELECTED_LAYER))
}

/// The theme's base colour for `color`, `None` for a colour with no hex.
fn palette(theme: &Theme, color: Color) -> Option<HexColor> {
    Some(match color {
        Color::Primary => theme.primary,
        Color::Secondary => theme.secondary,
        Color::Error => theme.error,
        Color::Warning => theme.warning,
        Color::Info => theme.info,
        Color::Success => theme.success,
        Color::Neutral => theme.neutral,
        Color::Muted => theme.muted,
        Color::Ink => theme.ink,
        Color::Surface => theme.surface,
    })
}

/// What `stop` paints as in `theme`, as the stylesheet derives it.
fn stop_hex(stop: &ThemeAwareValue, theme: &Theme, text: bool) -> Option<HexColor> {
    let ends = Ends {
        surface: theme.surface,
        ink: theme.ink,
    };
    match stop {
        ThemeAwareValue::ColorValue(
            ColorValue::Shade(color, shade) | ColorValue::Fill(color, shade),
        ) => {
            let base = palette(theme, *color)?;
            if matches!(color, Color::Ink | Color::Surface) {
                return Some(base);
            }
            let ramp = color.shade_ramp();
            let rebased = match (text, ramp) {
                (true, ShadeRamp::Chromatic) => base.text_base(ramp, ends, &[]),
                (true, ShadeRamp::Neutral) => base,
                (false, _) => base.fill_base(ramp, ends),
            };
            Some(rebased.shade(*shade, ramp, ends))
        }
        ThemeAwareValue::RawColor(_, hex) => Some(*hex),
        _ => None,
    }
}

fn midpoint(a: HexColor, b: HexColor) -> HexColor {
    let mid = |x: u8, y: u8| (x as u32 + y as u32).div_ceil(2);
    HexColor::new((mid(a.r(), b.r()) << 16) | (mid(a.g(), b.g()) << 8) | mid(a.b(), b.b()))
}

/// Both stops and the midpoint in `theme`; only the measurable stop where the other has no hex.
fn gradient_points(
    stops: &[ThemeAwareValue; 2],
    theme: &Theme,
    text: bool,
) -> Option<Vec<HexColor>> {
    match (
        stop_hex(&stops[0], theme, text),
        stop_hex(&stops[1], theme, text),
    ) {
        (Some(from), Some(to)) => Some(vec![from, to, midpoint(from, to)]),
        (Some(only), None) | (None, Some(only)) => Some(vec![only]),
        (None, None) => None,
    }
}

/// The label, its opposite end and the worst ratio, best over every point of every theme.
/// Theme ends first; literals only where they fail. `None`: nothing measurable.
fn best_label(
    themes: &[&Theme],
    points: impl Fn(&Theme) -> Option<Vec<HexColor>>,
) -> Option<(f32, String, String)> {
    let (ink, surface) = (NamedColorCss::INK.value(), NamedColorCss::SURFACE.value());
    let (black, white) = (BLACK.to_string(), WHITE.to_string());
    type Paint = fn(&Theme) -> HexColor;
    let candidates: [(&str, &str, Paint); 4] = [
        (&surface, &ink, |theme| theme.surface),
        (&ink, &surface, |theme| theme.ink),
        (&white, &black, |_| WHITE),
        (&black, &white, |_| BLACK),
    ];

    let score = |paint: Paint| {
        themes
            .iter()
            .filter_map(|theme| {
                let label = paint(theme);
                points(theme)?
                    .into_iter()
                    .map(|point| point.contrast_ratio(label))
                    .reduce(f32::min)
            })
            .reduce(f32::min)
    };

    let mut best: Option<(f32, &str, &str)> = None;
    for (label, layer, paint) in candidates {
        let Some(score) = score(paint) else { continue };
        if best.is_none_or(|(top, ..)| score > top) {
            best = Some((score, label, layer));
        }
    }
    best.map(|(score, label, layer)| (score, label.to_string(), layer.to_string()))
}

/// The label and layer tint with the best worst-case contrast on both stops and the
/// midpoint (luminance sags there). A stop with no hex (`"violet"`, a var) drops out
/// with the midpoint, and says so.
fn pick_label(stops: &[ThemeAwareValue; 2], themes: &[&Theme], text: bool) -> (String, String) {
    let best = best_label(themes, |theme| gradient_points(stops, theme, text));
    if !text {
        let theme = themes[0];
        if let Some(stop) = stops
            .iter()
            .find(|stop| stop_hex(stop, theme, false).is_none())
        {
            warn_once(format!(
                "gradient: the label is not measured on {stop:?}; give the stop as a palette colour or hex."
            ));
        }
    }
    // Nothing measurable: Mantine's white, on the caller.
    let (score, label, layer) = best.unwrap_or((f32::MAX, WHITE.to_string(), BLACK.to_string()));
    if score < TEXT_CONTRAST && !text {
        warn_once(format!(
            "gradient: no label reads at 4.5:1 on {stops:?} (best {score:.2}:1); pick closer stops."
        ));
    }
    (label, layer)
}

/// What a coloured glass needs to stay legible: the label, the tint's share of the
/// fill and the white sheen (percent) left on it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlassTint {
    pub label: String,
    pub share: u8,
    pub sheen: u8,
}

/// The [`GlassTint`] of `fill`: the label reads on the solid fill (the opaque fallbacks)
/// and on the tint, with its sheen, over the page's surface. The share is the theme's
/// `glass_background`, raised in steps of 5% until it does; where even the solid fill
/// cannot carry the full sheen, the sheen is thinned first.
/// `None` for a literal with no hex.
pub(crate) fn glass_tint(fill: &ThemeAwareValue, themes: &[&Theme]) -> Option<GlassTint> {
    glass_share(fill, themes, |theme| {
        Some(vec![stop_hex(fill, theme, false)?])
    })
}

/// [`glass_tint`] of a gradient from `from`: every point of the gradient is a fill.
pub(crate) fn glass_gradient_tint(
    gradient: &Gradient,
    from: Option<&ThemeAwareValue>,
    themes: &[&Theme],
) -> Option<GlassTint> {
    let stops = gradient.stops(from, &themes[0].gradient);
    glass_share(&stops, themes, |theme| {
        gradient_points(&stops, theme, false)
    })
}

/// The gradient's label, glass share and sheen as vars, over [`Gradient::declarations`].
pub(crate) fn glass_gradient_declarations(
    gradient: &Gradient,
    from: Option<&ThemeAwareValue>,
    themes: &[&Theme],
) -> Vec<CssDeclaration> {
    glass_gradient_tint(gradient, from, themes)
        .map(|tint| {
            vec![
                GRADIENT_CONTRAST.declare(tint.label),
                GLASS_SHARE.declare(format!("{}%", tint.share)),
                GLASS_SHEEN.declare(format!("{}%", tint.sheen)),
            ]
        })
        .unwrap_or_default()
}

/// The search behind [`glass_tint`]: per sheen, thinning from [`GLASS_SHEEN_MAX`], the lowest
/// share at which a label reads 4.5:1 on every `fills` point, its tint and the sheen on it.
fn glass_share(
    what: &impl std::fmt::Debug,
    themes: &[&Theme],
    fills: impl Fn(&Theme) -> Option<Vec<HexColor>>,
) -> Option<GlassTint> {
    let start = themes
        .iter()
        .map(|theme| theme.paper.glass_background)
        .max()?
        .min(100);
    let at = |share: u8, sheen: u8| {
        best_label(themes, |theme| {
            let mut points = Vec::new();
            for solid in fills(theme)? {
                let tint = mix(solid, theme.surface, share);
                points.extend([solid, tint, midpoint(solid, tint), mix(WHITE, tint, sheen)]);
            }
            Some(points)
        })
    };
    let mut last = None;
    for sheen in (0..=GLASS_SHEEN_MAX).rev().step_by(6) {
        for share in (start..100).step_by(5).chain([100]) {
            let (score, label, _) = at(share, sheen)?;
            if score >= TEXT_CONTRAST {
                return Some(GlassTint {
                    label,
                    share,
                    sheen,
                });
            }
            last = Some((score, label));
        }
    }
    let (score, label) = last?;
    warn_once(format!(
        "Paper: no label reads at 4.5:1 on {what:?} (best {score:.2}:1); pick a darker colour."
    ));
    Some(GlassTint {
        label,
        share: 100,
        sheen: 0,
    })
}

/// `a` at `percent` over `b`, as `color-mix(in srgb, a percent, b)` paints it.
fn mix(a: HexColor, b: HexColor, percent: u8) -> HexColor {
    let share = f32::from(percent.min(100)) / 100.0;
    let mix = |x: u8, y: u8| (f32::from(x) * share + f32::from(y) * (1.0 - share)).round() as u32;
    HexColor::new((mix(a.r(), b.r()) << 16) | (mix(a.g(), b.g()) << 8) | mix(a.b(), b.b()))
}

/// Gradient text with a literal stop: warns where a stop, or the midpoint,
/// falls under 4.5:1 on the page background. Palette stops come off the text ramp.
fn check_text_stops(stops: &[ThemeAwareValue; 2], themes: &[&Theme]) {
    if !stops
        .iter()
        .any(|stop| matches!(stop, ThemeAwareValue::RawColor(..)))
    {
        return;
    }
    let worst = themes
        .iter()
        .filter_map(|theme| {
            let from = stop_hex(&stops[0], theme, true)?;
            let to = stop_hex(&stops[1], theme, true)?;
            [from, to, midpoint(from, to)]
                .into_iter()
                .map(|point| point.contrast_ratio(theme.surface))
                .reduce(f32::min)
        })
        .reduce(f32::min);
    if let Some(worst) = worst.filter(|worst| *worst < TEXT_CONTRAST) {
        warn_once(format!(
            "gradient text: {stops:?} reads at {worst:.2}:1 on the page background, under 4.5:1; \
             pick darker (or, in dark schemes, lighter) stops."
        ));
    }
}

/// Once per message: the check reruns on every render.
fn warn_once(message: String) {
    thread_local! {
        static WARNED: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    }
    let first = WARNED.with_borrow_mut(|warned| {
        let first = !warned.contains(&message);
        if first {
            warned.push(message.clone());
        }
        first
    });
    if first {
        warn(&message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn worst(stops: &[ThemeAwareValue; 2], theme: &Theme, label: HexColor) -> f32 {
        let from = stop_hex(&stops[0], theme, false).unwrap();
        let to = stop_hex(&stops[1], theme, false).unwrap();
        [from, to, midpoint(from, to)]
            .into_iter()
            .map(|point| point.contrast_ratio(label))
            .fold(f32::MAX, f32::min)
    }

    /// The stops of `color` into `gradient`, over the default theme gradient.
    fn stops(color: Option<&str>, gradient: Gradient) -> [ThemeAwareValue; 2] {
        let color = color.map(ThemeAwareValue::from);
        gradient.stops(color.as_ref(), &GradientDefaults::DEFAULT)
    }

    /// Todo 2051: every shipped set's own gradient carries a 4.5:1 label in both its schemes.
    #[test]
    fn every_shipped_gradient_label_reads_in_both_schemes() {
        use crate::theme::ThemeSet;

        let mut short = Vec::new();
        for set in ThemeSet::CATALOGUE {
            let themes: Vec<&Theme> = [Some(set.light_theme()), set.dark_theme()]
                .into_iter()
                .flatten()
                .collect();
            for active in &themes {
                let stops = Gradient::default().stops(None, &active.gradient);
                let (score, ..) =
                    best_label(&themes, |theme| gradient_points(&stops, theme, false)).unwrap();
                if score < TEXT_CONTRAST {
                    short.push(format!("{}: {score:.2}:1", set.name()));
                }
            }
        }
        assert!(short.is_empty(), "{short:?}");
    }

    /// The default gradient's label reads at 4.5:1 on every point, in both schemes.
    #[test]
    fn the_default_gradient_label_reads_in_both_schemes() {
        let themes = [&Theme::DEFAULT, &Theme::DARK];
        let stops = stops(None, Gradient::default());
        let (label, _) = pick_label(&stops, &themes, false);
        assert_eq!(label, NamedColorCss::SURFACE.value());
        for theme in themes {
            let ratio = worst(&stops, theme, theme.surface);
            assert!(ratio >= TEXT_CONTRAST, "{ratio}");
        }
    }

    /// `color` is the first stop, `to` the second; unset, both are the theme's.
    #[test]
    fn color_is_the_first_stop_and_unset_falls_back_to_the_theme() {
        let s6 = |color| ThemeAwareValue::ColorValue(ColorValue::Shade(color, ColorShade::S6));
        assert_eq!(
            stops(None, Gradient::default()),
            [s6(Color::Primary), s6(Color::Secondary)]
        );
        assert_eq!(
            stops(Some("error"), ("info", 90).into()),
            [s6(Color::Error), s6(Color::Info)]
        );
        let literal = stops(Some("#ffe066"), Gradient::default());
        assert_eq!(literal[0], ThemeAwareValue::from("#ffe066"));
    }

    /// A glass tint's label reads on the tint over the page and on the opaque fallback,
    /// light and dark, for every chromatic palette colour; the share only rises where needed.
    #[test]
    fn a_glass_tint_label_reads_on_the_tint_and_the_solid_fill() {
        crate::utils::take_warnings();
        let themes = [&Theme::DEFAULT, &Theme::DARK];
        for color in [
            Color::Primary,
            Color::Secondary,
            Color::Error,
            Color::Warning,
            Color::Info,
            Color::Success,
        ] {
            let fill = ThemeAwareValue::ColorValue(ColorValue::Shade(color, ColorShade::S6));
            let GlassTint {
                label,
                share,
                sheen,
            } = glass_tint(&fill, &themes).unwrap();
            for theme in themes {
                let paint = match label.as_str() {
                    "#000000" => BLACK,
                    "#FFFFFF" => WHITE,
                    label if label == NamedColorCss::INK.value() => theme.ink,
                    _ => theme.surface,
                };
                let solid = stop_hex(&fill, theme, false).unwrap();
                let tint = mix(solid, theme.surface, share);
                for point in [solid, tint, mix(WHITE, tint, sheen)] {
                    let ratio = point.contrast_ratio(paint);
                    assert!(ratio >= TEXT_CONTRAST, "{color:?}: {ratio}");
                }
            }
        }
        assert!(crate::utils::take_warnings().is_empty());
    }

    /// The label reads 4.5:1 on every gradient point at the raised glass share, sheen included.
    #[test]
    fn a_glass_gradient_label_reads_on_the_tint_and_the_sheen() {
        crate::utils::take_warnings();
        let themes = [&Theme::DEFAULT, &Theme::DARK];
        for (color, to) in [
            ("primary", "secondary"),
            ("warning", "info"),
            ("error", "success"),
        ] {
            let gradient = Gradient::default().to(to);
            let from = ThemeAwareValue::from(color);
            let GlassTint {
                label,
                share,
                sheen,
            } = glass_gradient_tint(&gradient, Some(&from), &themes).unwrap();
            let stops = gradient.stops(Some(&from), &GradientDefaults::DEFAULT);
            for theme in themes {
                let paint = match label.as_str() {
                    "#000000" => BLACK,
                    "#FFFFFF" => WHITE,
                    label if label == NamedColorCss::INK.value() => theme.ink,
                    _ => theme.surface,
                };
                let a = stop_hex(&stops[0], theme, false).unwrap();
                let b = stop_hex(&stops[1], theme, false).unwrap();
                for solid in [a, b, midpoint(a, b)] {
                    let tint = mix(solid, theme.surface, share);
                    for point in [solid, tint, mix(WHITE, tint, sheen)] {
                        let ratio = point.contrast_ratio(paint);
                        assert!(ratio >= TEXT_CONTRAST, "{color}->{to}: {ratio}");
                    }
                }
            }
        }
        assert!(crate::utils::take_warnings().is_empty());
    }

    /// The share never drops under the theme's `glass_background`.
    #[test]
    fn a_glass_share_starts_at_the_theme_s_glass_background() {
        let fill = ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Primary, ColorShade::S6));
        let GlassTint { share, .. } = glass_tint(&fill, &[&Theme::DEFAULT, &Theme::DARK]).unwrap();
        assert!(share >= Theme::DEFAULT.paper.glass_background);
    }

    /// A mid-tone fill that the full sheen would push under 4.5:1 loses the sheen, not the label.
    #[test]
    fn the_sheen_thins_where_the_fill_cannot_carry_it() {
        let themes = [&Theme::DEFAULT, &Theme::DARK];
        let tint = |color| {
            let fill = ThemeAwareValue::ColorValue(ColorValue::Shade(color, ColorShade::S6));
            glass_tint(&fill, &themes).unwrap()
        };
        assert_eq!(tint(Color::Warning).sheen, GLASS_SHEEN_MAX);
        assert!(tint(Color::Secondary).sheen < GLASS_SHEEN_MAX);
    }

    /// The tuple form is the builder's shorthand.
    #[test]
    fn a_tuple_is_the_second_stop_and_the_angle() {
        let tuple: Gradient = ("secondary", 45).into();
        assert_eq!(tuple, Gradient::default().to("secondary").deg(45));
    }

    /// A palette `color` into another palette stop reads at 4.5:1, light and dark.
    #[test]
    fn a_palette_color_into_a_palette_stop_reads_in_both_schemes() {
        let themes = [&Theme::DEFAULT, &Theme::DARK];
        for (color, to) in [
            ("error", "warning"),
            ("info", "primary"),
            ("success", "info"),
        ] {
            let stops = stops(Some(color), Gradient::default().to(to));
            let (label, _) = pick_label(&stops, &themes, false);
            for theme in themes {
                let paint = match label.as_str() {
                    "#000000" => BLACK,
                    "#FFFFFF" => WHITE,
                    label if label == NamedColorCss::INK.value() => theme.ink,
                    _ => theme.surface,
                };
                let ratio = worst(&stops, theme, paint);
                assert!(ratio >= TEXT_CONTRAST, "{color} to {to}: {ratio}");
            }
        }
    }

    /// A light pair takes a dark label, which the midpoint is checked for too.
    #[test]
    fn a_light_literal_pair_takes_black() {
        let stops = stops(Some("#ffe066"), Gradient::default().to("#8ce99a"));
        let (label, layer) = pick_label(&stops, &[&Theme::DEFAULT, &Theme::DARK], false);
        // Ink in light, surface in dark: only a literal serves both.
        assert_eq!((label.as_str(), layer.as_str()), ("#000000", "#FFFFFF"));
    }

    /// Navy wants white and yellow black: nothing reads on both, and it says so.
    #[test]
    fn a_pair_no_label_reads_on_warns() {
        crate::utils::take_warnings();
        let stops = stops(Some("#1a1a80"), Gradient::default().to("#ffe066"));
        pick_label(&stops, &[&Theme::DEFAULT, &Theme::DARK], false);
        let warnings = crate::utils::take_warnings();
        assert!(warnings.iter().any(|w| w.contains("4.5:1")), "{warnings:?}");
    }

    /// Pale yellow text fails on the light page; the default palette text passes.
    #[test]
    fn gradient_text_warns_on_a_pale_literal_stop() {
        crate::utils::take_warnings();
        let themes = [&Theme::DEFAULT, &Theme::DARK];
        Gradient::default().declarations(None, &themes, true);
        assert!(crate::utils::take_warnings().is_empty());

        Gradient::default().declarations(Some(&"#ffe066".into()), &themes, true);
        let warnings = crate::utils::take_warnings();
        assert!(
            warnings.iter().any(|w| w.contains("gradient text")),
            "{warnings:?}"
        );
    }

    /// A dark literal pair reads on the light page, so nothing is said.
    #[test]
    fn gradient_text_on_readable_literal_stops_stays_quiet() {
        crate::utils::take_warnings();
        Gradient::default().to("#5c1a80").declarations(
            Some(&"#1a1a80".into()),
            &[&Theme::DEFAULT],
            true,
        );
        assert!(crate::utils::take_warnings().is_empty());
    }

    #[test]
    fn an_unmeasurable_stop_falls_back_to_white() {
        let stops = stops(
            Some("rebeccapurple"),
            Gradient::default().to("var(--brand)"),
        );
        assert_eq!(pick_label(&stops, &[&Theme::DEFAULT], false).0, "#FFFFFF");
    }

    /// Todo 1664: a named stop drops out of the pick, which still fits the other stop and warns.
    #[test]
    fn a_named_stop_is_measured_without_and_warns() {
        crate::utils::take_warnings();
        let themes = [&Theme::DEFAULT, &Theme::DARK];
        let stops = stops(Some("#ffe066"), ("violet", 90).into());
        let (label, _) = pick_label(&stops, &themes, false);
        // The pale literal alone wants black, not the white an unmeasured pair falls back to.
        assert_eq!(label, "#000000");
        let warnings = crate::utils::take_warnings();
        assert!(
            warnings
                .iter()
                .any(|w| w.contains("not measured") && w.contains("violet")),
            "{warnings:?}"
        );
    }

    #[test]
    fn declarations_resolve_palette_stops_in_their_role() {
        let gradient = Gradient::from(("info", 90));
        let css: Vec<String> = gradient
            .declarations(None, &[&Theme::DEFAULT], false)
            .iter()
            .map(ToString::to_string)
            .collect();
        assert!(
            css.contains(&"--lsx-gradient-from:var(--lsx-primary-fill-6);".to_string()),
            "{css:?}"
        );
        assert!(
            css.contains(&"--lsx-gradient-to:var(--lsx-info-fill-6);".to_string()),
            "{css:?}"
        );
        assert!(
            css.contains(&"--lsx-gradient-angle:90deg;".to_string()),
            "{css:?}"
        );

        let text: Vec<String> = gradient
            .declarations(Some(&"error".into()), &[&Theme::DEFAULT], true)
            .iter()
            .map(ToString::to_string)
            .collect();
        assert!(
            text.contains(&"--lsx-gradient-from:var(--lsx-error-text-6);".to_string()),
            "{text:?}"
        );
    }
}
