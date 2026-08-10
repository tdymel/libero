use crate::sx::DeclarationProperty;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssDeclarationProperty(pub DeclarationProperty);

impl CssDeclarationProperty {
    pub(crate) fn to_string(self) -> String {
        match self.0 {
            DeclarationProperty::Known(property) => property.as_str().to_string(),
            DeclarationProperty::Raw(property) => property.to_string(),
        }
    }
}
