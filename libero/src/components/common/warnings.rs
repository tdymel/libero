use dioxus::{dioxus_core::AttributeValue, prelude::*};

use crate::utils::warn;

/// Whether a spread `aria-label` or `aria-labelledby` names the element; `None` and a
/// blank text do not (accname ignores an empty `aria-label`).
pub(crate) fn names_itself(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        matches!(attribute.name, "aria-label" | "aria-labelledby")
            && match &attribute.value {
                AttributeValue::None => false,
                AttributeValue::Text(text) => !text.trim().is_empty(),
                _ => true,
            }
    })
}

/// A spread `aria-label`'s text, for a part of the control (its list) that carries the name too.
pub(crate) fn aria_label_of(attributes: &[Attribute]) -> Option<String> {
    attributes
        .iter()
        .find_map(|attribute| match &attribute.value {
            AttributeValue::Text(text)
                if attribute.name == "aria-label" && !text.trim().is_empty() =>
            {
                Some(text.clone())
            }
            _ => None,
        })
}

/// Warns once per mount that a role lacks its accessible name. A hook: call it on every
/// render, whatever `named` is.
pub(crate) fn use_name_warning(named: bool, message: &'static str) {
    use_hook(move || {
        if !named {
            warn(message);
        }
    });
}

/// The scheme of a link target that runs or embeds script. Browsers strip controls, tabs
/// and newlines first, so `" Java\tScript:"` counts too.
pub(crate) fn blocked_scheme(url: &str) -> Option<&'static str> {
    const BLOCKED: [&str; 3] = ["javascript", "data", "vbscript"];
    let head: String = url
        .trim_matches(|c: char| c <= ' ')
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .take("javascript:".len())
        .collect::<String>()
        .to_ascii_lowercase();
    let (scheme, _) = head.split_once(':')?;
    BLOCKED.into_iter().find(|blocked| *blocked == scheme)
}

/// Whether `url` may become an `href`. A `javascript:`, `data:` or `vbscript:` URL may
/// come from user data: it warns and the caller renders the link without an `href`.
pub(crate) fn allowed_href(url: &str, owner: &str) -> bool {
    let Some(scheme) = blocked_scheme(url) else {
        return true;
    };
    warn(&format!(
        "{owner}: a `{scheme}:` URL is not rendered as an href, as it would run script on click."
    ));
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attribute(name: &'static str) -> Attribute {
        Attribute::new(name, AttributeValue::Text("x".to_string()), None, false)
    }

    #[test]
    fn only_a_label_or_a_labelledby_names_an_element() {
        assert!(!names_itself(&[]));
        assert!(!names_itself(&[attribute("aria-describedby")]));
        assert!(names_itself(&[attribute("aria-label")]));
        assert!(names_itself(&[
            attribute("id"),
            attribute("aria-labelledby")
        ]));
        let unset = Attribute::new("aria-label", AttributeValue::None, None, false);
        assert!(!names_itself(&[unset]));
    }

    #[test]
    fn a_blank_label_does_not_name_an_element() {
        for name in ["aria-label", "aria-labelledby"] {
            for blank in ["", "  "] {
                let attribute =
                    Attribute::new(name, AttributeValue::Text(blank.into()), None, false);
                assert!(!names_itself(&[attribute]), "{name}={blank:?}");
            }
        }
    }

    #[test]
    fn a_blocked_scheme_is_caught_however_it_is_spelled() {
        for (url, scheme) in [
            ("javascript:alert(1)", "javascript"),
            ("JavaScript:alert(1)", "javascript"),
            ("  javascript:void(0)", "javascript"),
            ("\u{1}javascript:x", "javascript"),
            ("java\tscript:x", "javascript"),
            ("jav\nascript:x", "javascript"),
            ("jav\rascript:x", "javascript"),
            ("DATA:text/html,<b>", "data"),
            (" \tda\nta:text/html;base64,AAAA", "data"),
            ("VbScript:msgbox(1)", "vbscript"),
            ("vb\tscript:x", "vbscript"),
        ] {
            assert_eq!(blocked_scheme(url), Some(scheme), "{url:?}");
        }
    }

    #[test]
    fn an_ordinary_url_is_not_blocked() {
        for url in [
            "",
            "/javascript:guide",
            "/data:x",
            "https://example.com/?q=javascript:",
            "javascript",
            "data",
            "#section",
            "mailto:a@b.c",
            "tel:+4930",
            "java script:x",
            "java\u{1}script:x",
            "database:x",
            "metadata:x",
            "x:javascript:y",
        ] {
            assert_eq!(blocked_scheme(url), None, "{url:?}");
        }
    }
}
