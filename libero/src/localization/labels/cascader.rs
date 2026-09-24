/// A `Cascader` on a narrow screen, which shows one level at a time.
/// `{label}` is the parent option's label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CascaderLabels {
    /// The header button that returns to the parent's level.
    pub back: &'static str,
    /// Under `any_level`, the first row, which picks the parent itself.
    pub select: &'static str,
}

impl CascaderLabels {
    pub const ENGLISH: Self = Self {
        back: "Back to {label}",
        select: "Select {label}",
    };

    pub const GERMAN: Self = Self {
        back: "Zurück zu {label}",
        select: "{label} auswählen",
    };
}
