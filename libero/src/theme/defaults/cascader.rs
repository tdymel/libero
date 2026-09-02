use crate::theme::Size;

/// What `Cascader` does not share with every other field. The frame's numbers
/// live on `FieldDefaults` and the dropdown's - its padding, its rows, its
/// `max_dropdown_height` - on `ComboboxDefaults`, so a cascader lines up with a
/// `TextField` above it and with a `Select`'s list below it by construction.
///
/// `column_width` is the one number neither of those has: a cascader's list is
/// N columns wide rather than one, and nothing else in the library has a
/// column. It is a plain value read from Rust and written onto each column's
/// `style` - the component draws those columns itself, so no CSS var is needed
/// to reach them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CascaderDefaults {
    pub size: Size,
    pub radius: Size,
    pub column_width: &'static str,
}
