use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Size, SizeCss, Sizes};

pub const LIST_GAP: SizeCss = SizeCss::new("--lsx-list-gap-");
pub const LIST_INDENT: SizeCss = SizeCss::new("--lsx-list-indent-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListDefaults {
    pub size: Size,
    pub gaps: Sizes<u8>,
    pub indents: Sizes<u8>,
}

impl ListDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        gaps: Sizes::new(4, 8, 12, 16, 20, 24),
        indents: Sizes::new(8, 12, 16, 20, 24, 28),
    };

    fn size_sx(size: Size) -> Sx {
        sx().gap(LIST_GAP.value(size))
            .selector("& ul", sx().padding_left(LIST_INDENT.value(size)))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for ListDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = self.gaps.to_css_declarations(LIST_GAP, "px");
        declarations.extend(self.indents.to_css_declarations(LIST_INDENT, "px"));
        declarations
    }
}
