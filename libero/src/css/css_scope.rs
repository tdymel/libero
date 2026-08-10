use crate::{common::ConstVec, sx::Declaration};

use super::css_declaration::CssDeclaration;

const DEFAULT_SCOPE_DECLARATION_CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssScope {
    specifier: crate::common::ConstStr,
    declarations: ConstVec<Declaration, DEFAULT_SCOPE_DECLARATION_CAPACITY>,
}

impl CssScope {
    pub const fn new(specifier: &'static str) -> Self {
        Self::from_const_str(crate::common::ConstStr::from_str(specifier))
    }

    pub const fn from_const_str(specifier: crate::common::ConstStr) -> Self {
        Self {
            specifier,
            declarations: ConstVec::new_with_max_size(),
        }
    }

    pub const fn with(mut self, declaration: Declaration) -> Self {
        self.declarations.push(declaration);
        self
    }

    pub const fn specifier(&self) -> &str {
        self.specifier.as_str()
    }

    pub const fn declarations(&self) -> &[Declaration] {
        self.declarations.as_ref()
    }

    pub(crate) const fn to_const_str(self) -> crate::common::ConstStr {
        let mut css = crate::common::ConstStr::new()
            .append(self.specifier)
            .push_char('{');

        let declarations = self.declarations();
        let mut declaration_index = 0;
        while declaration_index < declarations.len() {
            css = css.append(CssDeclaration(declarations[declaration_index]).to_const_str());
            declaration_index += 1;
        }

        css.push_char('}')
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        css::Stylesheet,
        sx::{DeclarationProperty, Property, ThemeAwareValue},
    };

    use super::*;

    #[test]
    fn css_scope_happy_path() {
        let scope = CssScope::new(".wambo")
            .with(Declaration {
                property: DeclarationProperty::Known(Property::Width),
                value: ThemeAwareValue::Raw("120px"),
            })
            .with(Declaration {
                property: DeclarationProperty::Known(Property::PaddingTop),
                value: ThemeAwareValue::Size(crate::theme::Size::Sm),
            });

        let stylesheet = Stylesheet::new().append(scope.to_const_str());

        assert_eq!(scope.specifier(), ".wambo");
        assert_eq!(
            stylesheet.as_str(),
            ".wambo{width:120px;padding-top:var(--lsx-spacing-sm);}"
        );
    }
}
