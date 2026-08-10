use crate::sx::{Declaration, DeclarationProperty, Property, ThemeAwareValue};

use super::{
    Stylesheet, css_color_value::CssColorValue, css_property::CssDeclarationProperty,
    css_var::SizeCssVar,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CssDeclaration(pub Declaration);

impl CssDeclaration {
    pub(crate) const fn extend(self, css: &mut Stylesheet) {
        CssDeclarationProperty(self.0.property).extend_name(css);
        *css = css.push_char(':');
        self.extend_value(css);
        *css = css.push_char(';');
    }

    const fn extend_value(self, css: &mut Stylesheet) {
        match self.0.value {
            ThemeAwareValue::Raw(value) => {
                *css = css.push_str(value);
            }
            ThemeAwareValue::Color(color_value) => {
                CssColorValue(color_value).extend_var(css);
            }
            ThemeAwareValue::Size(size) => match self.0.property {
                DeclarationProperty::Known(Property::PaddingTop) => {
                    *css = css.extend(SizeCssVar::SPACING.push_var(css.into_const_str(), size));
                }
                _ => {
                    *css = css.push_str(size.as_str());
                }
            },
        }
    }
}
