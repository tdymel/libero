use std::fmt::{self, Display, Formatter};

use super::css_declaration::CssDeclaration;

#[derive(Clone, Debug, PartialEq)]
pub struct CssScope {
    selector: String,
    declarations: Vec<CssDeclaration>,
}

impl CssScope {
    pub fn new(selector: &'static str) -> Self {
        Self::from_string(selector.to_string())
    }

    pub fn from_string(selector: String) -> Self {
        Self {
            selector,
            declarations: Vec::new(),
        }
    }

    pub fn with(mut self, declaration: CssDeclaration) -> Self {
        self.declarations.push(declaration);
        self
    }
}

impl Display for CssScope {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{{", self.selector)?;
        for declaration in &self.declarations {
            write!(f, "{declaration}")?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use crate::css::{Stylesheet, StylesheetBuilder};

    use super::*;

    #[test]
    fn css_scope_happy_path() {
        let scope = CssScope::new(".wambo")
            .with(CssDeclaration::new("width", "120px"))
            .with(CssDeclaration::new("padding-top", "var(--lsx-spacing-sm)"));

        let stylesheet = Stylesheet::from(StylesheetBuilder::new().with_scope(scope));

        assert_eq!(
            stylesheet.as_str(),
            ".wambo{width:120px;padding-top:var(--lsx-spacing-sm);}"
        );
    }
}
