use crate::common::ConstVec;

use super::{
    declaration::{Declaration, DeclarationProperty, Property, ThemeAwareValue},
    sx_block::SxBlock,
    sx_modifier::SxModifier,
};

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

    pub const fn with(self, property: &'static str, value: &'static str) -> Self {
        self.with_property(DeclarationProperty::parse(property), value)
    }

    pub(super) const fn with_known_property(self, property: Property, value: &'static str) -> Self {
        self.with_property(DeclarationProperty::Known(property), value)
    }

    const fn with_property(mut self, property: DeclarationProperty, value: &'static str) -> Self {
        self.declarations.push(Declaration {
            property,
            value: ThemeAwareValue::parse(value),
        });
        self
    }

    pub(super) const fn modifier(mut self, modifier: SxModifier, nested: Sx) -> Self {
        let start = self.declarations.len();
        let parent = self.blocks.len();
        self.declarations.extend({
            let this = &nested;
            this.declarations.as_ref()
        });
        let end = self.declarations.len();
        self.blocks.push(SxBlock {
            modifier,
            start,
            end,
            parent: super::ROOT_BLOCK_PARENT,
        });

        let nested_blocks = {
            let this = &nested;
            this.blocks.as_ref()
        };
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

    pub const fn hash(&self) -> u64 {
        let mut hash = 0xcbf29ce484222325u64;

        let declarations = self.declarations.as_ref();
        let mut declaration_index = 0;
        while declaration_index < declarations.len() {
            hash = declarations[declaration_index].hash(hash);
            declaration_index += 1;
        }

        let blocks = self.blocks.as_ref();
        let mut block_index = 0;
        while block_index < blocks.len() {
            hash = blocks[block_index].hash(hash);
            block_index += 1;
        }

        hash
    }

    pub(crate) fn class_name(&self) -> String {
        format!("lsx-{:016x}", self.hash())
    }

    pub(crate) const fn declarations(&self) -> &[Declaration] {
        self.declarations.as_ref()
    }

    pub(crate) const fn blocks(&self) -> &[SxBlock] {
        self.blocks.as_ref()
    }
}

pub const fn sx() -> Sx {
    Sx::new()
}
