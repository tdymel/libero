/// The words a burger announces itself with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BurgerLabels {
    /// Names the button while `open` is unset: it opens something.
    pub open: &'static str,
    /// Names the button in both states once `open` is set: `aria-expanded`
    /// carries the state.
    pub toggle: &'static str,
}

impl BurgerLabels {
    pub const ENGLISH: Self = Self {
        open: "Open navigation",
        toggle: "Toggle navigation",
    };

    pub const GERMAN: Self = Self {
        open: "Navigation öffnen",
        toggle: "Navigation umschalten",
    };
}
