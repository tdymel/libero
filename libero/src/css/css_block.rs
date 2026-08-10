use crate::common::ConstVec;

use super::{Stylesheet, css_scope::CssScope};

const DEFAULT_MEDIA_QUERY_SCOPE_CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CssBlock {
    Scope(CssScope),
    MediaQuery(CssMediaQuery),
}

impl CssBlock {
    pub(crate) const fn to_const_str(self) -> crate::common::ConstStr {
        match self {
            Self::Scope(scope) => scope.to_const_str(),
            Self::MediaQuery(media_query) => media_query.to_const_str(),
        }
    }

    pub(crate) const fn extend(self, css: &mut Stylesheet) {
        *css = css.append(self.to_const_str().as_str());
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssMediaQuery {
    condition: &'static str,
    scopes: ConstVec<CssScope, DEFAULT_MEDIA_QUERY_SCOPE_CAPACITY>,
}

impl CssMediaQuery {
    pub const fn new(condition: &'static str) -> Self {
        Self {
            condition,
            scopes: ConstVec::new_with_max_size(),
        }
    }

    pub const fn with(mut self, scope: CssScope) -> Self {
        self.scopes.push(scope);
        self
    }

    pub const fn condition(&self) -> &'static str {
        self.condition
    }

    pub const fn scopes(&self) -> &[CssScope] {
        self.scopes.as_ref()
    }

    pub(crate) const fn to_const_str(self) -> crate::common::ConstStr {
        let mut css = crate::common::ConstStr::new()
            .push_str("@media ")
            .push_str(self.condition)
            .push_char('{');

        let scopes = self.scopes();
        let mut scope_index = 0;
        while scope_index < scopes.len() {
            css = css.push_str(scopes[scope_index].to_const_str().as_str());
            scope_index += 1;
        }

        css.push_char('}')
    }

    pub(crate) const fn extend(self, css: &mut Stylesheet) {
        *css = css.append(self.to_const_str().as_str());
    }
}

#[cfg(test)]
mod tests {
    use crate::sx::{Declaration, DeclarationProperty, Property, ThemeAwareValue};

    use super::*;

    #[test]
    fn css_media_query_happy_path() {
        let scope = CssScope::new(".wambo").with(Declaration {
            property: DeclarationProperty::Known(Property::Width),
            value: ThemeAwareValue::Raw("120px"),
        });
        let media_query = CssMediaQuery::new("(min-width: 48em)").with(scope);

        let stylesheet =
            Stylesheet::new().append(CssBlock::MediaQuery(media_query).to_const_str().as_str());

        assert_eq!(
            stylesheet.as_str(),
            "@media (min-width: 48em){.wambo{width:120px;}}"
        );
    }
}
