use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Side, Size, SizeCss, Sizes};

pub const TOOLTIP_FONT_SIZE: SizeCss = SizeCss::new("--lsx-tooltip-font-size-");

pub const TOOLTIP_BACKGROUND: CssVar = CssVar::new("--lsx-tooltip-background");
pub const TOOLTIP_COLOR: CssVar = CssVar::new("--lsx-tooltip-color");
pub const TOOLTIP_DURATION: CssVar = CssVar::new("--lsx-tooltip-duration");

/// The bubble's fade in. It mounts when it opens, so a keyframe, not a
/// transition, and it unmounts when it closes, so there is no fade out.
pub(crate) const TOOLTIP_KEYFRAMES: &str = "@keyframes lsx-tooltip-in{from{opacity:0;}}";
pub(crate) const TOOLTIP_IN: &str = "lsx-tooltip-in";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TooltipDefaults {
    pub side: Side,
    /// Distance between trigger and bubble, bridged so the pointer can cross.
    pub gap: Size,
    pub size: Size,
    /// Milliseconds before the bubble appears / disappears.
    pub open_delay: u32,
    pub close_delay: u32,
    /// Fade duration, in milliseconds.
    pub duration: u32,
    pub font_sizes: Sizes<u16>,
    pub background: &'static str,
    pub color: &'static str,
}

impl TooltipDefaults {
    pub const DEFAULT: Self = Self {
        side: Side::Top,
        gap: Size::Xs,
        size: Size::Sm,
        open_delay: 0,
        close_delay: 0,
        duration: 150,
        font_sizes: Sizes::new(10, 12, 13, 14, 16, 18),
        // Drawn against the page, not on it: the `muted` step furthest from
        // the page, lettered in the page itself (todo 396).
        background: "var(--lsx-muted-9)",
        color: "var(--lsx-surface)",
    };

    fn size_sx(size: Size) -> Sx {
        sx().font_size(TOOLTIP_FONT_SIZE.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().background(TOOLTIP_BACKGROUND.value())
            .color(TOOLTIP_COLOR.value())
            .border_radius(SizeCss::RADIUS.value(Size::Sm))
            .padding(format!(
                "{} {}",
                SizeCss::SPACING.value(Size::Xs),
                SizeCss::SPACING.value(Size::Sm)
            ))
            .per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for TooltipDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = self.font_sizes.to_css_declarations(TOOLTIP_FONT_SIZE, "px");
        declarations.push(TOOLTIP_BACKGROUND.declare(self.background));
        declarations.push(TOOLTIP_COLOR.declare(self.color));
        declarations.push(TOOLTIP_DURATION.declare(format!("{}ms", self.duration)));
        declarations
    }
}
