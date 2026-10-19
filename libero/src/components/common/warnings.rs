use dioxus::{
    dioxus_core::{AttributeValue, DynamicNode},
    prelude::*,
};

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

/// Whether `element` is the bare `rsx! {}`: a label that draws nothing.
pub(crate) fn is_empty_element(element: &Element) -> bool {
    element.as_ref().is_ok_and(|element| {
        element.template().strings().is_empty()
            && element
                .dynamic_node_values()
                .iter()
                .all(|node| matches!(node, DynamicNode::Fragment(children) if children.is_empty()))
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
    scheme_among(url, &["javascript", "data", "vbscript"])
}

fn scheme_among(url: &str, blocked: &[&'static str]) -> Option<&'static str> {
    let head: String = url
        .trim_matches(|c: char| c <= ' ')
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .take("javascript:".len())
        .collect::<String>()
        .to_ascii_lowercase();
    let (scheme, _) = head.split_once(':')?;
    blocked.iter().copied().find(|blocked| *blocked == scheme)
}

/// `url` for a media element's `src`, `poster` or `<track>`, or `None` with a warning for a
/// `javascript:` or `vbscript:` one. A `data:` file passes: media runs no script from it.
pub(crate) fn media_url(url: &str, owner: &str) -> Option<String> {
    let Some(scheme) = scheme_among(url, &["javascript", "vbscript"]) else {
        return Some(url.to_string());
    };
    warn(&format!(
        "{owner}: a `{scheme}:` URL is not rendered, as it is not a media file."
    ));
    None
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

    #[test]
    fn only_a_bare_rsx_is_an_empty_element() {
        let bare: Element = rsx! {};
        let text: Element = rsx! { "x" };
        let node: Element = rsx! { span {} };
        assert!(is_empty_element(&bare));
        assert!(is_empty_element(&VNode::empty()));
        assert!(!is_empty_element(&text));
        assert!(!is_empty_element(&node));
    }

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
    fn a_media_url_blocks_script_schemes_however_they_are_spelled() {
        for url in [
            "javascript:alert(1)",
            "JavaScript:alert(1)",
            "  javascript:void(0)",
            "java\tscript:x",
            "jav\nascript:x",
            "VbScript:msgbox(1)",
            "vb\tscript:x",
        ] {
            assert_eq!(media_url(url, "Video"), None, "{url:?}");
        }
    }

    #[test]
    fn a_media_url_passes_files_and_data() {
        for url in [
            "",
            "/launch.webm",
            "https://example.com/a.mp4?q=javascript:",
            "data:audio/wav;base64,AAAA",
            "DATA:text/vtt,WEBVTT",
            "blob:https://example.com/1",
        ] {
            assert_eq!(media_url(url, "Video").as_deref(), Some(url), "{url:?}");
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
