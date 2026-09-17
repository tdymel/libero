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

// The picked level of each axis, resolved on the root so the `<li>`s and the
// bullets - which carry no `data-state` of their own - inherit it. The
// `Slider` inherited-var trick.
pub const TIMELINE_BULLET: CssVar = CssVar::new("--lsx-timeline-bullet");
pub const TIMELINE_SPACE: CssVar = CssVar::new("--lsx-timeline-space");
pub const TIMELINE_RADIUS: CssVar = CssVar::new("--lsx-timeline-radius");

// Resolved per item rather than per timeline, so one `::before` rule serves
// every line style and every active state instead of one rule per
// combination. The `<li>` sets the first from its own `TimelineLine`; the
// other two flip on the item's `data-state`.
pub const TIMELINE_LINE_STYLE: CssVar = CssVar::new("--lsx-timeline-line-style");
pub const TIMELINE_CONNECTOR: CssVar = CssVar::new("--lsx-timeline-connector");
pub const TIMELINE_MARKER: CssVar = CssVar::new("--lsx-timeline-marker");

/// `PAPER_BACKGROUND.value()`, spelled as a `const` because `Theme::DEFAULT`
/// is one and `CssVar::value` formats at runtime. `a_bullet_sits_on_the_paper
/// _surface` pins the two together, so the duplicated spelling cannot drift
/// from the var it names.
pub const TIMELINE_BULLET_BACKGROUND_DEFAULT: &str = "var(--lsx-paper-background)";

str_enum! {
    /// Which side of the rail an event's content sits on. Logical: `Start` is
    /// the right under `dir="rtl"`.
    #[state_prefix = "align"]
    pub enum TimelineAlign {
        /// The rail on the inline-start edge, content after it.
        #[default]
        Start = "start",
        /// The rail on the inline-end edge, content before it, text aligned
        /// to the end.
        End = "end",
        /// Content alternates either side of a centred rail, at every width.
        /// There is no size below which it falls back to one side: asking for
        /// an alternating timeline in a narrow box is a decision the caller
        /// has made, and the component does not overrule it. It also makes
        /// the list fill its parent, since a centred rail needs a width to be
        /// centred in.
        ///
        /// A variant rather than a second `alternate` prop: beside `align` it
        /// would make `align: "end", alternate: true` expressible and
        /// meaningless.
        Alternate = "alternate",
    }
}

/// The `data-state` token for the gap axis.
///
/// `const fn` because `States::with` needs a `&'static str`, and shared with
/// [`TimelineDefaults::gap_sx`] so the rule and the token cannot drift.
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimelineDefaults {
    pub align: TimelineAlign,
    /// `ColorValue`, not a `&'static str`. A raw `"primary.6"` is declared
    /// verbatim and is not a CSS colour, which silently deletes every
    /// shorthand that reads it - see the `to_css_declarations` note below.
    pub color: ColorValue,
    pub line_color: ColorValue,
    /// The surface the bullet is drawn on, and the glyph colour once an
    /// active bullet inverts. Defaults to `PAPER_BACKGROUND` rather than a
    /// literal `white`, so dark mode is a change to `PaperDefaults` and not to
    /// this component - see todo 45.
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
        // The pending ring and rail: 3:1 on the page and the dark `Paper`
        // (WCAG 1.4.11, todo 599).
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

    /// Hand-rolled rather than a `per_gap` helper beside `per_size` and
    /// `per_radius`: `Timeline` is the only component with a third
    /// independent size axis, and adding one would mean a `Size` method and
    /// an `Sx` method in two files another unit owns, for a single caller.
    /// Promote it if a second component ever needs one - this is exactly what
    /// `per_size` does underneath.
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
        // `.value()`, never the field itself. Declared raw, `primary.6` reaches
        // `:root` as a bare token: a custom property accepts it, so the var is
        // *set* and no `var()` fallback can fire, and then
        // `border: 2px solid var(--lsx-timeline-marker)` is invalid at
        // computed-value time and the whole declaration is dropped - no bullet
        // ring and no rail at all.
        declarations.push(TIMELINE_COLOR.declare(self.color.value()));
        declarations.push(TIMELINE_LINE_COLOR.declare(self.line_color.value()));
        declarations.push(TIMELINE_LINE_WIDTH.declare(format!("{}px", self.line_width)));
        declarations.push(TIMELINE_BULLET_BACKGROUND.declare(self.bullet_background));
        // Every item sets its own, so this is never the value that renders -
        // but an *unset* custom property inside the `border-left` shorthand
        // makes the whole declaration invalid at computed-value time, and the
        // connector vanishes rather than falling back to solid. This is what
        // makes the `::before` rule robust for whoever renders an item next.
        declarations.push(TIMELINE_LINE_STYLE.declare("solid"));
        declarations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::PAPER_BACKGROUND;

    /// The bullet is a ring filled with whatever surface it sits on, so its
    /// default has to *be* `PAPER_BACKGROUND` rather than a second spelling of
    /// white. Dark mode is then a change to `PaperDefaults` and not to this
    /// component.
    #[test]
    fn a_bullet_sits_on_the_paper_surface() {
        assert_eq!(TIMELINE_BULLET_BACKGROUND_DEFAULT, PAPER_BACKGROUND.value());
    }
}
