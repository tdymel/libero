use std::fmt::Write;

use super::css_scope::CssScope;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stylesheet(String);

impl Stylesheet {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn new(scopes: Vec<CssScope>) -> Self {
        let mut css = String::new();
        for scope in scopes {
            write!(&mut css, "{scope}").expect("writing CSS scope into String cannot fail");
        }
        Self(css)
    }
}

impl From<String> for Stylesheet {
    fn from(value: String) -> Self {
        Self(value)
    }
}
