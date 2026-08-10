use crate::{common::ConstVec, sx::Declaration};

use super::{Stylesheet, css_declaration::CssDeclaration};

const DEFAULT_SCOPE_DECLARATION_CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssScope {
    specifier: &'static str,
    declarations: ConstVec<Declaration, DEFAULT_SCOPE_DECLARATION_CAPACITY>,
}

impl CssScope {
    pub const fn new(specifier: &'static str) -> Self {
        Self {
            specifier,
            declarations: ConstVec::new_with_max_size(),
        }
    }

    pub const fn with(mut self, declaration: Declaration) -> Self {
        self.declarations.push(declaration);
        self
    }

    pub const fn specifier(&self) -> &'static str {
        self.specifier
    }

    pub const fn declarations(&self) -> &[Declaration] {
        self.declarations.as_ref()
    }

    pub(crate) const fn extend(self, css: &mut Stylesheet) {
        *css = css.start_block(self.specifier);

        let declarations = self.declarations();
        let mut declaration_index = 0;
        while declaration_index < declarations.len() {
            CssDeclaration(declarations[declaration_index]).extend(css);
            declaration_index += 1;
        }

        *css = css.end_block();
    }
}

#[cfg(test)]
mod tests {
    use crate::sx::{DeclarationProperty, Property, ThemeAwareValue};

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

        let mut stylesheet = Stylesheet::new();
        scope.extend(&mut stylesheet);

        assert_eq!(scope.specifier(), ".wambo");
        assert_eq!(
            stylesheet.as_str(),
            ".wambo{width:120px;padding-top:var(--lsx-spacing-sm);}"
        );
    }
}
