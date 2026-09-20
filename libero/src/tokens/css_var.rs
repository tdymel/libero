use std::fmt::Display;

use super::{ColorShade, Size};
use crate::css::CssDeclaration;

/// The per-instance twin of a themed var: `Theme` declares `--lsx-icon-size`, a prop
/// sets `--lsx-icon-size-override`. Always derived, so the two can't drift.
const OVERRIDE_SUFFIX: &str = "-override";

/// A CSS custom property's name, e.g. `--lsx-spacing-md`.
///
/// ```
/// # use libero::theme::CssVar;
/// const ACCENT: CssVar = CssVar::new("--app-accent");
/// assert_eq!(ACCENT.value(), "var(--app-accent)");
/// ```
#[derive(Clone, Debug)]
pub enum CssVar {
    Static(&'static str),
    Owned(String),
}

// By name only: `Variables` dedupes on it and equal CSS must hash alike.
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

    /// `var(--name, <fallback>)`. Pass another var's `value()`, not a hand-written `var(..)`.
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

    /// Accepts `--name` or `var(--name)`.
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

/// A per-[`Size`] family of vars sharing a prefix, e.g. `--lsx-spacing-md`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SizeCss {
    prefix: &'static str,
}

impl SizeCss {
    pub const SPACING: SizeCss = SizeCss::new("--lsx-spacing-");
    pub const BREAKPOINT: SizeCss = SizeCss::new("--lsx-breakpoint-");
    pub const RADIUS: SizeCss = SizeCss::new("--lsx-radius-");
    pub const SHADOW: SizeCss = SizeCss::new("--lsx-shadow-");
    pub const FONT_SIZE: SizeCss = SizeCss::new("--lsx-font-size-");

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

/// A per-[`ColorShade`] family of vars sharing a prefix, e.g. `--lsx-primary-7`.
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
    pub const NEUTRAL: ColorCss = ColorCss::new("--lsx-neutral-");
    pub const MUTED: ColorCss = ColorCss::new("--lsx-muted-");
    pub const PRIMARY_CONTRAST: ColorCss = ColorCss::new("--lsx-primary-contrast-");
    pub const SECONDARY_CONTRAST: ColorCss = ColorCss::new("--lsx-secondary-contrast-");
    pub const ERROR_CONTRAST: ColorCss = ColorCss::new("--lsx-error-contrast-");
    pub const WARNING_CONTRAST: ColorCss = ColorCss::new("--lsx-warning-contrast-");
    pub const INFO_CONTRAST: ColorCss = ColorCss::new("--lsx-info-contrast-");
    pub const SUCCESS_CONTRAST: ColorCss = ColorCss::new("--lsx-success-contrast-");
    pub const NEUTRAL_CONTRAST: ColorCss = ColorCss::new("--lsx-neutral-contrast-");
    pub const MUTED_CONTRAST: ColorCss = ColorCss::new("--lsx-muted-contrast-");

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

    /// A derived ramp's var by infix, `--lsx-primary-text-6` from `--lsx-primary-`.
    pub fn role_name(self, infix: &str, shade: ColorShade) -> String {
        format!("{}{infix}{}", self.prefix, shade.as_str())
    }

    pub fn role_value(self, infix: &str, shade: ColorShade) -> String {
        format!("var({})", self.role_name(infix, shade))
    }
}

/// A single named colour var, without shades.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedColorCss {
    css_var: CssVar,
}

impl NamedColorCss {
    /// Text colour; a dark theme swaps its end of the greyscale with `SURFACE`.
    pub const INK: NamedColorCss = NamedColorCss::new("--lsx-ink");
    /// The page text is set on.
    pub const SURFACE: NamedColorCss = NamedColorCss::new("--lsx-surface");
    /// Published by a known-contrast `background()`, read by focus rings inside it.
    pub const FOCUS_CONTRAST: NamedColorCss = NamedColorCss::new("--lsx-focus-contrast");
    /// Quieter text: a placeholder, hint or unit. `"text-dimmed"` in an `Sx`; icons use `muted.6`.
    pub const TEXT_DIMMED: NamedColorCss = NamedColorCss::new("--lsx-text-dimmed");

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

    pub fn var(&self) -> CssVar {
        self.css_var.clone()
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

    // Byte-identical, or the `:root` declaration and the override stop meeting.
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
