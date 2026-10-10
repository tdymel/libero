/// Read by a screen reader after a step's label. The marker's glyph is
/// drawing only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepperLabels {
    pub completed: &'static str,
    /// So an error is not colour alone.
    pub error: &'static str,
    /// The name of a scrolling strip that is a tab stop, when the caller gave none.
    pub steps: &'static str,
}

impl StepperLabels {
    pub const ENGLISH: Self = Self {
        completed: "Completed",
        error: "Error",
        steps: "Steps",
    };

    pub const GERMAN: Self = Self {
        completed: "Abgeschlossen",
        error: "Fehler",
        steps: "Schritte",
    };
}
