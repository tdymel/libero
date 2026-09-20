use std::fmt::{self, Display, Formatter};

/// One `property:value;` pair.
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

    pub(crate) fn property(&self) -> &str {
        &self.property
    }

    pub(crate) fn value(&self) -> &str {
        &self.value
    }
}

impl Display for CssDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{};", self.property, self.value)
    }
}

/// Renders a theme default as its CSS custom-property declarations.
pub(crate) trait ToCssDeclarations {
    fn to_css_declarations(&self) -> Vec<CssDeclaration>;
}
