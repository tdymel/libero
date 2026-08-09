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

    pub const fn hover(self, nested: Sx) -> Self {
        self.selector(":hover", nested)
    }

    pub const fn focus(self, nested: Sx) -> Self {
        self.selector(":focus", nested)
    }

    pub const fn selector(mut self, selector: &'static str, nested: Sx) -> Self {
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

    pub(crate) const fn declarations(&self) -> &[Declaration] {
        self.declarations.as_ref()
    }

    pub(crate) const fn selector_blocks(&self) -> &[SelectorBlock] {
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
            .focus(sx().height("220px"))
            .selector("> .item", sx().width("20px"))
            .selector(" .label", sx().background("green"))
            .selector(" ~ .peer", sx().height("240px"))
            .selector(" + .next", sx().width("140px"))
            .selector(":has(+ .prev)", sx().background("orange"))
            .selector(".is-active", sx().height("260px"));

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
                Declaration {
                    property: "width",
                    value: "20px",
                },
                Declaration {
                    property: "background",
                    value: "green",
                },
                Declaration {
                    property: "height",
                    value: "240px",
                },
                Declaration {
                    property: "width",
                    value: "140px",
                },
                Declaration {
                    property: "background",
                    value: "orange",
                },
                Declaration {
                    property: "height",
                    value: "260px",
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
                SelectorBlock {
                    selector: "> .item",
                    start: 5,
                    end: 6,
                },
                SelectorBlock {
                    selector: " .label",
                    start: 6,
                    end: 7,
                },
                SelectorBlock {
                    selector: " ~ .peer",
                    start: 7,
                    end: 8,
                },
                SelectorBlock {
                    selector: " + .next",
                    start: 8,
                    end: 9,
                },
                SelectorBlock {
                    selector: ":has(+ .prev)",
                    start: 9,
                    end: 10,
                },
                SelectorBlock {
                    selector: ".is-active",
                    start: 10,
                    end: 11,
                },
            ]
        );
    }
}
