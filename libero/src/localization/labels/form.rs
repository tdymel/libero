/// What a `Form` says about its fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "compares by address; a miss on a copied fn only re-renders"
)]
pub struct FormLabels {
    /// A `required` field left empty, shown at submit when no rule of its own says more.
    pub required: &'static str,
    /// As `required`, for an unchecked `Checkbox` or `Switch`.
    pub required_check: &'static str,
    /// As `required`, for a select with nothing picked.
    pub required_select: &'static str,
    /// As `required`, for a `RadioGroup` or `SegmentedControl` with nothing picked.
    pub required_option: &'static str,
    /// How many problems block the submit, announced in place of the summary by
    /// `SummaryMode::Hidden`.
    pub error_count: fn(usize) -> String,
}

impl FormLabels {
    pub const ENGLISH: Self = Self {
        required: "Fill in this field.",
        required_check: "Check this box.",
        required_select: "Select an item in the list.",
        required_option: "Select one of these options.",
        error_count: |count| match count {
            1 => "1 error in the form.".to_string(),
            count => format!("{count} errors in the form."),
        },
    };

    pub const GERMAN: Self = Self {
        required: "Füllen Sie dieses Feld aus.",
        required_check: "Aktivieren Sie dieses Kästchen.",
        required_select: "Wählen Sie ein Element aus der Liste aus.",
        required_option: "Wählen Sie eine dieser Optionen aus.",
        error_count: |count| match count {
            1 => "1 Fehler im Formular.".to_string(),
            count => format!("{count} Fehler im Formular."),
        },
    };
}
