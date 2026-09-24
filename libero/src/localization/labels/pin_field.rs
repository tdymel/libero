/// A `PinField`'s cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PinFieldLabels {
    /// Names a cell: `{n}` is its one-based position, `{m}` the count.
    pub cell: &'static str,
}

impl PinFieldLabels {
    pub const ENGLISH: Self = Self {
        cell: "Character {n} of {m}",
    };

    pub const GERMAN: Self = Self {
        cell: "Zeichen {n} von {m}",
    };
}
