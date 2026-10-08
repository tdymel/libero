/// What a `Form` says about its fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormLabels {
    /// A `required` field left empty, shown at submit when no rule of its own says more.
    pub required: &'static str,
    /// As `required`, for an unchecked `Checkbox` or `Switch`.
    pub required_check: &'static str,
    /// As `required`, for a select with nothing picked.
    pub required_select: &'static str,
}

impl FormLabels {
    pub const ENGLISH: Self = Self {
        required: "Fill in this field.",
        required_check: "Check this box.",
        required_select: "Select an item in the list.",
    };

    pub const GERMAN: Self = Self {
        required: "Füllen Sie dieses Feld aus.",
        required_check: "Aktivieren Sie dieses Kästchen.",
        required_select: "Wählen Sie ein Element aus der Liste aus.",
    };
}
