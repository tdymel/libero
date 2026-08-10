use super::{CssBlock, CssScope};

#[derive(Debug, Clone, PartialEq)]
pub struct Stylesheet {
    class_name: Option<String>,
    css: String,
}

impl Stylesheet {
    pub fn new() -> Self {
        Self {
            class_name: None,
            css: String::new(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.css
    }

    pub fn class_name(&self) -> Option<&str> {
        self.class_name.as_deref()
    }

    pub fn with_class_name(mut self, class_name: String) -> Self {
        self.class_name = Some(class_name);
        self
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
