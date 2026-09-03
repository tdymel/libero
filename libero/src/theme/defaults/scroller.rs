use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const SCROLLER_CONTROL_SIZE: SizeCss = SizeCss::new("--lsx-scroller-control-size-");
/// The picked step's width, resolved on the root from the `size-*` token, so
/// the two controls - which carry no size token of their own - inherit it.
pub const SCROLLER_CONTROL: CssVar = CssVar::new("--lsx-scroller-control");
/// What the gradient under a control fades from. A per-instance
/// `fade_color` writes its `-override` twin.
pub const SCROLLER_FADE: CssVar = CssVar::new("--lsx-scroller-fade");

/// `PAPER_BACKGROUND.value()`, spelled as a `const` because `Theme::DEFAULT`
/// is one - the `TIMELINE_BULLET_BACKGROUND_DEFAULT` precedent, pinned the
/// same way by `the_fade_is_the_paper_surface`.
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
        /// Neither renders. Pair with `on_edge_change` for controls of your
        /// own.
        Never = "never",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollerDefaults {
    pub controls: ScrollerControls,
    /// Pixels one control press scrolls.
    pub scroll_amount: u32,
    pub control_size: Size,
    /// The width of each control strip, in px.
    pub control_sizes: Sizes<u16>,
    /// A CSS colour. Defaults to the surface colour, so dark mode is a change
    /// to `PaperDefaults` and not to this component.
    pub fade_color: &'static str,
    /// Mouse drag-to-pan. Touch and trackpad scroll natively either way.
    pub draggable: bool,
    /// English literals, the `DateDefaults` precedent: no i18n mechanism yet
    /// (todo 28). "Backward"/"forward" rather than "left"/"right", so the
    /// names do not lie under a right-to-left page.
    pub scroll_start_label: &'static str,
    pub scroll_end_label: &'static str,
}

impl ScrollerDefaults {
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

    /// The fade has to *be* the surface colour, not a second spelling of
    /// white, or the control strip shows as a band on anything but white.
    #[test]
    fn the_fade_is_the_paper_surface() {
        assert_eq!(SCROLLER_FADE_DEFAULT, PAPER_BACKGROUND.value());
    }
}
