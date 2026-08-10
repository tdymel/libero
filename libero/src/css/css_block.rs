use crate::common::ConstVec;

use super::{Stylesheet, css_scope::CssScope};

const DEFAULT_MEDIA_QUERY_SCOPE_CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CssBlock {
    Scope(CssScope),
    MediaQuery(CssMediaQuery),
}

impl CssBlock {
    pub(crate) const fn extend(self, css: &mut Stylesheet) {
        match self {
            Self::Scope(scope) => scope.extend(css),
            Self::MediaQuery(media_query) => media_query.extend(css),
        }
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

    pub(crate) const fn extend(self, css: &mut Stylesheet) {
        *css = css.start_at_rule("@media ", self.condition);

        let scopes = self.scopes();
        let mut scope_index = 0;
        while scope_index < scopes.len() {
            scopes[scope_index].extend(css);
            scope_index += 1;
        }

        *css = css.end_block();
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

        let mut stylesheet = Stylesheet::new();
        CssBlock::MediaQuery(media_query).extend(&mut stylesheet);

        assert_eq!(
            stylesheet.as_str(),
            "@media (min-width: 48em){.wambo{width:120px;}}"
        );
    }
}
