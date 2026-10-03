use dioxus::prelude::*;

/// Adds one to a counter an effect watches, reading nothing, so a callback
/// outside every scope can record an event without subscribing.
pub(crate) fn bump(mut counter: Signal<u64>) {
    let next = counter.peek().wrapping_add(1);
    counter.set(next);
}
