use crate::common::ConstStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stylesheet {
    css: ConstStr,
}

impl Stylesheet {
    pub const fn new() -> Self {
        Self {
            css: ConstStr::new(),
        }
    }

    pub const fn from_const_str(css: ConstStr) -> Self {
        Self { css }
    }

    pub const fn as_str(&self) -> &str {
        self.css.as_str()
    }

    pub const fn into_const_str(self) -> ConstStr {
        self.css
    }

    pub(crate) const fn extend(mut self, css: ConstStr) -> Self {
        self.css = css;
        self
    }

    pub const fn append(mut self, value: &str) -> Self {
        self.css = self.css.push_str(value);
        self
    }

    pub const fn push_str(self, value: &str) -> Self {
        self.append(value)
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

    pub const fn start_at_rule(self, prefix: &str, condition: &str) -> Self {
        self.push_str(prefix).push_str(condition).push_char('{')
    }

    pub const fn end_declaration(self) -> Self {
        self.push_char(';')
    }

    pub const fn push_u8(mut self, value: u8) -> Self {
        if value >= 100 {
            self = self.push_char((b'0' + (value / 100)) as char);
            self = self.push_char((b'0' + ((value / 10) % 10)) as char);
            self = self.push_char((b'0' + (value % 10)) as char);
            return self;
        }

        if value >= 10 {
            self = self.push_char((b'0' + (value / 10)) as char);
            self = self.push_char((b'0' + (value % 10)) as char);
            return self;
        }

        self.push_char((b'0' + value) as char)
    }
}
