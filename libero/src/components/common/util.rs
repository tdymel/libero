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

/// The stand-in that draws a focus ring for an element the focus lands
/// *inside* of - a field's frame, a checkbox's box. Placed after the
/// focusable, so `:focus-visible ~ [data-ring]` reaches it: a sibling rule.
/// The obvious `:has(:focus-visible)` never matches natively, because stylo
/// rejects `:has()` ([[codebase/blitz-platform-gaps]]), and there is no
/// `:focus-visible-within`.
///
/// It covers its containing block - the nearest positioned ancestor, which
/// must be the element the ring belongs to - and is styled there with
/// [`ring_overlay_sx`]. Hidden from assistive tech, and never a tab stop.
pub(crate) fn ring_overlay() -> dioxus::prelude::Element {
    use dioxus::prelude::*;

    rsx! {
        span { "data-ring": true, "aria-hidden": "true" }
    }
}

/// The overlay's own box: over the whole padding box, with the owner's
/// corners, and never in the way of a click. An owner with a border moves it
/// out by the border's width, so the ring's offset is measured from the same
/// edge as an outline on the owner.
pub(crate) fn ring_overlay_sx() -> crate::sx::Sx {
    crate::sx::sx()
        .position("absolute")
        .inset("0")
        .border_radius("inherit")
        .pointer_events("none")
}

/// A user-supplied value as a CSS string literal, quotes included. Rust's
/// `{:?}` is not CSS escaping - it emits `\u{...}`, which no selector parses.
pub(crate) fn css_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            // A hex escape needs its terminating space, or the next character
            // is read as part of the escape.
            '\0'..='\u{1f}' | '\u{7f}' => out.push_str(&format!("\\{:x} ", ch as u32)),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::css_string;

    #[test]
    fn quotes_and_escapes_only_what_css_requires() {
        assert_eq!(css_string("node-1"), r#""node-1""#);
        assert_eq!(css_string(r#"a"b\c"#), r#""a\"b\\c""#);
        assert_eq!(css_string("über"), r#""über""#);
        assert_eq!(css_string("a\nb"), "\"a\\a b\"");
    }
}
