use crate::{
    common::ConstStr,
    theme::{ColorShade, Size},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CssVar {
    prefix: &'static str,
}

impl CssVar {
    pub const fn new(prefix: &'static str) -> Self {
        Self { prefix }
    }

    pub(crate) const fn to_const_str_name(self, suffix: &'static str) -> ConstStr {
        ConstStr::from_str(self.prefix).push_str(suffix)
    }

    pub(crate) const fn to_const_str_var(self, suffix: &'static str) -> ConstStr {
        ConstStr::new()
            .push_str("var(")
            .push_str(self.prefix)
            .push_str(suffix)
            .push_char(')')
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SizeCssVar {
    css_var: CssVar,
}

impl SizeCssVar {
    pub const SPACING: SizeCssVar = SizeCssVar::new("--lsx-spacing-");

    pub const fn new(prefix: &'static str) -> Self {
        Self {
            css_var: CssVar::new(prefix),
        }
    }

    pub const fn to_const_str(self, size: Size) -> ConstStr {
        self.css_var.to_const_str_var(size.as_str())
    }

    pub const fn to_const_str_name(self, size: Size) -> ConstStr {
        self.css_var.to_const_str_name(size.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorCssVar {
    css_var: CssVar,
}

impl ColorCssVar {
    pub const PRIMARY: ColorCssVar = ColorCssVar::new("--lsx-primary-");
    pub const SECONDARY: ColorCssVar = ColorCssVar::new("--lsx-secondary-");
    pub const PRIMARY_CONTRAST: ColorCssVar = ColorCssVar::new("--lsx-primary-contrast-");
    pub const SECONDARY_CONTRAST: ColorCssVar = ColorCssVar::new("--lsx-secondary-contrast-");

    pub const fn new(prefix: &'static str) -> Self {
        Self {
            css_var: CssVar::new(prefix),
        }
    }

    pub const fn to_const_str(self, shade: ColorShade) -> ConstStr {
        self.css_var.to_const_str_var(shade.as_str())
    }

    pub const fn to_const_str_name(self, shade: ColorShade) -> ConstStr {
        self.css_var.to_const_str_name(shade.as_str())
    }
}
