//! Per-breakpoint value props of the data_display family (todo 5654).

use crate::{
    components::common::responsive_var_sx,
    sx::{Input, Sx, ThemeAwareValue},
    tokens::{CssVar, Responsive, SizeCss},
};

/// A plain value's inline var; a per-breakpoint one goes to [`var_sx`] instead.
pub(crate) fn plain_var(
    value: &Input<Responsive<ThemeAwareValue>>,
    scale: Option<SizeCss>,
) -> Option<String> {
    value
        .as_ref()
        .filter(|value| value.is_plain())
        .and_then(|value| value.base_ref().resolve(scale))
}

/// A per-breakpoint value's var for the user layer; `None` when plain or unset.
pub(crate) fn var_sx(
    value: &Input<Responsive<ThemeAwareValue>>,
    var: CssVar,
    scale: Option<SizeCss>,
) -> Option<Sx> {
    value
        .as_ref()
        .and_then(|value| responsive_var_sx(value, var, scale))
}

/// Both blocks, either, or none.
pub(crate) fn and_sx(first: Option<Sx>, second: Option<Sx>) -> Option<Sx> {
    match (first, second) {
        (Some(first), Some(second)) => Some(first.and(second)),
        (first, second) => first.or(second),
    }
}
