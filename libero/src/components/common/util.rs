use std::fmt::Write as _;

use crate::{
    sx::{Sx, sx},
    theme::CssVar,
};

/// A non-global attribute to push into a `Vec<Attribute>` by hand.
pub(crate) fn attr<T>(
    name: &'static str,
    value: impl dioxus::core::IntoAttributeValue<T>,
) -> dioxus::prelude::Attribute {
    dioxus::prelude::Attribute::new(name, value, None, false)
}

/// A user-supplied value as a CSS string literal. Not `{:?}`: its `\u{...}` doesn't parse.
pub(crate) fn css_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            // The space ends the escape, or the next character joins it.
            '\0'..='\u{1f}' | '\u{7f}' => {
                let _ = write!(out, "\\{:x} ", ch as u32);
            }
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

/// `base` plus whatever a notch or a home indicator takes on `side`; 0 without
/// `viewport-fit=cover`.
pub(crate) fn safe_area_padding(base: &str, side: &str) -> String {
    format!("calc({base} + env(safe-area-inset-{side}, 0px))")
}

/// A docked `NavigationBar`'s height on `:root`, its bottom inset included; FAB and toasts rise above it.
pub(crate) const NAVIGATION_BAR_HEIGHT_VAR: CssVar = CssVar::new("--lsx-navigation-bar-height");

/// Set to `0` by a panel that pads that inline side's safe area, so a `Sidebar` inside skips it (5519).
pub(crate) const SAFE_AREA_INLINE_START_VAR: CssVar = CssVar::new("--lsx-safe-area-inline-start");
pub(crate) const SAFE_AREA_INLINE_END_VAR: CssVar = CssVar::new("--lsx-safe-area-inline-end");

/// `padding-inline-start` (or `-end`) plus the inset of the physical side it lands on, flipped
/// under RTL. `nested` scales the inset by that side's var above.
pub(crate) fn safe_inline_padding(base: &str, start: bool, nested: bool) -> Sx {
    let (property, var, ltr, rtl) = if start {
        (
            "padding-inline-start",
            SAFE_AREA_INLINE_START_VAR,
            "left",
            "right",
        )
    } else {
        (
            "padding-inline-end",
            SAFE_AREA_INLINE_END_VAR,
            "right",
            "left",
        )
    };
    let side = |physical: &str| {
        let inset = format!("env(safe-area-inset-{physical}, 0px)");
        let inset = if nested {
            format!("{inset} * {}", var.value_or("1"))
        } else {
            inset
        };
        sx().with(property, format!("calc({base} + {inset})"))
    };
    sx().and(side(ltr)).rtl(side(rtl))
}

#[cfg(test)]
mod tests {
    use super::{css_string, safe_area_padding, safe_inline_padding};

    #[test]
    fn safe_area_padding_adds_the_inset_to_the_base() {
        assert_eq!(
            safe_area_padding("8px", "bottom"),
            "calc(8px + env(safe-area-inset-bottom, 0px))"
        );
    }

    /// Todo 5523: the start side is the left, the right under RTL; nested scales the inset.
    #[test]
    fn inline_padding_flips_under_rtl() {
        let css = crate::css::Stylesheet::from(&safe_inline_padding("8px", true, true));
        let css = css.as_str();
        for side in ["left", "right"] {
            assert!(
                css.contains(&format!(
                    "padding-inline-start:calc(8px + env(safe-area-inset-{side}, 0px) * var(--lsx-safe-area-inline-start, 1))"
                )),
                "{css}"
            );
        }
    }

    #[test]
    fn quotes_and_escapes_only_what_css_requires() {
        assert_eq!(css_string("node-1"), r#""node-1""#);
        assert_eq!(css_string(r#"a"b\c"#), r#""a\"b\\c""#);
        assert_eq!(css_string("über"), r#""über""#);
        assert_eq!(css_string("a\nb"), "\"a\\a b\"");
    }
}
