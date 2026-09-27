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

/// Warns once per mount that a role lacks its accessible name. A hook: call it on every
/// render, whatever `named` is.
pub(crate) fn use_name_warning(named: bool, message: &'static str) {
    use_hook(move || {
        if !named {
            warn(message);
        }
    });
}

/// Whether a link target runs script. Browsers strip controls, tabs and newlines first,
/// so `" Java\tScript:"` counts too.
pub(crate) fn is_javascript_url(url: &str) -> bool {
    let scheme: String = url
        .trim_matches(|c: char| c <= ' ')
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .take("javascript:".len())
        .collect();
    scheme.eq_ignore_ascii_case("javascript:")
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
    fn a_javascript_url_is_caught_however_it_is_spelled() {
        assert!(is_javascript_url("javascript:alert(1)"));
        assert!(is_javascript_url("JavaScript:alert(1)"));
        assert!(is_javascript_url("  javascript:void(0)"));
        assert!(is_javascript_url("\u{1}javascript:x"));
        assert!(is_javascript_url("java\tscript:x"));
        assert!(is_javascript_url("jav\nascript:x"));
    }

    #[test]
    fn an_ordinary_url_is_not_a_javascript_url() {
        assert!(!is_javascript_url(""));
        assert!(!is_javascript_url("/javascript:guide"));
        assert!(!is_javascript_url("https://example.com/?q=javascript:"));
        assert!(!is_javascript_url("javascript"));
        assert!(!is_javascript_url("#section"));
    }
}
