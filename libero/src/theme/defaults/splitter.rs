use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{Size, SizeCss, Sizes};

pub const SPLITTER_DIVIDER_SIZE: SizeCss = SizeCss::new("--lsx-splitter-divider-size-");
pub const SPLITTER_HIT_SIZE: SizeCss = SizeCss::new("--lsx-splitter-hit-size-");

#[derive(Clone, Debug, PartialEq)]
pub struct SplitterDefaults {
    /// Which size level `divider_size` uses when unset.
    pub size: Size,
    pub divider_size: Sizes<u8>,
    /// Invisible drag/keyboard hit-target thickness - stays fixed regardless
    /// of `divider_size`, doesn't grow on hover/drag.
    pub hit_size: Sizes<u8>,
    /// Percent floor applied to both panes.
    pub min_size: f64,
    /// Percent moved per arrow key press.
    pub step: f64,
    /// Percent moved per Shift+arrow key press.
    pub big_step: f64,
}

impl SplitterDefaults {
    pub const fn new(
        size: Size,
        divider_size: Sizes<u8>,
        hit_size: Sizes<u8>,
        min_size: f64,
        step: f64,
        big_step: f64,
    ) -> Self {
        Self {
            size,
            divider_size,
            hit_size,
            min_size,
            step,
            big_step,
        }
    }
}

impl ToCssDeclarations for SplitterDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = self.divider_size.to_css_declarations(SPLITTER_DIVIDER_SIZE, "px");
        declarations.extend(self.hit_size.to_css_declarations(SPLITTER_HIT_SIZE, "px"));
        declarations
    }
}
