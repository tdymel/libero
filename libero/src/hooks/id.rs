use dioxus::core::provide_root_context;
use dioxus::{core::AttributeValue, prelude::*};

/// The app's id counter, so a server render and the client's hydration count
/// from the same start.
#[derive(Clone, Copy)]
struct NextId(CopyValue<u64>);

fn next_id() -> String {
    let NextId(mut next) = try_consume_context::<NextId>()
        .unwrap_or_else(|| provide_root_context(NextId(CopyValue::new_in_scope(0, ScopeId::ROOT))));
    let id = *next.peek();
    next.set(id + 1);
    format!("lsx-{id}")
}

/// A DOM id unique within one app (`VirtualDom`), stable for the component's
/// lifetime, for aria wiring.
///
/// Ids follow render order, so a server render and its hydration agree. Known
/// limit: suspense that resolves out of order on the client can shift them.
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
    use dioxus::prelude::*;

    use super::{id_selector, use_id};

    #[test]
    fn an_id_selector_takes_any_id() {
        assert_eq!(id_selector("1-email"), r#"[id="1-email"]"#);
        assert_eq!(id_selector("user.email"), r#"[id="user.email"]"#);
        assert_eq!(id_selector(r#"a"b\c"#), r#"[id="a\"b\\c"]"#);
    }

    #[component]
    fn Field() -> Element {
        let id = use_id();
        rsx! {
            label { r#for: "{id}", "Name" }
            input { id: "{id}" }
        }
    }

    fn app() -> Element {
        rsx! {
            Field {}
            Field {}
        }
    }

    fn render() -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    /// Todo 1680: the hydration contract, a fresh dom of one app counts the same.
    #[test]
    fn two_renders_of_one_app_give_the_same_ids() {
        let first = render();
        assert!(first.contains(r#"id="lsx-0""#), "{first}");
        assert!(first.contains(r#"id="lsx-1""#), "{first}");
        assert_eq!(first, render());
    }
}
