use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const SCROLLER_CONTROL_SIZE: SizeCss = SizeCss::new("--lsx-scroller-control-size-");
/// Resolved on the root; the two controls inherit it.
pub const SCROLLER_CONTROL: CssVar = CssVar::new("--lsx-scroller-control");
/// What the gradient under a control fades from; a `fade_color` prop writes the `-override` twin.
pub const SCROLLER_FADE: CssVar = CssVar::new("--lsx-scroller-fade");

/// `PAPER_BACKGROUND.value()` as a `const`, for `Theme::DEFAULT`.
pub const SCROLLER_FADE_DEFAULT: &str = "var(--lsx-paper-background)";

str_enum! {
    /// When the two step controls show.
    #[state_prefix = "controls"]
    pub enum ScrollerControls {
        /// Each control shows while there is content that way.
        #[default]
        Auto = "auto",
        /// Both always show, dimmed at their own end.
        Always = "always",
        /// Neither renders. Pair with `onedgechange` for controls of your own.
        Never = "never",
    }
}

/// Theme defaults for `Scroller`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollerDefaults {
    pub controls: ScrollerControls,
    /// Pixels one control press scrolls.
    pub scroll_amount: u32,
    pub control_size: Size,
    /// The width of each control strip, in px.
    pub control_sizes: Sizes<u16>,
    /// A CSS colour; the surface colour by default.
    pub fade_color: &'static str,
    /// Mouse drag-to-pan. Touch and trackpad scroll natively either way.
    pub draggable: bool,
}

impl ScrollerDefaults {
    pub const DEFAULT: Self = Self {
        controls: ScrollerControls::Auto,
        scroll_amount: 200,
        control_size: Size::Md,
        control_sizes: Sizes::new(24, 32, 40, 48, 56, 64),
        fade_color: SCROLLER_FADE_DEFAULT,
        draggable: false,
    };

    fn size_sx(size: Size) -> Sx {
        sx().var(SCROLLER_CONTROL, SCROLLER_CONTROL_SIZE.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for ScrollerDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = self
            .control_sizes
            .to_css_declarations(SCROLLER_CONTROL_SIZE, "px");
        declarations.push(SCROLLER_FADE.declare(self.fade_color));
        declarations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::PAPER_BACKGROUND;

    /// A second spelling of white would show the strip as a band on dark surfaces.
    #[test]
    fn the_fade_is_the_paper_surface() {
        assert_eq!(SCROLLER_FADE_DEFAULT, PAPER_BACKGROUND.value());
    }
}
