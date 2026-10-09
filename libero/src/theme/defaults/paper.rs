use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{FORCED_COLORS, Sx, sx};
use crate::theme::{
    Color, ColorShade, ColorValue, CssVar, FOCUS_RING_HALO, GRADIENT_FROM, NamedColorCss, Size,
    SizeCss, gradient_image,
};

/// The library's one surface colour: anything painted as paper reads this.
pub const PAPER_BACKGROUND: CssVar = CssVar::new("--lsx-paper-background");
pub const PAPER_BORDER_COLOR: CssVar = CssVar::new("--lsx-paper-border-color");
/// What reads against [`PAPER_BACKGROUND`], published as `--lsx-focus-contrast`
/// by [`PaperDefaults::theme_vars`].
pub const PAPER_CONTRAST: CssVar = CssVar::new("--lsx-paper-contrast");
pub const PAPER_RADIUS: CssVar = CssVar::new("--lsx-paper-radius");
pub const PAPER_SHADOW: CssVar = CssVar::new("--lsx-paper-shadow");
/// [`PAPER_BACKGROUND`] mixed with transparency.
pub const GLASS_BACKGROUND: CssVar = CssVar::new("--lsx-glass-background");
/// The `backdrop-filter` behind a `glass` surface.
pub const GLASS_BLUR: CssVar = CssVar::new("--lsx-glass-blur");
/// `glass_background` as a percentage, for mixing a gradient's stops.
pub(crate) const GLASS_SHARE: CssVar = CssVar::new("--lsx-glass-share");

/// The white sheen over a coloured glass, thinned per surface where its label needs it.
pub(crate) const GLASS_SHEEN: CssVar = CssVar::new("--lsx-glass-sheen");
/// The full sheen, in percent; `glass_tint` measures the label with it.
pub(crate) const GLASS_SHEEN_MAX: u8 = 18;
const GLASS_HIGHLIGHT: u8 = 40;

/// A one-pixel top highlight and a light sheen, painted over a coloured glass.
fn glass_cue_layers() -> String {
    let white =
        |share: &dyn std::fmt::Display| format!("color-mix(in srgb, white {share}, transparent)");
    let highlight = white(&format!("{GLASS_HIGHLIGHT}%"));
    let sheen = GLASS_SHEEN.value_or(format!("{GLASS_SHEEN_MAX}%"));
    format!(
        "linear-gradient({highlight}, {highlight}) top / 100% 1px no-repeat, \
         linear-gradient({}, transparent)",
        white(&sheen)
    )
}

fn under_tint(tint: Option<&str>, layers: String) -> String {
    match tint {
        Some(tint) => format!("{tint}, {layers}"),
        None => layers,
    }
}

/// The theme's blur, saturated so the backdrop's colour bleeds through.
fn glass_cue_filter() -> String {
    format!("{} saturate(160%)", GLASS_BLUR.value())
}

/// Not Baseline yet, so [`FORCED_COLORS`] and the native branch back it up.
const REDUCED_TRANSPARENCY: &str = "(prefers-reduced-transparency: reduce)";

/// Theme defaults for `Paper`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaperDefaults {
    pub radius: Size,
    pub shadow: Size,
    /// A CSS colour: the palette has no "surface itself" shade yet.
    pub background: &'static str,
    /// What reads against `background`; not derivable from arbitrary CSS.
    /// Change one and change the other.
    pub contrast: ColorValue,
    pub border_color: ColorValue,
    /// Percent of `background` a `glass` surface keeps. Keep it at 70 or more
    /// for text contrast.
    pub glass_background: u8,
    /// The `backdrop-filter` behind a `glass` surface, e.g. `"blur(12px)"`.
    pub glass_blur: &'static str,
}

impl PaperDefaults {
    pub const DEFAULT: Self = Self {
        radius: Size::Md,
        shadow: Size::Sm,
        background: "#fff",
        contrast: ColorValue::Shade(Color::Ink, ColorShade::S1),
        border_color: ColorValue::Shade(Color::Muted, ColorShade::S3),
        glass_background: 80,
        glass_blur: "blur(12px)",
    };

    /// The card on an inked page, one step off the surface.
    pub const DARK: Self = Self {
        background: "#25262B",
        ..Self::DEFAULT
    };

    fn radius_sx(radius: Size) -> Sx {
        sx().border_radius(SizeCss::RADIUS.overridable(radius))
    }

    fn shadow_sx(shadow: Size) -> Sx {
        sx().box_shadow(SizeCss::SHADOW.value(shadow))
    }

