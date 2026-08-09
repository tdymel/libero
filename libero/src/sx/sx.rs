use crate::common::ConstVec;

use super::{declaration::Declaration, selector_block::SelectorBlock};

const DEFAULT_SX_DECLARATION_CAPACITY: usize = 64;
const DEFAULT_SX_SELECTOR_BLOCK_CAPACITY: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sx {
    declarations: ConstVec<Declaration, DEFAULT_SX_DECLARATION_CAPACITY>,
    selector_blocks: ConstVec<SelectorBlock, DEFAULT_SX_SELECTOR_BLOCK_CAPACITY>,
}

impl Default for Sx {
    fn default() -> Self {
        Self::new()
    }
}

impl Sx {
    pub const fn new() -> Self {
        Self {
            declarations: ConstVec::new_with_max_size(),
            selector_blocks: ConstVec::new_with_max_size(),
        }
    }

    pub const fn with(mut self, property: &'static str, value: &'static str) -> Self {
        self.declarations.push(Declaration { property, value });
        self
    }

    pub const fn background(self, value: &'static str) -> Self {
        self.with("background", value)
    }

    pub const fn width(self, value: &'static str) -> Self {
        self.with("width", value)
    }

    pub const fn height(self, value: &'static str) -> Self {
        self.with("height", value)
    }

    pub const fn nested(mut self, selector: &'static str, nested: Sx) -> Self {
        let start = self.declarations.len();
        self.declarations.extend(nested.declarations());
        let end = self.declarations.len();
        self.selector_blocks.push(SelectorBlock {
            selector,
            start,
            end,
        });
        self
    }

    pub const fn hover(self, nested: Sx) -> Self {
        self.nested(":hover", nested)
    }

    pub const fn focus(self, nested: Sx) -> Self {
        self.nested(":focus", nested)
    }

    pub const fn declarations(&self) -> &[Declaration] {
        self.declarations.as_ref()
    }

    pub const fn selector_blocks(&self) -> &[SelectorBlock] {
        self.selector_blocks.as_ref()
    }
}

pub const fn sx() -> Sx {
    Sx::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sx_happy_path() {
        const STYLE: Sx = sx()
            .background("red")
            .hover(sx().background("blue").width("120px"))
            .height("200px")
            .focus(sx().height("220px"));

        assert_eq!(
            STYLE.declarations(),
            &[
                Declaration {
                    property: "background",
                    value: "red",
                },
                Declaration {
                    property: "background",
                    value: "blue",
                },
                Declaration {
                    property: "width",
                    value: "120px",
                },
                Declaration {
                    property: "height",
                    value: "200px",
                },
                Declaration {
                    property: "height",
                    value: "220px",
                },
            ]
        );

        assert_eq!(
            STYLE.selector_blocks(),
            &[
                SelectorBlock {
                    selector: ":hover",
                    start: 1,
                    end: 3,
                },
                SelectorBlock {
                    selector: ":focus",
                    start: 4,
                    end: 5,
                },
            ]
        );
    }
}
