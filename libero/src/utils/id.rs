use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// A process-unique number, for keys that never reach the DOM (a modal's or
/// a portal's stack entry). A DOM id comes from `use_id`.
pub(crate) fn unique_id() -> u64 {
    NEXT.fetch_add(1, Ordering::Relaxed)
}
