//! Not behind a feature: `ActionIcon`, `Chip` and `SegmentedControl` name these without `button` (todo 5670).

use crate::{
    components::common::{
        BUTTON_COLOR_VAR, BUTTON_CONTAINER_VAR, BUTTON_CONTRAST_VAR, BUTTON_FILL_VAR,
        BUTTON_HOVER_VAR, BUTTON_ON_CONTAINER_VAR, BUTTON_ON_STATE_VAR, BUTTON_SELECTED_VAR,
        Variant,
    },
    theme::CssVar,
};

/// The colour vars `button_variables` writes, which differ per variant.
pub(crate) const VARIANT_VARS: [CssVar; 8] = [
    BUTTON_COLOR_VAR,
    BUTTON_FILL_VAR,
    BUTTON_CONTRAST_VAR,
    BUTTON_HOVER_VAR,
    BUTTON_SELECTED_VAR,
    BUTTON_ON_STATE_VAR,
    BUTTON_CONTAINER_VAR,
    BUTTON_ON_CONTAINER_VAR,
];

/// `var` for one variant of a responsive one, as `--lsx-button-hover-v-filled`.
pub(crate) fn variant_var(var: &CssVar, variant: Variant) -> CssVar {
    CssVar::Owned(format!("{}-v-{}", var.name(), variant.as_str()))
}
