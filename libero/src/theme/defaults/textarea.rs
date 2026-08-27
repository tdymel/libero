use crate::theme::Size;

/// What `Textarea` does not share with every other field. The frame's numbers
/// live on `FieldDefaults`, so a `Textarea` under a `TextField` lines up with
/// it by construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextareaDefaults {
    pub size: Size,
    pub radius: Size,
}
