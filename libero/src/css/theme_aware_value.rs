use crate::{common::ConstStr, sx::ThemeAwareValue};

use super::css_var::SizeCssVar;

impl ThemeAwareValue {
    pub(crate) const fn push<const MAX_SIZE: usize>(
        self,
        css: ConstStr<MAX_SIZE>,
    ) -> ConstStr<MAX_SIZE> {
        match self {
            Self::Raw(value) => css.push_str(value),
            Self::Color(color_value) => color_value.push_var(css),
            Self::Spacing(size) => SizeCssVar::SPACING.push_var(css, size),
        }
    }
}
