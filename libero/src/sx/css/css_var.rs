use crate::{
    common::ConstStr,
    theme::{ColorShade, Size},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CssVar {
    pub name: &'static str,
}

impl CssVar {
    pub const SPACING: CssVar = CssVar::new("--lsx-spacing-");
    pub const PRIMARY: CssVar = CssVar::new("--lsx-primary-");
    pub const SECONDARY: CssVar = CssVar::new("--lsx-secondary-");
    pub const PRIMARY_CONTRAST: CssVar = CssVar::new("--lsx-primary-contrast");
    pub const SECONDARY_CONTRAST: CssVar = CssVar::new("--lsx-secondary-contrast");

    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }

    pub const fn push_name_with_size(self, mut css: ConstStr, size: Size) -> ConstStr {
        css = css.push_str(self.name);
        css = css.push_str(size.as_str());
        css
    }

    pub const fn push_value_with_size(self, mut css: ConstStr, size: Size) -> ConstStr {
        css = css.push_str("var(");
        css = self.push_name_with_size(css, size);
        css = css.push_char(')');
        css
    }

    pub const fn push_name_with_shade(self, mut css: ConstStr, shade: ColorShade) -> ConstStr {
        css = css.push_str(self.name);
        css = css.push_str(shade.as_hundreds_str());
        css
    }

    pub const fn push_value_with_shade(self, mut css: ConstStr, shade: ColorShade) -> ConstStr {
        css = css.push_str("var(");
        css = self.push_name_with_shade(css, shade);
        css = css.push_char(')');
        css
    }

    pub const fn push_name(self, mut css: ConstStr) -> ConstStr {
        css = css.push_str(self.name);
        css
    }

    pub const fn push_value(self, mut css: ConstStr) -> ConstStr {
        css = css.push_str("var(");
        css = self.push_name(css);
        css = css.push_char(')');
        css
    }
}
