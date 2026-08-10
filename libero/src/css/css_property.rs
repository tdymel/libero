use crate::{common::ConstStr, sx::DeclarationProperty};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CssDeclarationProperty(pub DeclarationProperty);

impl CssDeclarationProperty {
    pub(crate) const fn to_const_str(self) -> ConstStr {
        match self.0 {
            DeclarationProperty::Known(property) => ConstStr::new().push_str(property.as_str()),
            DeclarationProperty::Raw(property) => ConstStr::new().push_str(property),
        }
    }
}
