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

    pub(crate) const fn push_name(self, mut css: ConstStr, suffix: &'static str) -> ConstStr {
        css = css.push_str(self.prefix);
        css = css.push_str(suffix);
        css
    }

    pub(crate) const fn push_var(self, mut css: ConstStr, suffix: &'static str) -> ConstStr {
        css = css.push_str("var(");
        css = self.push_name(css, suffix);
        css = css.push_char(')');
        css
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

    pub const fn push_var(self, css: ConstStr, size: Size) -> ConstStr {
        self.css_var.push_var(css, size.as_str())
    }

    pub const fn push_name(self, css: ConstStr, size: Size) -> ConstStr {
        self.css_var.push_name(css, size.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorCssVar {
    css_var: CssVar,
}

impl ColorCssVar {
    pub const PRIMARY: ColorCssVar = ColorCssVar::new("--lsx-primary-");
    pub const SECONDARY: ColorCssVar = ColorCssVar::new("--lsx-secondary-");
    pub const PRIMARY_CONTRAST: ColorCssVar = ColorCssVar::new("--lsx-primary-contrast");
    pub const SECONDARY_CONTRAST: ColorCssVar = ColorCssVar::new("--lsx-secondary-contrast");

    pub const fn new(prefix: &'static str) -> Self {
        Self {
            css_var: CssVar::new(prefix),
        }
    }

    pub const fn push_var(self, css: ConstStr, shade: ColorShade) -> ConstStr {
        self.css_var.push_var(css, shade.as_hundreds_str())
    }

    pub const fn push_css_var(self, css: ConstStr) -> ConstStr {
        self.css_var.push_var(css, "")
    }

    pub const fn push_name_with_shade(self, css: ConstStr, shade: ColorShade) -> ConstStr {
        self.css_var.push_name(css, shade.as_hundreds_str())
    }

    pub const fn push_name(self, css: ConstStr) -> ConstStr {
        self.css_var.push_name(css, "")
    }
}
