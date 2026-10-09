use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Side, Size, SizeCss, Sizes};

pub const TOOLTIP_FONT_SIZE: SizeCss = SizeCss::new("--lsx-tooltip-font-size-");

pub const TOOLTIP_BACKGROUND: CssVar = CssVar::new("--lsx-tooltip-background");
pub const TOOLTIP_COLOR: CssVar = CssVar::new("--lsx-tooltip-color");
pub const TOOLTIP_DURATION: CssVar = CssVar::new("--lsx-tooltip-duration");

/// Fade in only: the bubble mounts on open and unmounts on close.
pub(crate) const TOOLTIP_KEYFRAMES: &str = "@keyframes lsx-tooltip-in{from{opacity:0;}}";
pub(crate) const TOOLTIP_IN: &str = "lsx-tooltip-in";

/// Theme defaults for `Tooltip`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TooltipDefaults {
    pub side: Side,
    /// Distance between trigger and bubble, bridged so the pointer can cross.
    pub gap: Size,
    pub size: Size,
    /// Milliseconds before the bubble appears and disappears.
    pub open_delay: u32,
    pub close_delay: u32,
    /// Fade duration, in milliseconds.
    pub duration: u32,
    /// Pixels at a 16px root, written as `rem`, so the bubble grows with a raised text size.
    pub font_sizes: Sizes<u16>,
    pub background: &'static str,
    pub color: &'static str,
}

impl TooltipDefaults {
    pub const DEFAULT: Self = Self {
        side: Side::Top,
        gap: Size::Xs,
        // 14px, the label floor (todo 2707).
        size: Size::Lg,
        open_delay: 0,
        close_delay: 0,
        duration: 150,
        font_sizes: Sizes::new(10, 12, 13, 14, 16, 18),
        // Inverted: the `muted` step furthest from the page, lettered in the page (todo 396).
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
        let mut declarations: Vec<_> = Size::ALL
            .into_iter()
            .map(|size| {
                let rem = f32::from(self.font_sizes.get(size)) / 16.0;
                TOOLTIP_FONT_SIZE.declare(size, format!("{rem}rem"))
            })
            .collect();
        declarations.push(TOOLTIP_BACKGROUND.declare(self.background));
        declarations.push(TOOLTIP_COLOR.declare(self.color));
        declarations.push(TOOLTIP_DURATION.declare(format!("{}ms", self.duration)));
        declarations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Todo 2601: rem, the same size as the old px at a 16px root.
    #[test]
    fn the_font_sizes_are_rem() {
        let css: Vec<String> = TooltipDefaults::DEFAULT
            .to_css_declarations()
            .iter()
            .map(ToString::to_string)
            .collect();

        assert!(
            css.iter()
                .any(|d| d.contains("--lsx-tooltip-font-size-xs") && d.contains(":0.625rem")),
            "{css:?}"
        );
        assert!(
            css.iter()
                .any(|d| d.contains("--lsx-tooltip-font-size-xxl") && d.contains(":1.125rem")),
            "{css:?}"
        );
        assert!(
            !css.iter()
                .any(|d| d.contains("font-size") && d.contains("px")),
            "{css:?}"
        );
    }
}
