use super::{ColorShade, Size};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CssVar {
    Static(&'static str),
    Owned(String),
}

impl CssVar {
    pub const fn new(name: &'static str) -> Self {
        Self::Static(name)
    }

    pub fn from_name(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        if Self::parse_name(name.as_str()).is_some() {
            Some(Self::Owned(name))
        } else {
            None
        }
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

    pub fn matches_name(&self, value: &str) -> bool {
        self.name() == value
    }

    pub fn matches_value(&self, value: &str) -> bool {
        value == self.value()
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

    pub const fn new(prefix: &'static str) -> Self {
        Self { prefix }
    }

    pub fn value(self, size: Size) -> String {
        format!("var({}{})", self.prefix, size.as_str())
    }

    pub fn name(self, size: Size) -> String {
        format!("{}{}", self.prefix, size.as_str())
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
