use super::{CssBlock, CssScope};

#[derive(Debug, Clone, PartialEq)]
pub struct Stylesheet {
    css: String,
}

impl Stylesheet {
    pub fn new() -> Self {
        Self { css: String::new() }
    }

    pub fn as_str(&self) -> &str {
        &self.css
    }

    pub fn append(mut self, value: String) -> Self {
        self.css.push_str(&value);
        self
    }

    pub fn append_scope(self, scope: CssScope) -> Self {
        self.append(scope.to_string())
    }

    pub fn append_block(self, block: CssBlock) -> Self {
        self.append(block.to_string())
    }
}
