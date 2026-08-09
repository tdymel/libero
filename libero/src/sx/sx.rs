use crate::common::{ConstStr, ConstVec};

use super::{declaration::Declaration, selector_block::SelectorBlock, sx_to_css};

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
        let parent = self.selector_blocks.len();
        self.declarations.extend(nested.declarations());
        let end = self.declarations.len();
        self.selector_blocks.push(SelectorBlock {
            selector,
            start,
            end,
            parent: sx_to_css::ROOT_SELECTOR_BLOCK_PARENT,
        });

        let nested_selector_blocks = nested.selector_blocks();
        let mut i = 0;
        while i < nested_selector_blocks.len() {
            let nested_selector_block = nested_selector_blocks[i];
            self.selector_blocks.push(SelectorBlock {
                selector: nested_selector_block.selector,
                start: start + nested_selector_block.start,
                end: start + nested_selector_block.end,
                parent: if nested_selector_block.parent == sx_to_css::ROOT_SELECTOR_BLOCK_PARENT {
                    parent
                } else {
                    parent + 1 + nested_selector_block.parent
                },
            });
            i += 1;
        }

        self
    }

    pub(crate) const fn declarations(&self) -> &[Declaration] {
        self.declarations.as_ref()
    }

    pub(crate) const fn selector_blocks(&self) -> &[SelectorBlock] {
        self.selector_blocks.as_ref()
    }

    pub const fn to_css(
        &self,
        class_name: &'static str,
    ) -> ConstStr<{ sx_to_css::DEFAULT_SX_CSS_CAPACITY }> {
        sx_to_css::to_css(self, class_name)
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
            .selector(".is-active", sx().height("260px"))
            .selector(
                " .nested",
                sx().background("purple").selector(
                    ":hover",
                    sx().height("280px")
                        .selector(" .nested_nested_nested", sx().width("300px")),
                ),
            );

        const CSS: ConstStr<{ sx_to_css::DEFAULT_SX_CSS_CAPACITY }> = STYLE.to_css(".button");
        assert_eq!(
            CSS.as_str(),
            ".button{background:red;height:200px;}.button:hover{background:blue;width:120px;}.button:focus{height:220px;}.button> .item{width:20px;}.button .label{background:green;}.button ~ .peer{height:240px;}.button + .next{width:140px;}.button:has(+ .prev){background:orange;}.button.is-active{height:260px;}.button .nested{background:purple;}.button .nested:hover{height:280px;}.button .nested:hover .nested_nested_nested{width:300px;}"
        );
    }
}
