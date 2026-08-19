use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// A process-unique DOM id, stable for the component's lifetime - for the
/// `dom_api()` lookups that scope an interaction to one instance.
pub fn use_id() -> Signal<String> {
    use_signal(|| format!("lsx-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed)))
}
