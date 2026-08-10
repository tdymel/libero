use crate::sx::{Declaration, DeclarationProperty, Property, ThemeAwareValue};

use super::{
    Stylesheet, css_color_value::CssColorValue, css_property::CssDeclarationProperty,
    css_var::SizeCssVar,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CssDeclaration(pub Declaration);

impl CssDeclaration {
    pub(crate) const fn to_const_str(self) -> crate::common::ConstStr {
        let mut css = crate::common::ConstStr::new();
        css = css.append(CssDeclarationProperty(self.0.property).to_const_str());
        css = css.push_char(':');
        css = css.append(self.value_to_const_str());
        css = css.push_char(';');
        css
    }

    pub(crate) const fn extend(self, css: &mut Stylesheet) {
        *css = css.append(self.to_const_str());
    }

    const fn value_to_const_str(self) -> crate::common::ConstStr {
        match self.0.value {
            ThemeAwareValue::Raw(value) => crate::common::ConstStr::from_str(value),
            ThemeAwareValue::Color(color_value) => CssColorValue(color_value).to_const_str(),
            ThemeAwareValue::Size(size) => match self.0.property {
                DeclarationProperty::Known(Property::PaddingTop) => {
                    SizeCssVar::SPACING.to_const_str(size)
                }
                _ => crate::common::ConstStr::from_str(size.as_str()),
            },
        }
    }
}
