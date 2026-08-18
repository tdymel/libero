/// Builds a non-global attribute (e.g. `src`/`href`) to push into a
/// `Vec<Attribute>` by hand, bypassing `extends = GlobalAttributes`.
pub(crate) fn attr<T>(
    name: &'static str,
    value: impl dioxus::core::IntoAttributeValue<T>,
) -> dioxus::prelude::Attribute {
    dioxus::prelude::Attribute::new(name, value, None, false)
}

/// Debug-only warning, a no-op in release builds - unlike `dioxus::warn!`,
/// just a plain string, no format-string semantics.
#[cfg(debug_assertions)]
pub(crate) fn warn(message: &str) {
    dioxus::prelude::warn!("{message}");
}

#[cfg(not(debug_assertions))]
pub(crate) fn warn(_message: &str) {}

/// Standard `:focus-visible` ring - contrasts against whichever ancestor
/// most recently set a background (via `--lsx-focus-contrast`, published by
/// `background()`; see `ThemeAwareValue::focus_contrast`), falling back to
/// the theme's primary color where that isn't known (e.g. a raw/unparseable
/// background, or no themed background above it at all).
pub(crate) fn focus_ring_sx() -> crate::sx::Sx {
    use crate::tokens::{ColorCss, ColorShade, NamedColorCss};

    crate::sx::sx()
        .outline(format!(
            "2px solid {}",
            NamedColorCss::FOCUS_CONTRAST.value_or(ColorCss::PRIMARY.value(ColorShade::S6))
        ))
        .outline_offset("2px")
}
