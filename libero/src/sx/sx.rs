use crate::common::{ConstStr, ConstVec};

use super::{declaration::Declaration, sx_block::SxBlock, sx_modifier::SxModifier};
use crate::css;
use crate::theme::Size;

const DEFAULT_SX_DECLARATION_CAPACITY: usize = 64;
const DEFAULT_SX_BLOCK_CAPACITY: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sx {
    declarations: ConstVec<Declaration, DEFAULT_SX_DECLARATION_CAPACITY>,
    blocks: ConstVec<SxBlock, DEFAULT_SX_BLOCK_CAPACITY>,
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
            blocks: ConstVec::new_with_max_size(),
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

    pub const fn padding_top(self, value: &'static str) -> Self {
        self.with("padding-top", value)
    }

    pub const fn hover(self, nested: Sx) -> Self {
        self.selector(":hover", nested)
    }

    pub const fn focus(self, nested: Sx) -> Self {
        self.selector(":focus", nested)
    }

    pub const fn when(self, condition: &'static str, nested: Sx) -> Self {
        self.modifier(SxModifier::Condition(condition), nested)
    }

    pub const fn selector(self, selector: &'static str, nested: Sx) -> Self {
        self.modifier(SxModifier::Selector(selector), nested)
    }

    pub const fn breakpoint(self, breakpoint: Size, nested: Sx) -> Self {
        self.modifier(SxModifier::Breakpoint(breakpoint), nested)
    }

    const fn modifier(mut self, modifier: SxModifier, nested: Sx) -> Self {
        let start = self.declarations.len();
        let parent = self.blocks.len();
        self.declarations.extend(nested.declarations());
        let end = self.declarations.len();
        self.blocks.push(SxBlock {
            modifier,
            start,
            end,
            parent: super::ROOT_BLOCK_PARENT,
        });

        let nested_blocks = nested.blocks();
        let mut i = 0;
        while i < nested_blocks.len() {
            let nested_block = nested_blocks[i];
            self.blocks.push(SxBlock {
                modifier: nested_block.modifier,
                start: start + nested_block.start,
                end: start + nested_block.end,
                parent: if nested_block.parent == super::ROOT_BLOCK_PARENT {
                    parent
                } else {
                    parent + 1 + nested_block.parent
                },
            });
            i += 1;
        }

        self
    }

    pub(crate) const fn declarations(&self) -> &[Declaration] {
        self.declarations.as_ref()
    }

    pub(crate) const fn blocks(&self) -> &[SxBlock] {
        self.blocks.as_ref()
    }

    pub const fn to_css(&self, class_name: &'static str) -> ConstStr {
        css::to_css(self, class_name)
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
            .background("primary")
            .hover(sx().background("primary.1").width("120px"))
            .height("200px")
            .padding_top("sm")
            .focus(sx().height("220px"))
            .selector("> .item", sx().width("20px"))
            .selector(" .label", sx().background("green"))
            .selector(" ~ .peer", sx().height("240px"))
            .selector(" + .next", sx().width("140px"))
            .selector(":has(+ .prev)", sx().background("orange"))
            .when("selected", sx().background("secondary"))
            .selector(
                " .nested",
                sx().background("purple").selector(
                    ":hover",
                    sx().height("280px")
                        .selector(" .nested_nested_nested", sx().width("300px")),
                ),
            )
            .breakpoint(
                Size::Sm,
                sx().width("400px").breakpoint(
                    Size::Lg,
                    sx().height("500px")
                        .breakpoint(Size::Md, sx().background("secondary.7")),
                ),
            );

        const CSS: ConstStr = STYLE.to_css(".button");
        assert_eq!(
            CSS.as_str(),
            ".button{background:var(--lsx-primary-500);height:200px;padding-top:var(--lsx-spacing-sm);}.button:hover{background:var(--lsx-primary-100);width:120px;}.button:focus{height:220px;}.button> .item{width:20px;}.button .label{background:green;}.button ~ .peer{height:240px;}.button + .next{width:140px;}.button:has(+ .prev){background:orange;}.button[data-state~=\"selected\"]{background:var(--lsx-secondary-500);}.button .nested{background:purple;}.button .nested:hover{height:280px;}.button .nested:hover .nested_nested_nested{width:300px;}@media (min-width: 48em){.button{width:400px;}}@media (min-width: 75em){.button{height:500px;}}@media (min-width: 75em){.button{background:var(--lsx-secondary-700);}}"
        );
    }
}
