use crate::common::ConstVec;

use super::{declaration::Declaration, sx_block::SxBlock};

const DEFAULT_SX_DECLARATION_CAPACITY: usize = 64;
const DEFAULT_SX_BLOCK_CAPACITY: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sx {
    declarations: ConstVec<Declaration, DEFAULT_SX_DECLARATION_CAPACITY>,
    blocks: ConstVec<SxBlock, DEFAULT_SX_BLOCK_CAPACITY>,
}

impl Sx {
    pub const fn new(
        declarations: ConstVec<Declaration, DEFAULT_SX_DECLARATION_CAPACITY>,
        blocks: ConstVec<SxBlock, DEFAULT_SX_BLOCK_CAPACITY>,
    ) -> Self {
        Self {
            declarations,
            blocks,
        }
    }

    pub const fn hash(&self) -> u64 {
        let mut hash = 0xcbf29ce484222325u64;

        let declarations = self.declarations();
        let mut declaration_index = 0;
        while declaration_index < declarations.len() {
            hash = declarations[declaration_index].hash(hash);
            declaration_index += 1;
        }

        let blocks = self.blocks();
        let mut block_index = 0;
        while block_index < blocks.len() {
            hash = blocks[block_index].hash(hash);
            block_index += 1;
        }

        hash
    }

    pub fn class_name(&self) -> String {
        format!("lsx-{:016x}", self.hash())
    }

    pub(crate) const fn declarations(&self) -> &[Declaration] {
        self.declarations.as_ref()
    }

    pub(crate) const fn blocks(&self) -> &[SxBlock] {
        self.blocks.as_ref()
    }
}
