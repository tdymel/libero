/// What a `Form` says about its fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormLabels {
    /// A `required` field left empty, shown at submit when no rule of its own says more.
    pub required: &'static str,
}

impl FormLabels {
    pub const ENGLISH: Self = Self {
        required: "Fill in this field.",
    };

    pub const GERMAN: Self = Self {
        required: "Füllen Sie dieses Feld aus.",
    };
}
