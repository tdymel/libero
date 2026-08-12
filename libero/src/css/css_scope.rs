use std::fmt::{self, Display, Formatter};

use super::css_declaration::CssDeclaration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CssScope {
    selector: String,
    declarations: Vec<CssDeclaration>,
    media_query: Option<String>,
}

impl CssScope {
    pub(crate) fn new(selector: impl Into<String>, declarations: Vec<CssDeclaration>) -> Self {
        Self {
            selector: selector.into(),
            declarations,
            media_query: None,
        }
    }

    pub(crate) fn in_media_query(mut self, media_query: impl Into<String>) -> Self {
        self.media_query = Some(media_query.into());
        self
    }
}

impl Display for CssScope {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(media_query) = &self.media_query {
            write!(f, "@media {}{{", media_query)?;
        }

        write!(f, "{}{{", self.selector)?;
        for declaration in &self.declarations {
            write!(f, "{declaration}")?;
        }
        write!(f, "}}")?;

        if self.media_query.is_some() {
            write!(f, "}}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::css::Stylesheet;

    use super::*;

    #[test]
    fn css_scope_happy_path() {
        let stylesheet = Stylesheet::new(vec![CssScope::new(
            ".wambo",
            vec![
                CssDeclaration::new("width", "120px"),
                CssDeclaration::new("padding-top", "var(--lsx-spacing-sm)"),
            ],
        )]);

        assert_eq!(
            stylesheet.as_str(),
            ".wambo{width:120px;padding-top:var(--lsx-spacing-sm);}"
        );
    }
}
