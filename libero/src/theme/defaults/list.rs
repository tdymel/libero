use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Size, SizeCss, Sizes};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListDefaults {
    pub size: Size,
    pub gap: Sizes<u8>,
    pub indent: Sizes<u8>,
}

impl ListDefaults {
    fn size_sx(size: Size) -> Sx {
        sx().gap(SizeCss::LIST_GAP.value(size))
            .selector("& ul", sx().padding_left(SizeCss::LIST_INDENT.value(size)))
    }

    pub fn theme_vars() -> Sx {
        Size::ALL.into_iter().fold(sx(), |base, size| {
            base.when(size.state_name(), Self::size_sx(size))
        })
    }
}

impl ToCssDeclarations for ListDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = self.gap.to_css_declarations(SizeCss::LIST_GAP, "px");
        declarations.extend(self.indent.to_css_declarations(SizeCss::LIST_INDENT, "px"));
        declarations
    }
}
