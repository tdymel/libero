use super::backend;

/// Marks a field's own placeholder, drawn where the renderer draws none.
pub(crate) const PLACEHOLDER_ATTR: &str = "data-lsx-placeholder";
/// Set on a [`PLACEHOLDER_ATTR`] span while its control holds no text.
pub(crate) const PLACEHOLDER_SHOWN_ATTR: &str = "data-lsx-placeholder-shown";
/// The box a control shares with its [`PLACEHOLDER_ATTR`] span.
pub(crate) const PLACEHOLDER_CELL_ATTR: &str = "data-lsx-placeholder-cell";

/// Whether the renderer draws an input's `placeholder`. Blitz does not, so a
/// framed field draws its own there.
pub(crate) fn draws_placeholders() -> bool {
    backend::DRAWS_PLACEHOLDERS
}

/// A field's own placeholder mounted: the renderer marks it once laid out.
pub(crate) fn placeholder_drawn() {
    backend::placeholder_drawn();
}
