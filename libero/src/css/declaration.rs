use crate::{
    common::ConstStr,
    sx::{Declaration, DeclarationProperty, Property, ThemeAwareValue},
};

use super::css_var::SizeCssVar;

impl Declaration {
    pub(crate) const fn push<const MAX_SIZE: usize>(
        self,
        mut css: ConstStr<MAX_SIZE>,
    ) -> ConstStr<MAX_SIZE> {
        css = self.property.push_name(css);
        css = css.push_char(':');
        css = self.push_value(css);
        css = css.push_char(';');
        css
    }

    const fn push_value<const MAX_SIZE: usize>(
        self,
        css: ConstStr<MAX_SIZE>,
    ) -> ConstStr<MAX_SIZE> {
        match self.value {
            ThemeAwareValue::Raw(value) => css.push_str(value),
            ThemeAwareValue::Color(color_value) => color_value.push_var(css),
            ThemeAwareValue::Size(size) => match self.property {
                DeclarationProperty::Known(Property::PaddingTop) => {
                    SizeCssVar::SPACING.push_var(css, size)
                }
                _ => css.push_str(size.as_str()),
            },
        }
    }
}
