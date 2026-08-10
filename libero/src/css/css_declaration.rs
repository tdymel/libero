use crate::sx::{Declaration, DeclarationProperty, Property, ThemeAwareValue};

use super::{
    css_color_value::CssColorValue, css_property::CssDeclarationProperty, css_var::SizeCssVar,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssDeclaration(pub Declaration);

impl CssDeclaration {
    pub(crate) fn to_string(self) -> String {
        let mut css = String::new();
        css.push_str(&CssDeclarationProperty(self.0.property).to_string());
        css.push(':');
        css.push_str(&self.value_to_string());
        css.push(';');
        css
    }

    fn value_to_string(self) -> String {
        match self.0.value {
            ThemeAwareValue::Raw(value) => value.to_string(),
            ThemeAwareValue::Color(color_value) => CssColorValue(color_value).to_string(),
            ThemeAwareValue::Size(size) => match self.0.property {
                DeclarationProperty::Known(Property::PaddingTop) => {
                    SizeCssVar::SPACING.to_string(size)
                }
                _ => size.as_str().to_string(),
            },
        }
    }
}
