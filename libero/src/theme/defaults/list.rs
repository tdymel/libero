use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Size, SizeCss, Sizes};

pub const LIST_GAP: SizeCss = SizeCss::new("--lsx-list-gap-");
pub const LIST_INDENT: SizeCss = SizeCss::new("--lsx-list-indent-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListDefaults {
    pub size: Size,
    pub gap: Sizes<u8>,
    pub indent: Sizes<u8>,
}

impl ListDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        gap: Sizes::new(4, 8, 12, 16, 20, 24),
        indent: Sizes::new(8, 12, 16, 20, 24, 28),
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
        let mut declarations = self.gap.to_css_declarations(LIST_GAP, "px");
        declarations.extend(self.indent.to_css_declarations(LIST_INDENT, "px"));
        declarations
    }
}
