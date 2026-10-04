/// A `use_tour` card's strings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TourLabels {
    /// Names a step's card when the step has no title and the tour no `aria_label`.
    pub label: &'static str,
    pub previous: &'static str,
    pub next: &'static str,
    /// The last step's Next button.
    pub done: &'static str,
    pub skip: &'static str,
    /// Where the tour stands: `{n}` is the step, `{m}` the step count.
    pub progress: &'static str,
    /// Names the card's close button.
    pub close: &'static str,
}

impl TourLabels {
    pub const ENGLISH: Self = Self {
        label: "Tour",
        previous: "Back",
        next: "Next",
        done: "Done",
        skip: "Skip",
        progress: "{n} of {m}",
        close: "Close tour",
    };

    pub const GERMAN: Self = Self {
        label: "Rundgang",
        previous: "Zurück",
        next: "Weiter",
        done: "Fertig",
        skip: "Überspringen",
        progress: "{n} von {m}",
        close: "Rundgang schließen",
    };
}
