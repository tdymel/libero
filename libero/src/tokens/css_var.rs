use std::fmt::Display;

use super::{ColorShade, Size};
use crate::css::CssDeclaration;

/// Marks the per-instance twin of a themed var: `Theme` declares
/// `--lsx-icon-size`, a caller's prop sets `--lsx-icon-size-override`.
/// Always derived, so the two can't drift apart.
const OVERRIDE_SUFFIX: &str = "-override";

#[derive(Clone, Debug)]
pub enum CssVar {
    Static(&'static str),
    Owned(String),
}

// By name only - the variants are a storage detail, but `Variables` dedupes
// on equality and identical CSS must hash to the same class name.
impl PartialEq for CssVar {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for CssVar {}

impl std::hash::Hash for CssVar {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name().hash(state);
    }
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

    /// `var(--name, <fallback>)`. The fallback is CSS text - pass another
    /// var's `value()`, don't hand-write `var(--lsx-...)`.
    pub fn value_or(&self, fallback: impl Display) -> String {
        format!("var({}, {fallback})", self.name())
    }

    /// This var's per-instance override twin (see [`OVERRIDE_SUFFIX`]).
    pub fn override_var(&self) -> Self {
        Self::Owned(format!("{}{OVERRIDE_SUFFIX}", self.name()))
    }

    /// `var(--name-override, var(--name))`: the caller's value, else theme's.
    pub fn overridable(&self) -> String {
        self.override_var().value_or(self.value())
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

    pub const fn new(prefix: &'static str) -> Self {
        Self { prefix }
    }

    pub fn value(self, size: Size) -> String {
        format!("var({}{})", self.prefix, size.as_str())
    }

    /// `var(--prefix-<size>, <fallback>)`.
    pub fn value_or(self, size: Size, fallback: impl Display) -> String {
        format!("var({}{}, {fallback})", self.prefix, size.as_str())
    }

    /// One override var for the whole scale, not one per size.
    pub fn override_var(self) -> CssVar {
        CssVar::Owned(format!(
            "{}{}",
            self.prefix.trim_end_matches('-'),
            OVERRIDE_SUFFIX
        ))
    }

    /// `var(--prefix-override, var(--prefix-<size>))`.
    pub fn overridable(self, size: Size) -> String {
        self.override_var().value_or(self.value(size))
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

    /// `var(--prefix-<shade>, <fallback>)`.
    pub fn value_or(self, shade: ColorShade, fallback: impl Display) -> String {
        format!("var({}{}, {fallback})", self.prefix, shade.as_str())
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
    /// Published by `background()` when a color's contrast is known, and read
    /// by focus rings, so a ring contrasts against the nearest ancestor
    /// background without either side knowing about the other.
    pub const FOCUS_CONTRAST: NamedColorCss = NamedColorCss::new("--lsx-focus-contrast");

    pub const fn new(name: &'static str) -> Self {
        Self {
            css_var: CssVar::new(name),
        }
    }

    pub fn value(&self) -> String {
        self.css_var.value()
    }

    /// `var(--name, <fallback>)`.
    pub fn value_or(&self, fallback: impl Display) -> String {
        self.css_var.value_or(fallback)
    }

    pub fn name(&self) -> &str {
        self.css_var.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_name_compares_equal_across_variants() {
        assert_eq!(CssVar::new("--lsx-x"), CssVar::Owned("--lsx-x".to_string()));
        assert_ne!(CssVar::new("--lsx-x"), CssVar::new("--lsx-y"));
    }

    #[test]
    fn value_and_fallback() {
        let var = CssVar::new("--lsx-icon-color");
        assert_eq!(var.value(), "var(--lsx-icon-color)");
        assert_eq!(var.value_or("inherit"), "var(--lsx-icon-color, inherit)");
        assert_eq!(var.value_or(0), "var(--lsx-icon-color, 0)");
    }

    // These names must stay byte-identical, or the theme's `:root`
    // declaration and the element's override stop meeting.
    #[test]
    fn override_names_match_the_literals_they_replaced() {
        for (base, expected) in [
            ("--lsx-aspect-ratio", "--lsx-aspect-ratio-override"),
            ("--lsx-center-display", "--lsx-center-display-override"),
            ("--lsx-container-size", "--lsx-container-size-override"),
            (
                "--lsx-container-gutters",
                "--lsx-container-gutters-override",
            ),
            ("--lsx-float-z-index", "--lsx-float-z-index-override"),
            ("--lsx-action-icon-size", "--lsx-action-icon-size-override"),
            (
                "--lsx-action-icon-radius",
                "--lsx-action-icon-radius-override",
            ),
        ] {
            assert_eq!(CssVar::new(base).override_var().name(), expected);
        }

        for (scale, expected) in [
            (SizeCss::new("--lsx-icon-size-"), "--lsx-icon-size-override"),
            (
                SizeCss::new("--lsx-header-height-"),
                "--lsx-header-height-override",
            ),
            (
                SizeCss::new("--lsx-drawer-size-"),
                "--lsx-drawer-size-override",
            ),
            (
                SizeCss::new("--lsx-dialog-size-"),
                "--lsx-dialog-size-override",
            ),
        ] {
            assert_eq!(scale.override_var().name(), expected);
        }
    }

    #[test]
    fn overridable_falls_back_to_the_themed_default() {
        assert_eq!(
            CssVar::new("--lsx-aspect-ratio").overridable(),
            "var(--lsx-aspect-ratio-override, var(--lsx-aspect-ratio))"
        );
        assert_eq!(
            SizeCss::new("--lsx-dialog-size-").overridable(Size::Md),
            "var(--lsx-dialog-size-override, var(--lsx-dialog-size-md))"
        );
    }
}
