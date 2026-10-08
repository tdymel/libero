use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Size, SizeCss, Sizes};

pub const LIST_GAP: SizeCss = SizeCss::new("--lsx-list-gap-");
pub const LIST_INDENT: SizeCss = SizeCss::new("--lsx-list-indent-");

/// Theme defaults for `List`, set on [`Theme`](crate::theme::Theme).
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

    // Only a `List` directly under this list's items, so a raw `ul` and a deeper nest keep their own indent.
    const NESTED: &str = "& > li > ul[role='list'], & > li > [data-slot='body'] > ul[role='list']";

    fn size_sx(size: Size) -> Sx {
        sx().gap(LIST_GAP.value(size)).selector(
            Self::NESTED,
            sx().padding_inline_start(LIST_INDENT.value(size)),
        )
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
