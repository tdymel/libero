use crate::sx::{Declaration, DeclarationProperty, Property, ThemeAwareValue};

use super::{
    Stylesheet, css_color_value::CssColorValue, css_property::CssDeclarationProperty,
    css_var::SizeCssVar,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CssDeclaration(pub Declaration);

impl CssDeclaration {
    pub(crate) const fn to_const_str(
        self,
    ) -> crate::common::ConstStr<{ super::sx_to_css::DEFAULT_SX_CSS_CAPACITY }> {
        let mut css = crate::common::ConstStr::new();
        css = css.push_str(
            CssDeclarationProperty(self.0.property)
                .to_const_str()
                .as_str(),
        );
        css = css.push_char(':');
        css = css.push_str(self.value_to_const_str().as_str());
        css = css.push_char(';');
        css
    }

    pub(crate) const fn extend(self, css: &mut Stylesheet) {
        *css = css.append(self.to_const_str().as_str());
    }

    const fn value_to_const_str(
        self,
    ) -> crate::common::ConstStr<{ super::sx_to_css::DEFAULT_SX_CSS_CAPACITY }> {
        match self.0.value {
            ThemeAwareValue::Raw(value) => crate::common::ConstStr::new().push_str(value),
            ThemeAwareValue::Color(color_value) => CssColorValue(color_value).to_const_str(),
            ThemeAwareValue::Size(size) => match self.0.property {
                DeclarationProperty::Known(Property::PaddingTop) => {
                    SizeCssVar::SPACING.push_var(crate::common::ConstStr::new(), size)
                }
                _ => crate::common::ConstStr::new().push_str(size.as_str()),
            },
        }
    }
}
