use std::fmt::{self, Display, Formatter};

use super::css_scope::CssScope;

#[derive(Clone, Debug, PartialEq)]
pub enum CssBlock {
    Scope(CssScope),
    MediaQuery(CssMediaQuery),
}

impl Display for CssBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scope(scope) => write!(f, "{scope}"),
            Self::MediaQuery(media_query) => write!(f, "{media_query}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssMediaQuery {
    condition: String,
    scopes: Vec<CssScope>,
}

impl CssMediaQuery {
    pub(crate) fn from_string(condition: String) -> Self {
        Self {
            condition,
            scopes: Vec::new(),
        }
    }

    pub(crate) fn with(mut self, scope: CssScope) -> Self {
        self.scopes.push(scope);
        self
    }
}

impl Display for CssMediaQuery {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "@media {}{{", self.condition)?;
        for scope in &self.scopes {
            write!(f, "{scope}")?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use crate::css::{CssDeclaration, Stylesheet};

    use super::*;

    #[test]
    fn css_media_query_happy_path() {
        let scope = CssScope::new(".wambo").with(CssDeclaration::new("width", "120px"));
        let media_query = CssMediaQuery::from_string("(min-width: 48em)".to_string()).with(scope);

        let stylesheet = Stylesheet::from(
            crate::css::StylesheetBuilder::new().with_block(CssBlock::MediaQuery(media_query)),
        );

        assert_eq!(
            stylesheet.as_str(),
            "@media (min-width: 48em){.wambo{width:120px;}}"
        );
    }
}
