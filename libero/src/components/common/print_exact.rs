use crate::sx::{Sx, sx};

/// Prints a state that is only a background (inherited by the parts): print drops them by default.
pub(crate) fn print_exact_sx() -> Sx {
    sx().with("print-color-adjust", "exact")
        .with("-webkit-print-color-adjust", "exact")
}
