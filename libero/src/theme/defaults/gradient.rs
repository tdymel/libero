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

/// A linear gradient fill. Every key left `None` falls back to the theme's
/// [`GradientDefaults`], so `Gradient::default()` is the theme's gradient.
///
/// A stop is a palette colour (`"primary"`, `"info.7"`) or a literal CSS one.
/// The label is picked to read on palette and hex stops; any other literal
/// (`"rebeccapurple"`, a `var()`) is the caller's to check.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::Button;
/// # use libero::theme::Gradient;
/// # fn app() -> Element {
/// # rsx! {
/// Button { variant: "gradient", gradient: Gradient::default().to("info").deg(90), "Upgrade" }
/// # } }
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Gradient {
    pub from: Option<ThemeAwareValue>,
    pub to: Option<ThemeAwareValue>,
    pub deg: Option<u16>,
}

impl Gradient {
    pub fn from(mut self, stop: impl Into<ThemeAwareValue>) -> Self {
        self.from = Some(stop.into());
        self
    }

    pub fn to(mut self, stop: impl Into<ThemeAwareValue>) -> Self {
        self.to = Some(stop.into());
        self
    }

    pub fn deg(mut self, deg: u16) -> Self {
        self.deg = Some(deg);
        self
    }

    fn stops(&self, defaults: &GradientDefaults) -> [ThemeAwareValue; 2] {
        // A bare palette name is its `S6`, as on every variant's `color`.
        let stop = |stop: &Option<ThemeAwareValue>, color| match stop {
            None => ThemeAwareValue::ColorValue(ColorValue::Shade(color, ColorShade::S6)),
            Some(ThemeAwareValue::Color(color)) => {
                ThemeAwareValue::ColorValue(ColorValue::Shade(*color, ColorShade::S6))
            }
            Some(other) => other.clone(),
        };
        [stop(&self.from, defaults.from), stop(&self.to, defaults.to)]
    }

    /// The five vars for this gradient, measured against every theme a
    /// scheme switch can show without a re-render. `text` resolves palette
    /// stops in the text role, for gradient text on the page.
    pub(crate) fn declarations(&self, themes: &[&Theme], text: bool) -> Vec<CssDeclaration> {
        let active = themes[0];
        let stops = self.stops(&active.gradient);
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

/// One theme's `:root` vars, from its own [`GradientDefaults`].
pub(crate) fn theme_declarations(theme: &Theme) -> Vec<CssDeclaration> {
    Gradient::default().declarations(&[theme], false)
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

/// The label and its state-layer tint: the candidate whose worst contrast on
/// `from`, `to` and the midpoint, across `themes`, is highest. The midpoint
/// matters because luminance sags between two stops: it can sit darker than
/// both, and fail a black label they both pass. A theme's own ends go first,
/// so they follow the scheme; a literal black or white wins only where they
/// cannot serve both schemes.
fn pick_label(stops: &[ThemeAwareValue; 2], themes: &[&Theme], text: bool) -> (String, String) {
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
                let from = stop_hex(&stops[0], theme, text)?;
                let to = stop_hex(&stops[1], theme, text)?;
                let label = paint(theme);
                [from, to, midpoint(from, to)]
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
    // Nothing measurable: Mantine's white, on the caller.
    let (score, label, layer) = best.unwrap_or((f32::MAX, &white, &black));
    if score < TEXT_CONTRAST && !text {
        warn_once(format!(
            "gradient: no label reads at 4.5:1 on {stops:?} (best {score:.2}:1); pick closer stops."
        ));
    }
    (label.to_string(), layer.to_string())
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

    /// The default gradient's label reads at 4.5:1 on every point, in both schemes.
    #[test]
    fn the_default_gradient_label_reads_in_both_schemes() {
        let themes = [&Theme::DEFAULT, &Theme::DARK];
        let stops = Gradient::default().stops(&GradientDefaults::DEFAULT);
        let (label, _) = pick_label(&stops, &themes, false);
        assert_eq!(label, NamedColorCss::SURFACE.value());
        for theme in themes {
            let ratio = worst(&stops, theme, theme.surface);
            assert!(ratio >= TEXT_CONTRAST, "{ratio}");
        }
    }

    /// A light pair takes a dark label, which the midpoint is checked for too.
    #[test]
    fn a_light_literal_pair_takes_black() {
        let stops = Gradient::default()
            .from("#ffe066")
            .to("#8ce99a")
            .stops(&GradientDefaults::DEFAULT);
        let (label, layer) = pick_label(&stops, &[&Theme::DEFAULT, &Theme::DARK], false);
        // Ink in light, surface in dark: only a literal serves both.
        assert_eq!((label.as_str(), layer.as_str()), ("#000000", "#FFFFFF"));
    }

    /// Navy wants white and yellow black: nothing reads on both, and it says so.
    #[test]
    fn a_pair_no_label_reads_on_warns() {
        crate::utils::take_warnings();
        let stops = Gradient::default()
            .from("#1a1a80")
            .to("#ffe066")
            .stops(&GradientDefaults::DEFAULT);
        pick_label(&stops, &[&Theme::DEFAULT, &Theme::DARK], false);
        let warnings = crate::utils::take_warnings();
        assert!(warnings.iter().any(|w| w.contains("4.5:1")), "{warnings:?}");
    }

    /// Pale yellow text fails on the light page; the default palette text passes.
    #[test]
    fn gradient_text_warns_on_a_pale_literal_stop() {
        crate::utils::take_warnings();
        let themes = [&Theme::DEFAULT, &Theme::DARK];
        Gradient::default().declarations(&themes, true);
        assert!(crate::utils::take_warnings().is_empty());

        Gradient::default()
            .from("#ffe066")
            .declarations(&themes, true);
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
        Gradient::default()
            .from("#1a1a80")
            .to("#5c1a80")
            .declarations(&[&Theme::DEFAULT], true);
        assert!(crate::utils::take_warnings().is_empty());
    }

    #[test]
    fn an_unmeasurable_stop_falls_back_to_white() {
        let stops = Gradient::default()
            .from("rebeccapurple")
            .stops(&GradientDefaults::DEFAULT);
        assert_eq!(pick_label(&stops, &[&Theme::DEFAULT], false).0, "#FFFFFF");
    }

    #[test]
    fn declarations_resolve_palette_stops_in_their_role() {
        let gradient = Gradient::default().to("info").deg(90);
        let css: Vec<String> = gradient
            .declarations(&[&Theme::DEFAULT], false)
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
            .declarations(&[&Theme::DEFAULT], true)
            .iter()
            .map(ToString::to_string)
            .collect();
        assert!(
            text.contains(&"--lsx-gradient-from:var(--lsx-primary-text-6);".to_string()),
            "{text:?}"
        );
    }
}
