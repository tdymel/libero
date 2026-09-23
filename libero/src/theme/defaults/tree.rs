use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{Color, ColorShade, ColorValue, CssVar, Size};

pub const TREE_GUIDE_COLOR: CssVar = CssVar::new("--lsx-tree-guide-color");
pub const TREE_GUIDE_ACTIVE_COLOR: CssVar = CssVar::new("--lsx-tree-guide-active-color");
pub const TREE_GUIDE_WIDTH: CssVar = CssVar::new("--lsx-tree-guide-width");
pub const TREE_GUIDE_ACTIVE_WIDTH: CssVar = CssVar::new("--lsx-tree-guide-active-width");

/// Theme defaults for `Tree`, set on [`Theme`](crate::theme::Theme).
/// It renders through `List`, so `size` picks one of `List`'s levels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeDefaults {
    pub size: Size,
    /// Draws the indent guides by default.
    pub guides: bool,
    /// `ColorValue`: a raw `"muted.3"` is not CSS and drops the `border` reading it.
    pub guide_color: ColorValue,
    /// The `current` row's segment of its guide.
    pub guide_active_color: ColorValue,
    pub guide_width: u8,
    pub guide_active_width: u8,
}

impl TreeDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        guides: false,
        guide_color: ColorValue::Shade(Color::Muted, ColorShade::S3),
        guide_active_color: ColorValue::Shade(Color::Primary, ColorShade::S6),
        guide_width: 1,
        guide_active_width: 2,
    };
}

impl ToCssDeclarations for TreeDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            TREE_GUIDE_COLOR.declare(self.guide_color.value()),
            TREE_GUIDE_ACTIVE_COLOR.declare(self.guide_active_color.value()),
            TREE_GUIDE_WIDTH.declare(format!("{}px", self.guide_width)),
            TREE_GUIDE_ACTIVE_WIDTH.declare(format!("{}px", self.guide_active_width)),
        ]
    }
}
