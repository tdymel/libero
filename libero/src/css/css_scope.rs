use std::fmt::{self, Display, Formatter};

use super::at_rule::AtRule;
use super::css_declaration::CssDeclaration;

/// One rule: a selector and its declarations, inside zero or more at-rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CssScope {
    selector: String,
    declarations: Vec<CssDeclaration>,
    at_rules: Vec<AtRule>,
}

impl CssScope {
    pub(crate) fn new(selector: impl Into<String>, declarations: Vec<CssDeclaration>) -> Self {
        Self {
            selector: selector.into(),
            declarations,
            at_rules: Vec::new(),
        }
    }

    pub(crate) fn in_at_rules(mut self, at_rules: Vec<AtRule>) -> Self {
        self.at_rules = at_rules;
        self
    }
}

impl Display for CssScope {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for at_rule in &self.at_rules {
            write!(f, "{at_rule}{{")?;
        }

        write!(f, "{}{{", self.selector)?;
        for declaration in &self.declarations {
            write!(f, "{declaration}")?;
        }
        write!(f, "}}")?;

        for _ in &self.at_rules {
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

    #[test]
    fn at_rules_open_in_order_and_close_in_reverse() {
        let stylesheet = Stylesheet::new(vec![
            CssScope::new(".wambo", vec![CssDeclaration::new("width", "120px")]).in_at_rules(vec![
                AtRule::Media("(min-width: 48rem)".into()),
                AtRule::Container {
                    name: "card".into(),
                    condition: "(min-width: 640px)".into(),
                },
            ]),
        ]);

        assert_eq!(
            stylesheet.as_str(),
            "@media (min-width: 48rem){@container card (min-width: 640px){.wambo{width:120px;}}}"
        );
    }
}
