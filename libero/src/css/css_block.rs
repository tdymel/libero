use super::css_scope::CssScope;

#[derive(Clone)]
pub enum CssBlock {
    Scope(CssScope),
    MediaQuery(CssMediaQuery),
}

impl CssBlock {
    pub(crate) fn to_string(self) -> String {
        match self {
            Self::Scope(scope) => scope.to_string(),
            Self::MediaQuery(media_query) => media_query.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct CssMediaQuery {
    condition: String,
    scopes: Vec<CssScope>,
}

impl CssMediaQuery {
    pub(crate) fn new(condition: &'static str) -> Self {
        Self::from_string(condition.to_string())
    }

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

    fn scopes(&self) -> &[CssScope] {
        &self.scopes
    }

    pub(crate) fn to_string(self) -> String {
        let mut css = String::from("@media ");
        css.push_str(&self.condition);
        css.push('{');

        let scopes = self.scopes();
        let mut scope_index = 0;
        while scope_index < scopes.len() {
            css.push_str(&scopes[scope_index].clone().to_string());
            scope_index += 1;
        }

        css.push('}');
        css
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        css::Stylesheet,
        sx::{Declaration, DeclarationProperty, Property, ThemeAwareValue},
    };

    use super::*;

    #[test]
    fn css_media_query_happy_path() {
        let scope = CssScope::new(".wambo").with(Declaration {
            property: DeclarationProperty::Known(Property::Width),
            value: ThemeAwareValue::Raw("120px"),
        });
        let media_query = CssMediaQuery::new("(min-width: 48em)").with(scope);

        let stylesheet = Stylesheet::new().append(CssBlock::MediaQuery(media_query).to_string());

        assert_eq!(
            stylesheet.as_str(),
            "@media (min-width: 48em){.wambo{width:120px;}}"
        );
    }
}
