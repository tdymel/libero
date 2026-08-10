use crate::theme::{ColorShade, Size};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CssVar {
    prefix: &'static str,
}

impl CssVar {
    pub const fn new(prefix: &'static str) -> Self {
        Self { prefix }
    }

    pub(crate) fn to_string_name(self, suffix: &'static str) -> String {
        format!("{}{}", self.prefix, suffix)
    }

    pub(crate) fn to_string_var(self, suffix: &'static str) -> String {
        format!("var({}{})", self.prefix, suffix)
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

    pub fn to_string(self, size: Size) -> String {
        self.css_var.to_string_var(size.as_str())
    }

    pub fn to_string_name(self, size: Size) -> String {
        self.css_var.to_string_name(size.as_str())
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

    pub fn to_string(self, shade: ColorShade) -> String {
        self.css_var.to_string_var(shade.as_str())
    }

    pub fn to_string_name(self, shade: ColorShade) -> String {
        self.css_var.to_string_name(shade.as_str())
    }
}
