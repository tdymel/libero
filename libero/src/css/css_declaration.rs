use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CssDeclaration {
    property: String,
    value: String,
}

impl CssDeclaration {
    pub fn new(property: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            property: property.into(),
            value: value.into(),
        }
    }
}

impl Display for CssDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{};", self.property, self.value)
    }
}
