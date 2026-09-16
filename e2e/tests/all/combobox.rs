//! Bare `Combobox`, its rows drawn by the caller with `ComboboxOption`.

/// Todo 642: the selected-row bar, until now pinned only through `Select` and
/// `MultiSelect`.
#[test]
fn the_selected_row_shows_the_on_state_line() {
    crate::select::selected_row_is_marked("/combobox");
}
