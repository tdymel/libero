use crate::common::ConstVec;
use crate::theme::Size;

use super::{
    declaration::{Declaration, DeclarationProperty, Property, ThemeAwareValue},
    sx::Sx,
    sx_block::SxBlock,
    sx_modifier::SxModifier,
};

const DEFAULT_SX_DECLARATION_CAPACITY: usize = 64;
const DEFAULT_SX_BLOCK_CAPACITY: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SxBuilder {
    declarations: ConstVec<Declaration, DEFAULT_SX_DECLARATION_CAPACITY>,
    blocks: ConstVec<SxBlock, DEFAULT_SX_BLOCK_CAPACITY>,
}

impl Default for SxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SxBuilder {
    pub const fn new() -> Self {
        Self {
            declarations: ConstVec::new_with_max_size(),
            blocks: ConstVec::new_with_max_size(),
        }
    }

    pub const fn build(self) -> Sx {
        Sx::new(self.declarations, self.blocks)
    }

    pub const fn with(self, property: &'static str, value: &'static str) -> Self {
        self.with_property(DeclarationProperty::parse(property), value)
    }

    const fn with_known_property(self, property: Property, value: &'static str) -> Self {
        self.with_property(DeclarationProperty::Known(property), value)
    }

    const fn with_property(mut self, property: DeclarationProperty, value: &'static str) -> Self {
        self.declarations.push(Declaration {
            property,
            value: ThemeAwareValue::parse(value),
        });
        self
    }

    pub const fn background(self, value: &'static str) -> Self {
        self.with_known_property(Property::Background, value)
    }

    pub const fn width(self, value: &'static str) -> Self {
        self.with_known_property(Property::Width, value)
    }

    pub const fn height(self, value: &'static str) -> Self {
        self.with_known_property(Property::Height, value)
    }

    pub const fn padding_top(self, value: &'static str) -> Self {
        self.with_known_property(Property::PaddingTop, value)
    }

    pub const fn hover(self, nested: SxBuilder) -> Self {
        self.selector(":hover", nested)
    }

    pub const fn focus(self, nested: SxBuilder) -> Self {
        self.selector(":focus", nested)
    }

    pub const fn when(self, condition: &'static str, nested: SxBuilder) -> Self {
        self.modifier(SxModifier::Condition(condition), nested)
    }

    pub const fn selector(self, selector: &'static str, nested: SxBuilder) -> Self {
        self.modifier(SxModifier::Selector(selector), nested)
    }

    pub const fn breakpoint(self, breakpoint: Size, nested: SxBuilder) -> Self {
        self.modifier(SxModifier::Breakpoint(breakpoint), nested)
    }

    const fn modifier(mut self, modifier: SxModifier, nested: SxBuilder) -> Self {
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

    const fn declarations(&self) -> &[Declaration] {
        self.declarations.as_ref()
    }

    const fn blocks(&self) -> &[SxBlock] {
        self.blocks.as_ref()
    }
}

pub const fn sx() -> SxBuilder {
    SxBuilder::new()
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
            )
            .build();

        let css = crate::css::Stylesheet::from(&STYLE);
        assert_eq!(STYLE.class_name(), "lsx-6b3bf9b9d4bfb407");
        assert_eq!(
            css.as_str(),
            ".lsx-6b3bf9b9d4bfb407{background:var(--lsx-primary-5);height:200px;padding-top:var(--lsx-spacing-sm);}.lsx-6b3bf9b9d4bfb407:hover{background:var(--lsx-primary-1);width:120px;}.lsx-6b3bf9b9d4bfb407:focus{height:220px;}.lsx-6b3bf9b9d4bfb407> .item{width:20px;}.lsx-6b3bf9b9d4bfb407 .label{background:green;}.lsx-6b3bf9b9d4bfb407 ~ .peer{height:240px;}.lsx-6b3bf9b9d4bfb407 + .next{width:140px;}.lsx-6b3bf9b9d4bfb407:has(+ .prev){background:orange;}.lsx-6b3bf9b9d4bfb407[data-state~=\"selected\"]{background:var(--lsx-secondary-5);}.lsx-6b3bf9b9d4bfb407 .nested{background:purple;}.lsx-6b3bf9b9d4bfb407 .nested:hover{height:280px;}.lsx-6b3bf9b9d4bfb407 .nested:hover .nested_nested_nested{width:300px;}@media (min-width: 48rem){.lsx-6b3bf9b9d4bfb407{width:400px;}}@media (min-width: 75rem){.lsx-6b3bf9b9d4bfb407{height:500px;}}@media (min-width: 75rem){.lsx-6b3bf9b9d4bfb407{background:var(--lsx-secondary-7);}}"
        );
    }
}
