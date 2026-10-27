/// `AvatarGroup`'s overflow chip. `{n}` is the hidden count, `{names}` the
/// known hidden names joined by `, `.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AvatarLabels {
    /// The chip's visible text: `+3`.
    pub count: &'static str,
    /// The chip's accessible name: `3 more`. Its tooltip describes it with the names.
    pub more: &'static str,
    /// The tooltip when `total` counts more people than are listed: `Grace and others`.
    pub others: &'static str,
}

impl AvatarLabels {
    pub const ENGLISH: Self = Self {
        count: "+{n}",
        more: "{n} more",
        others: "{names} and others",
    };

    pub const GERMAN: Self = Self {
        count: "+{n}",
        more: "{n} weitere",
        others: "{names} und weitere",
    };
}
