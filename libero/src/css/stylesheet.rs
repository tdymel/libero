use crate::common::ConstStr;

use super::sx_to_css::DEFAULT_SX_CSS_CAPACITY;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stylesheet {
    css: ConstStr<DEFAULT_SX_CSS_CAPACITY>,
}

impl Stylesheet {
    pub const fn new() -> Self {
        Self {
            css: ConstStr::new(),
        }
    }

    pub const fn from_const_str(css: ConstStr<DEFAULT_SX_CSS_CAPACITY>) -> Self {
        Self { css }
    }

    pub const fn as_str(&self) -> &str {
        self.css.as_str()
    }

    pub const fn into_const_str(self) -> ConstStr<DEFAULT_SX_CSS_CAPACITY> {
        self.css
    }

    pub const fn push_str(mut self, value: &str) -> Self {
        self.css = self.css.push_str(value);
        self
    }

    pub const fn push_char(mut self, value: char) -> Self {
        self.css = self.css.push_char(value);
        self
    }

    pub const fn start_block(self, selector: &str) -> Self {
        self.push_str(selector).push_char('{')
    }

    pub const fn end_block(self) -> Self {
        self.push_char('}')
    }

    pub const fn start_root(self) -> Self {
        self.start_block(":root")
    }

    pub const fn start_media_min_width(self, breakpoint: &str) -> Self {
        self.push_str("@media (min-width: ")
            .push_str(breakpoint)
            .push_char(')')
            .push_char('{')
    }

    pub const fn end_declaration(self) -> Self {
        self.push_char(';')
    }
}
