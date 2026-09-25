use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::{core::AttributeValue, prelude::*};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn next_id() -> String {
    format!("lsx-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed))
}

/// A process-unique DOM id, stable for the component's lifetime, for aria wiring.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_id;
/// # fn app() -> Element {
/// let id = use_id();
///
/// rsx! {
///     label { r#for: "{id}", "Name" }
///     input { id: "{id}" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/accessibility/use-id>
pub fn use_id() -> Signal<String> {
    use_signal(next_id)
}

/// [`use_id`], except a caller's own `id` attribute takes over. A component
/// that spreads `attributes` must use it, or it renders two `id`s.
pub(crate) fn use_root_id(attributes: &[Attribute]) -> Signal<String> {
    let caller = caller_id(attributes);
    let mut id = use_signal(|| caller.clone().unwrap_or_else(next_id));

    if let Some(caller) = caller
        && *id.peek() != caller
    {
        id.set(caller);
    }

    id
}

/// The first, because that is the one `Box` renders and browsers keep.
fn caller_id(attributes: &[Attribute]) -> Option<String> {
    attributes
        .iter()
        .find_map(|attribute| match (attribute.name, &attribute.value) {
            ("id", AttributeValue::Text(value)) => Some(value.clone()),
            _ => None,
        })
}

/// A selector for the element with `id`. An attribute selector, since `#1-faq`
/// throws: a caller's `id` need not be a CSS identifier.
pub(crate) fn id_selector(id: &str) -> String {
    let escaped = id.replace('\\', "\\\\").replace('"', "\\\"");
    format!("[id=\"{escaped}\"]")
}

#[cfg(test)]
mod tests {
    use super::id_selector;

    #[test]
    fn an_id_selector_takes_any_id() {
        assert_eq!(id_selector("1-email"), r#"[id="1-email"]"#);
        assert_eq!(id_selector("user.email"), r#"[id="user.email"]"#);
        assert_eq!(id_selector(r#"a"b\c"#), r#"[id="a\"b\\c"]"#);
    }
}
