use crate::{common::ConstVec, sx::Declaration};

use super::css_declaration::CssDeclaration;

const DEFAULT_SCOPE_DECLARATION_CAPACITY: usize = 48;

#[derive(Clone, Debug, PartialEq)]
pub struct CssScope {
    specifier: String,
    declarations: ConstVec<Declaration, DEFAULT_SCOPE_DECLARATION_CAPACITY>,
}

impl CssScope {
    pub fn new(specifier: &'static str) -> Self {
        Self::from_string(specifier.to_string())
    }

    pub fn from_string(specifier: String) -> Self {
        Self {
            specifier,
            declarations: ConstVec::new_with_max_size(),
        }
    }

    pub const fn with(mut self, declaration: Declaration) -> Self {
        self.declarations.push(declaration);
        self
    }

    pub fn specifier(&self) -> &str {
        &self.specifier
    }

    pub const fn declarations(&self) -> &[Declaration] {
        self.declarations.as_ref()
    }

    pub(crate) fn to_string(self) -> String {
        let mut css = self.specifier.clone();
        css.push('{');

        let declarations = self.declarations();
        let mut declaration_index = 0;
        while declaration_index < declarations.len() {
            css.push_str(&CssDeclaration(declarations[declaration_index]).to_string());
            declaration_index += 1;
        }

        css.push('}');
        css
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

        let stylesheet = Stylesheet::new().append(scope.clone().to_string());

        assert_eq!(scope.specifier(), ".wambo");
        assert_eq!(
            stylesheet.as_str(),
            ".wambo{width:120px;padding-top:var(--lsx-spacing-sm);}"
        );
    }
}
