use std::fmt::Write;

use super::{css_block::CssBlock, css_scope::CssScope};

#[derive(Debug, Clone, PartialEq)]
pub struct Stylesheet(String);

impl Stylesheet {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Stylesheet {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StylesheetBuilder {
    blocks: Vec<CssBlock>,
}

impl StylesheetBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_scope(mut self, scope: CssScope) -> Self {
        self.blocks.push(CssBlock::Scope(scope));
        self
    }

    pub fn with_block(mut self, block: CssBlock) -> Self {
        self.blocks.push(block);
        self
    }
}

impl From<StylesheetBuilder> for Stylesheet {
    fn from(value: StylesheetBuilder) -> Self {
        let mut css = String::new();
        for block in value.blocks {
            write!(&mut css, "{block}").expect("writing CSS block into String cannot fail");
        }
        css.into()
    }
}
