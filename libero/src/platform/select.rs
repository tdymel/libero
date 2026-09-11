use super::backend;

/// Whether a `<select>` opens a picker. Blitz draws none and ignores the
/// arrows, so `NativeSelect` draws `Select`'s listbox there instead.
pub(crate) fn select_picker() -> bool {
    backend::SELECT_PICKER
}
