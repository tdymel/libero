use std::borrow::Cow;

/// `css` safe as a `<style>`'s text: `</` becomes `<\/`, the same in a CSS string, so
/// no `</style>` in it ends the element when the page is rendered as HTML (todo 2621).
pub(crate) fn style_text(css: &str) -> Cow<'_, str> {
    match css.contains("</") {
        true => Cow::Owned(css.replace("</", "<\\/")),
        false => Cow::Borrowed(css),
    }
}

/// Whether `value` stays inside its declaration: no `{`, `}` or `/*` outside a string,
/// no `;` outside a string or parentheses, and every string and parenthesis closed.
pub(crate) fn stays_in_declaration(value: &str) -> bool {
    let mut quote = None;
    let mut depth = 0u32;
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        // An escape takes the next character, which must exist: a trailing `\` escapes the `;`.
        if c == '\\' {
            if chars.next().is_none() {
                return false;
            }
            continue;
        }
        match (quote, c) {
            (Some(_), '\n' | '\r' | '\x0c') => return false,
            (Some(open), c) if c == open => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '(') => depth += 1,
            (None, ')') if depth == 0 => return false,
            (None, ')') => depth -= 1,
            (None, '{' | '}') => return false,
            (None, ';') if depth == 0 => return false,
            (None, '/') if chars.peek() == Some(&'*') => return false,
            (None, _) => {}
        }
    }
    quote.is_none() && depth == 0
}

/// Whether `name` is a plain property name, a custom one included.
pub(crate) fn is_property_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_closing_tag_cannot_end_the_style_element() {
        let css = ".a{content:\"</style><script>x</script>\"}";
        assert_eq!(
            style_text(css),
            ".a{content:\"<\\/style><script>x<\\/script>\"}"
        );
        assert!(matches!(style_text(".a{color:red}"), Cow::Borrowed(_)));
        // A media range keeps its `<`.
        assert_eq!(
            style_text("@media (width < 600px){}"),
            "@media (width < 600px){}"
        );
    }

    #[test]
    fn a_value_that_leaves_its_declaration_is_refused() {
        for attack in [
            "red}body{display:none",
            "red;background:url(https://evil.example/x)",
            "red}</style><script>alert(1)</script>",
            "red /* swallow the rules after",
            "red\\",
            "\"open",
            "'line\nbreak'",
            "calc(1px",
            "1px)",
            "{",
        ] {
            assert!(!stays_in_declaration(attack), "{attack}");
        }
    }

    #[test]
    fn an_ordinary_value_is_kept() {
        for value in [
            "red",
            "#ff0000",
            "1px solid var(--lsx-color-border, #ccc)",
            "calc(100% - (2 * var(--gap)))",
            "\"Inter\", 'Segoe UI', sans-serif",
            "\"a;b{c}d/*e\"",
            "url(data:image/png;base64,iVBORw0KGgo=)",
            "url(\"data:image/svg+xml;utf8,<svg/>\")",
            "\"\\\"\"",
            "",
        ] {
            assert!(stays_in_declaration(value), "{value}");
        }
    }

    #[test]
    fn a_property_name_is_plain() {
        assert!(is_property_name("--lsx-gap"));
        assert!(is_property_name("background-color"));
        assert!(!is_property_name("color:red;x"));
        assert!(!is_property_name(""));
    }
}
