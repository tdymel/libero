use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{Size, SizeCss, Sizes};

pub const SPLITTER_DIVIDER_SIZE: SizeCss = SizeCss::new("--lsx-splitter-divider-size-");
pub const SPLITTER_HIT_SIZE: SizeCss = SizeCss::new("--lsx-splitter-hit-size-");

/// Theme defaults for `Splitter`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitterDefaults {
    /// Which size level `divider_size` uses when unset.
    pub size: Size,
    pub divider_sizes: Sizes<u8>,
    /// Invisible hit-target thickness; 24 meets WCAG 2.5.8 on its own.
    pub hit_sizes: Sizes<u8>,
    /// Percent floor applied to both panes.
    pub min_size: f64,
    /// Percent moved per arrow key press.
    pub step: f64,
    /// Percent moved per Shift+arrow key press.
    pub big_step: f64,
}

impl SplitterDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Sm,
        divider_sizes: Sizes::new(1, 1, 2, 3, 4, 6),
        hit_sizes: Sizes::new(24, 24, 24, 24, 24, 24),
        min_size: 10.0,
        step: 1.0,
        big_step: 10.0,
    };
}

impl ToCssDeclarations for SplitterDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = self
            .divider_sizes
            .to_css_declarations(SPLITTER_DIVIDER_SIZE, "px");
        declarations.extend(self.hit_sizes.to_css_declarations(SPLITTER_HIT_SIZE, "px"));
        declarations
    }
}
