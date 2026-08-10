use crate::common::ConstStr;

use super::{CssBlock, CssScope};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stylesheet {
    class_name: Option<ConstStr>,
    css: ConstStr,
}

impl Stylesheet {
    pub const fn new() -> Self {
        Self {
            class_name: None,
            css: ConstStr::new(),
        }
    }

    pub const fn as_str(&self) -> &str {
        self.css.as_str()
    }

    pub const fn class_name(&self) -> Option<ConstStr> {
        self.class_name
    }

    pub const fn with_class_name(mut self, class_name: ConstStr) -> Self {
        self.class_name = Some(class_name);
        self
    }

    pub const fn append(mut self, value: ConstStr) -> Self {
        self.css = self.css.append(value);
        self
    }

    pub const fn append_scope(self, scope: CssScope) -> Self {
        self.append(scope.to_const_str())
    }

    pub const fn append_block(self, block: CssBlock) -> Self {
        self.append(block.to_const_str())
    }
}
