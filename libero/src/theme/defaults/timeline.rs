use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};

use crate::theme::{Color, ColorShade, ColorValue, CssVar, Size, SizeCss, Sizes};

pub const TIMELINE_COLOR: CssVar = CssVar::new("--lsx-timeline-color");
pub const TIMELINE_LINE_COLOR: CssVar = CssVar::new("--lsx-timeline-line-color");
pub const TIMELINE_LINE_WIDTH: CssVar = CssVar::new("--lsx-timeline-line-width");
pub const TIMELINE_BULLET_BACKGROUND: CssVar = CssVar::new("--lsx-timeline-bullet-background");

pub const TIMELINE_BULLET_SIZE: SizeCss = SizeCss::new("--lsx-timeline-bullet-size-");
pub const TIMELINE_GAP: SizeCss = SizeCss::new("--lsx-timeline-gap-");

// Each axis's picked level, resolved on the root so items and bullets inherit it.
pub const TIMELINE_BULLET: CssVar = CssVar::new("--lsx-timeline-bullet");
pub const TIMELINE_SPACE: CssVar = CssVar::new("--lsx-timeline-space");
pub const TIMELINE_RADIUS: CssVar = CssVar::new("--lsx-timeline-radius");

// Per item, so one `::before` rule serves every line style and active state.
pub const TIMELINE_LINE_STYLE: CssVar = CssVar::new("--lsx-timeline-line-style");
pub const TIMELINE_CONNECTOR: CssVar = CssVar::new("--lsx-timeline-connector");
pub const TIMELINE_MARKER: CssVar = CssVar::new("--lsx-timeline-marker");

/// `PAPER_BACKGROUND.value()` as a `const`, since `CssVar::value` formats at runtime.
/// A test pins the two together.
pub const TIMELINE_BULLET_BACKGROUND_DEFAULT: &str = "var(--lsx-paper-background)";

str_enum! {
    /// Which side of the rail content sits on. Logical: `Start` is the right under `dir="rtl"`.
    #[state_prefix = "align"]
    pub enum TimelineAlign {
        /// The rail on the inline-start edge, content after it.
        #[default]
        Start = "start",
        /// The rail on the inline-end edge, content before it, text aligned to the end.
        End = "end",
        /// Content alternates around a centred rail at every width (no narrow fallback),
        /// and the list fills its parent.
        Alternate = "alternate",
    }
}

/// The `data-state` token for the gap axis. `const` for `States::with`, and shared
/// with the gap rule so the two cannot drift.
pub const fn gap_state_name(gap: Size) -> &'static str {
    match gap {
        Size::Xs => "gap-xs",
        Size::Sm => "gap-sm",
        Size::Md => "gap-md",
        Size::Lg => "gap-lg",
        Size::Xl => "gap-xl",
        Size::Xxl => "gap-xxl",
    }
}

/// Theme defaults for `Timeline`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimelineDefaults {
    pub align: TimelineAlign,
    /// `ColorValue`: a raw `"primary.6"` is not CSS and drops every shorthand reading it.
    pub color: ColorValue,
    pub line_color: ColorValue,
    /// The bullet's surface and an active bullet's glyph colour. `PAPER_BACKGROUND`,
    /// not `white`, so dark mode follows `PaperDefaults` (todo 45).
    pub bullet_background: &'static str,
    /// `Xl` is the dot; lower it for a squarer marker.
    pub radius: Size,
    pub bullet_size: Size,
    pub bullet_sizes: Sizes<u16>,
    pub line_width: u8,
    pub gap: Size,
    pub gaps: Sizes<u16>,
}

impl TimelineDefaults {
    pub const DEFAULT: Self = Self {
        align: TimelineAlign::Start,
        color: ColorValue::Shade(Color::Primary, ColorShade::S6),
        // Pending ring and rail: 3:1 on page and dark `Paper` (1.4.11, todo 599).
        line_color: ColorValue::Shade(Color::Muted, ColorShade::S6),
        bullet_background: TIMELINE_BULLET_BACKGROUND_DEFAULT,
        radius: Size::Xl,
        bullet_size: Size::Md,
        bullet_sizes: Sizes::new(12, 16, 20, 24, 28, 32),
        line_width: 2,
        gap: Size::Xl,
        gaps: Sizes::new(12, 16, 24, 32, 40, 48),
    };

    fn size_sx(size: Size) -> Sx {
        sx().var(TIMELINE_BULLET, TIMELINE_BULLET_SIZE.value(size))
    }

    fn radius_sx(radius: Size) -> Sx {
        sx().var(TIMELINE_RADIUS, SizeCss::RADIUS.value(radius))
    }

    /// A hand-rolled `per_size` for the only third size axis; promote it if a second
    /// component needs one.
    fn gap_sx(base: Sx) -> Sx {
        Size::ALL.into_iter().fold(base, |acc, gap| {
            acc.when(
                gap_state_name(gap),
                sx().var(TIMELINE_SPACE, TIMELINE_GAP.value(gap)),
            )
        })
    }

    pub fn theme_vars() -> Sx {
        Self::gap_sx(sx().per_size(Self::size_sx).per_radius(Self::radius_sx))
    }
}

impl ToCssDeclarations for TimelineDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = self
            .bullet_sizes
            .to_css_declarations(TIMELINE_BULLET_SIZE, "px");
        declarations.extend(self.gaps.to_css_declarations(TIMELINE_GAP, "px"));
        // `.value()`: a bare `primary.6` sets the var, so no fallback fires and every
        // `border` reading it is dropped at computed-value time.
        declarations.push(TIMELINE_COLOR.declare(self.color.value()));
        declarations.push(TIMELINE_LINE_COLOR.declare(self.line_color.value()));
        declarations.push(TIMELINE_LINE_WIDTH.declare(format!("{}px", self.line_width)));
        declarations.push(TIMELINE_BULLET_BACKGROUND.declare(self.bullet_background));
        // Items set their own, but an unset var would drop the whole `border-left`
        // shorthand; this keeps the connector for any item that does not.
        declarations.push(TIMELINE_LINE_STYLE.declare("solid"));
        declarations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::PAPER_BACKGROUND;

    /// The bullet is filled with its surface, so its default must *be* `PAPER_BACKGROUND`.
    #[test]
    fn a_bullet_sits_on_the_paper_surface() {
        assert_eq!(TIMELINE_BULLET_BACKGROUND_DEFAULT, PAPER_BACKGROUND.value());
    }
}
