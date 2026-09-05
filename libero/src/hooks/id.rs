use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::{core::AttributeValue, prelude::*};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn next_id() -> String {
    format!("lsx-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed))
}

/// A process-unique DOM id, stable for the component's lifetime - for the aria
/// wiring that has to name one instance's elements.
pub fn use_id() -> Signal<String> {
    use_signal(next_id)
}

/// [`use_id`], except a caller's own `id` attribute takes over. Components that
/// both need an id and spread `attributes` must use this - emitting both would
/// render two `id`s, and the browser keeps the caller's.
pub fn use_root_id(attributes: &[Attribute]) -> Signal<String> {
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
