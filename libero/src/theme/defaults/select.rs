use crate::theme::Size;

/// What `Select` does not share with every other field. The frame's numbers -
/// font size, height, padding, radius scale - live on `FieldDefaults`, so a
/// `Select` and a `TextField` line up in one form by construction rather than
/// by two tables agreeing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectDefaults {
    pub size: Size,
    pub radius: Size,
}
