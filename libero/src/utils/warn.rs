use dioxus::{dioxus_core::AttributeValue, prelude::*};

/// Debug-only warning. Plain string, no `dioxus::warn!` format semantics.
#[cfg(debug_assertions)]
pub(crate) fn warn(message: &str) {
    #[cfg(test)]
    WARNINGS.with_borrow_mut(|warnings| warnings.push(message.to_string()));
    dioxus::prelude::warn!("{message}");
}

// Per thread, and every test runs on its own, so a test sees only its warnings.
#[cfg(test)]
thread_local! {
    static WARNINGS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Every `warn()` this thread made since the last call.
#[cfg(test)]
pub(crate) fn take_warnings() -> Vec<String> {
    WARNINGS.take()
}

#[cfg(not(debug_assertions))]
pub(crate) fn warn(_message: &str) {}

/// Whether a spread `aria-label` or `aria-labelledby` names the element. A
/// `None` value renders no attribute, so it names nothing.
pub(crate) fn names_itself(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        matches!(attribute.name, "aria-label" | "aria-labelledby")
            && !matches!(attribute.value, AttributeValue::None)
    })
}

/// The one way a role that needs an accessible name says it has none: a
/// `warn()` on mount, never a required prop. Once per mount rather than per
/// render, so a `ProgressBar` ticking every frame does not flood the console.
///
/// A hook: call it on every render, whatever `named` is.
pub(crate) fn use_name_warning(named: bool, message: &'static str) {
    use_hook(move || {
        if !named {
            warn(message);
        }
    });
}

/// Whether a link target runs script on click. Browsers strip leading and
/// trailing controls and spaces, and tabs and newlines anywhere, before they
/// read the scheme, so `" Java\tScript:"` counts too.
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
