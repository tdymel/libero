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
            '\0'..='\u{1f}' | '\u{7f}' => out.push_str(&format!("\\{:x} ", ch as u32)),
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

#[cfg(test)]
mod tests {
    use super::{css_string, safe_area_padding};

    #[test]
    fn safe_area_padding_adds_the_inset_to_the_base() {
        assert_eq!(
            safe_area_padding("8px", "bottom"),
            "calc(8px + env(safe-area-inset-bottom, 0px))"
        );
    }

    #[test]
    fn quotes_and_escapes_only_what_css_requires() {
        assert_eq!(css_string("node-1"), r#""node-1""#);
        assert_eq!(css_string(r#"a"b\c"#), r#""a\"b\\c""#);
        assert_eq!(css_string("über"), r#""über""#);
        assert_eq!(css_string("a\nb"), "\"a\\a b\"");
    }
}
