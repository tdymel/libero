/// A non-global attribute to push into a `Vec<Attribute>` by hand.
pub(crate) fn attr<T>(
    name: &'static str,
    value: impl dioxus::core::IntoAttributeValue<T>,
) -> dioxus::prelude::Attribute {
    dioxus::prelude::Attribute::new(name, value, None, false)
}

/// Standard `:focus-visible` ring, contrasting against the nearest ancestor
/// background via `--lsx-focus-contrast` (published by `background()`).
/// Falls back to primary when no ancestor published one.
pub(crate) fn focus_ring_sx() -> crate::sx::Sx {
    use crate::tokens::{ColorCss, ColorShade, NamedColorCss};

    crate::sx::sx()
        .outline(format!(
            "2px solid {}",
            NamedColorCss::FOCUS_CONTRAST.value_or(ColorCss::PRIMARY.value(ColorShade::S6))
        ))
        .outline_offset("2px")
}