    /// The surface colour for non-`Paper` surfaces. Never `background(PAPER_BACKGROUND.value())`
    /// alone: it would lose `--lsx-focus-contrast` and the ring's halo.
    pub(crate) fn background_sx() -> Sx {
        sx().background(PAPER_BACKGROUND.value())
            .var(
                CssVar::Owned(NamedColorCss::FOCUS_CONTRAST.name().to_string()),
                PAPER_CONTRAST.value(),
            )
            .var(FOCUS_RING_HALO, PAPER_BACKGROUND.value())
    }

    /// A surface with no `radius`/`shadow` carries no `data-state`; the folds apply once a caller names a step.
    pub fn theme_vars() -> Sx {
        Self::background_sx()
            .border_radius(PAPER_RADIUS.value())
            .box_shadow(PAPER_SHADOW.value())
            .per_radius(Self::radius_sx)
            .per_shadow(Self::shadow_sx)
    }

    /// Translucent plus a blur; opaque under reduced transparency and forced colours.
    pub(crate) fn glass_sx() -> Sx {
        let opaque = sx()
            .background(PAPER_BACKGROUND.value())
            .backdrop_filter("none");
        sx().background(GLASS_BACKGROUND.value())
            .backdrop_filter(GLASS_BLUR.value())
            .media(REDUCED_TRANSPARENCY, opaque.clone())
            .media(FORCED_COLORS, opaque)
    }

    /// Glass over a flat `fill`, mixed down to `share` (`glass_background` if `None`),
    /// with the glass cues; opaque where [`glass_sx`](Self::glass_sx) turns opaque.
    pub(crate) fn glass_fill_sx(fill: &str, share: Option<&CssVar>) -> Sx {
        Self::glass_fill_layers(fill, share, None)
    }

    /// [`glass_fill_sx`](Self::glass_fill_sx) with a state `tint` layer on top, so a
    /// hovered or pressed glass keeps its cues.
    pub(crate) fn glass_fill_state_sx(fill: &str, share: Option<&CssVar>, tint: &str) -> Sx {
        Self::glass_fill_layers(fill, share, Some(tint))
    }

    fn glass_fill_layers(fill: &str, share: Option<&CssVar>, tint: Option<&str>) -> Sx {
        let opaque = sx()
            .background(under_tint(tint, fill.to_string()))
            .backdrop_filter("none");
        let share = match share {
            Some(share) => share.value_or(GLASS_SHARE.value()),
            None => GLASS_SHARE.value(),
        };
        sx().background(under_tint(
            tint,
            format!(
                "{}, color-mix(in srgb, {fill} {share}, transparent)",
                glass_cue_layers()
            ),
        ))
        .backdrop_filter(glass_cue_filter())
        .media(REDUCED_TRANSPARENCY, opaque.clone())
        .media(FORCED_COLORS, opaque)
    }

    /// Glass over a gradient: each stop mixed down to the glass share, with the glass
    /// cues, and the opaque gradient again wherever [`glass_sx`](Self::glass_sx) turns opaque.
    pub(crate) fn glass_gradient_sx() -> Sx {
        Self::glass_gradient_layers(None)
    }

    /// [`glass_gradient_sx`](Self::glass_gradient_sx) with a state `tint` layer on top.
    pub(crate) fn glass_gradient_state_sx(tint: &str) -> Sx {
        Self::glass_gradient_layers(Some(tint))
    }

    fn glass_gradient_layers(tint: Option<&str>) -> Sx {
        let share = GLASS_SHARE.value();
        let opaque = sx()
            .background(under_tint(
                tint,
                format!("{}, {}", gradient_image(None), GRADIENT_FROM.value()),
            ))
            .backdrop_filter("none");
        sx().background(under_tint(
            tint,
            format!(
                "{}, {}, color-mix(in srgb, {} {share}, transparent)",
                glass_cue_layers(),
                gradient_image(Some(&share)),
                GRADIENT_FROM.value()
            ),
        ))
        .backdrop_filter(glass_cue_filter())
        .media(REDUCED_TRANSPARENCY, opaque.clone())
        .media(FORCED_COLORS, opaque)
    }
}

impl ToCssDeclarations for PaperDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            PAPER_RADIUS.declare(SizeCss::RADIUS.value(self.radius)),
            PAPER_SHADOW.declare(SizeCss::SHADOW.value(self.shadow)),
            PAPER_BACKGROUND.declare(self.background),
            PAPER_CONTRAST.declare(self.contrast.value()),
            PAPER_BORDER_COLOR.declare(self.border_color.value()),
            GLASS_BACKGROUND.declare(format!(
                "color-mix(in srgb, {} {}%, transparent)",
                PAPER_BACKGROUND.value(),
                self.glass_background
            )),
            GLASS_BLUR.declare(self.glass_blur),
            GLASS_SHARE.declare(format!("{}%", self.glass_background)),
        ]
    }
}
