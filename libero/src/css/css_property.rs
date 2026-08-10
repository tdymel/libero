use crate::sx::DeclarationProperty;

use super::Stylesheet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CssDeclarationProperty(pub DeclarationProperty);

impl CssDeclarationProperty {
    pub(crate) const fn extend_name(self, css: &mut Stylesheet) {
        *css = match self.0 {
            DeclarationProperty::Known(property) => css.push_str(property.as_str()),
            DeclarationProperty::Raw(property) => css.push_str(property),
        };
    }
}
