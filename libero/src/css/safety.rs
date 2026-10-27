use std::borrow::Cow;

use crate::utils::warn;

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
/// An unquoted `url(` must not turn into a bad-url, which ends at its first `)` (todo 4465).
pub(crate) fn stays_in_declaration(value: &str) -> bool {
    let mut quote = None;
    let mut depth = 0u32;
    let mut url = false;
    // Start of the identifier before the current character, and whether it holds an escape.
    let mut ident: Option<(usize, bool)> = None;
    let mut chars = value.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if url {
            match c {
                ')' => {
                    url = false;
                    depth -= 1;
                }
                '\\' if matches!(chars.next(), None | Some((_, '\n' | '\r' | '\x0c'))) => {
                    return false;
                }
                '\\' => {}
                '(' | '"' | '\'' | '{' | '}' => return false,
                c if is_css_whitespace(c) => {
                    while chars.next_if(|&(_, c)| is_css_whitespace(c)).is_some() {}
                    if !matches!(chars.peek(), None | Some((_, ')'))) {
                        return false;
                    }
                }
                c if c.is_control() => return false,
                _ => {}
            }
            continue;
        }
        // An escape takes the next character, which must exist: a trailing `\` escapes the `;`.
        if c == '\\' {
            if chars.next().is_none() {
                return false;
            }
            if quote.is_none() {
                ident = Some((ident.map_or(i, |(start, _)| start), true));
            }
            continue;
        }
        if quote.is_none() && (c.is_alphanumeric() || matches!(c, '-' | '_') || !c.is_ascii()) {
            ident.get_or_insert((i, false));
            continue;
        }
        let name = ident.take();
        match (quote, c) {
            (Some(_), '\n' | '\r' | '\x0c') => return false,
            (Some(open), c) if c == open => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '(') => {
                depth += 1;
                // An escaped function name could spell `url` unseen, so it is refused.
                if let Some((start, escaped)) = name {
                    if escaped {
                        return false;
                    }
                    if value[start..i].eq_ignore_ascii_case("url") {
                        while chars.next_if(|&(_, c)| is_css_whitespace(c)).is_some() {}
                        url = !matches!(chars.peek(), Some((_, '"' | '\'')));
                    }
                }
            }
            (None, ')') if depth == 0 => return false,
            (None, ')') => depth -= 1,
            (None, '{' | '}') => return false,
            (None, ';') if depth == 0 => return false,
            (None, '/') if matches!(chars.peek(), Some((_, '*'))) => return false,
            (None, _) => {}
        }
    }
    quote.is_none() && depth == 0
}

fn is_css_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0c')
}

/// `value` when it stays in its declaration, else `None` and a debug warning: a free
/// string a component writes into an inline style is dropped, not escaped (todo 3705).
pub(crate) fn declaration_value<'a>(property: &str, value: &'a str) -> Option<&'a str> {
    if stays_in_declaration(value) {
        return Some(value);
    }
    warn(&format!(
        "`{property}: {value}` dropped: it would leave its declaration."
    ));
    None
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
            // A bad-url ends at its first `)`, so the `;` after it ends the declaration.
            "url(a(b);color:red;--y:x)",
            "url(a(b);position:fixed;inset:0;background:url(https://evil.example/x);--y:x)",
            "URL(a\"b);color:red;--y:x)",
            "url(a b);color:red;--y:x)",
            "url(a\\\n);color:red;--y:x)",
            "u\\72l(a(b);color:red;--y:x)",
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
            "url( a.png )",
            "url( 'a b.png' )",
            "url(a\\(b.png)",
            "myurl(a(b))",
            "",
        ] {
            assert!(stays_in_declaration(value), "{value}");
        }
    }

    #[test]
    fn a_declaration_value_that_escapes_is_dropped() {
        assert_eq!(declaration_value("width", "10px"), Some("10px"));
        assert_eq!(declaration_value("width", "10px;position:fixed"), None);
        assert_eq!(declaration_value("width", "10px}"), None);
    }

    #[test]
    fn a_property_name_is_plain() {
        assert!(is_property_name("--lsx-gap"));
        assert!(is_property_name("background-color"));
        assert!(!is_property_name("color:red;x"));
        assert!(!is_property_name(""));
    }
}
