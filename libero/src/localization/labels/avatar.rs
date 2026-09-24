/// `AvatarGroup`'s overflow chip. `{n}` is the hidden count, `{names}` their
/// names joined by `, `.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AvatarLabels {
    /// The chip's visible text: `+3`.
    pub count: &'static str,
    /// The chip's accessible name: `3 more: Ada, Grace, Linus`.
    pub more: &'static str,
}

impl AvatarLabels {
    pub const ENGLISH: Self = Self {
        count: "+{n}",
        more: "{n} more: {names}",
    };

    pub const GERMAN: Self = Self {
        count: "+{n}",
        more: "{n} weitere: {names}",
    };
}
