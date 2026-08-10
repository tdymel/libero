use crate::sx::{Declaration, DeclarationProperty, Property, ThemeAwareValue};

use super::{Stylesheet, css_var::SizeCssVar};

impl Declaration {
    pub(crate) const fn push(self, mut css: Stylesheet) -> Stylesheet {
        css = Stylesheet::from_const_str(self.property.push_name(css.into_const_str()));
        css = css.push_char(':');
        css = self.push_value(css);
        css = css.push_char(';');
        css
    }

    const fn push_value(self, css: Stylesheet) -> Stylesheet {
        match self.value {
            ThemeAwareValue::Raw(value) => css.push_str(value),
            ThemeAwareValue::Color(color_value) => {
                Stylesheet::from_const_str(color_value.push_var(css.into_const_str()))
            }
            ThemeAwareValue::Size(size) => match self.property {
                DeclarationProperty::Known(Property::PaddingTop) => Stylesheet::from_const_str(
                    SizeCssVar::SPACING.push_var(css.into_const_str(), size),
                ),
                _ => css.push_str(size.as_str()),
            },
        }
    }
}
