use super::backend;

/// Whether the renderer blurs behind a `backdrop-filter`. Every anyrender
/// backend Blitz ships drops it, so a `glass` surface stays opaque there.
pub(crate) fn draws_backdrop_filter() -> bool {
    backend::DRAWS_BACKDROP_FILTER
}
