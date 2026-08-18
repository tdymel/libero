use super::{ColorShade, Size};
use crate::css::CssDeclaration;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CssVar {
    Static(&'static str),
    Owned(String),
}

impl CssVar {
    pub const fn new(name: &'static str) -> Self {
        Self::Static(name)
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Static(name) => name,
            Self::Owned(name) => name.as_str(),
        }
    }

    pub fn value(&self) -> String {
        format!("var({})", self.name())
    }

    pub fn parse_name(value: &str) -> Option<&str> {
        if value.starts_with("--") {
            Some(value)
        } else {
            None
        }
    }

    pub fn parse_value(value: &str) -> Option<&str> {
        let inner = value.strip_prefix("var(")?.strip_suffix(')')?;
        Self::parse_name(inner)
    }

    pub fn parse(value: &str) -> Option<Self> {
        if let Some(name) = Self::parse_name(value) {
            return Some(Self::Owned(name.to_string()));
        }

        if let Some(name) = Self::parse_value(value) {
            return Some(Self::Owned(name.to_string()));
        }

        None
    }

    pub(crate) fn declare(&self, value: impl Into<String>) -> CssDeclaration {
        CssDeclaration::new(self.name(), value.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SizeCss {
    prefix: &'static str,
}

impl SizeCss {
    pub const SPACING: SizeCss = SizeCss::new("--lsx-spacing-");
    pub const BREAKPOINT: SizeCss = SizeCss::new("--lsx-breakpoint-");
    pub const RADIUS: SizeCss = SizeCss::new("--lsx-radius-");
    pub const DATA_LIST_GAP: SizeCss = SizeCss::new("--lsx-data-list-gap-");
    pub const DIALOG_SIZE: SizeCss = SizeCss::new("--lsx-dialog-size-");
    pub const DRAWER_SIZE: SizeCss = SizeCss::new("--lsx-drawer-size-");
    pub const HEADER_HEIGHT: SizeCss = SizeCss::new("--lsx-header-height-");
    pub const ICON_SIZE: SizeCss = SizeCss::new("--lsx-icon-size-");
    pub const KBD_FONT_SIZE: SizeCss = SizeCss::new("--lsx-kbd-font-size-");
    pub const LIST_GAP: SizeCss = SizeCss::new("--lsx-list-gap-");
    pub const LIST_INDENT: SizeCss = SizeCss::new("--lsx-list-indent-");

    pub const fn new(prefix: &'static str) -> Self {
        Self { prefix }
    }

    pub fn value(self, size: Size) -> String {
        format!("var({}{})", self.prefix, size.as_str())
    }

    pub fn name(self, size: Size) -> String {
        format!("{}{}", self.prefix, size.as_str())
    }

    pub(crate) fn declare(self, size: Size, value: impl Into<String>) -> CssDeclaration {
        CssDeclaration::new(self.name(size), value.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorCss {
    prefix: &'static str,
}

impl ColorCss {
    pub const PRIMARY: ColorCss = ColorCss::new("--lsx-primary-");
    pub const SECONDARY: ColorCss = ColorCss::new("--lsx-secondary-");
    pub const ERROR: ColorCss = ColorCss::new("--lsx-error-");
    pub const WARNING: ColorCss = ColorCss::new("--lsx-warning-");
    pub const INFO: ColorCss = ColorCss::new("--lsx-info-");
    pub const SUCCESS: ColorCss = ColorCss::new("--lsx-success-");
    pub const GREY: ColorCss = ColorCss::new("--lsx-grey-");
    pub const PRIMARY_CONTRAST: ColorCss = ColorCss::new("--lsx-primary-contrast-");
    pub const SECONDARY_CONTRAST: ColorCss = ColorCss::new("--lsx-secondary-contrast-");
    pub const ERROR_CONTRAST: ColorCss = ColorCss::new("--lsx-error-contrast-");
    pub const WARNING_CONTRAST: ColorCss = ColorCss::new("--lsx-warning-contrast-");
    pub const INFO_CONTRAST: ColorCss = ColorCss::new("--lsx-info-contrast-");
    pub const SUCCESS_CONTRAST: ColorCss = ColorCss::new("--lsx-success-contrast-");
    pub const GREY_CONTRAST: ColorCss = ColorCss::new("--lsx-grey-contrast-");

    pub const fn new(prefix: &'static str) -> Self {
        Self { prefix }
    }

    pub fn value(self, shade: ColorShade) -> String {
        format!("var({}{})", self.prefix, shade.as_str())
    }

    pub fn name(self, shade: ColorShade) -> String {
        format!("{}{}", self.prefix, shade.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedColorCss {
    css_var: CssVar,
}

impl NamedColorCss {
    pub const BLACK: NamedColorCss = NamedColorCss::new("--lsx-black");
    pub const WHITE: NamedColorCss = NamedColorCss::new("--lsx-white");
    /// Published by `background()` wherever a color's contrast is known (see
    /// `ThemeAwareValue::focus_contrast`) - focus rings read it (with a
    /// fallback) so they contrast against whichever ancestor most recently
    /// set a background, without either side needing to know about the
    /// other.
    pub const FOCUS_CONTRAST: NamedColorCss = NamedColorCss::new("--lsx-focus-contrast");

    pub const fn new(name: &'static str) -> Self {
        Self {
            css_var: CssVar::new(name),
        }
    }

    pub fn value(&self) -> String {
        self.css_var.value()
    }

    pub fn name(&self) -> &str {
        self.css_var.name()
    }
}
