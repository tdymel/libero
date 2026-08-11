use super::{
    hash::{hash_str, hash_u8},
    sx_modifier::SxModifier,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SxBlock {
    pub modifier: SxModifier,
    pub start: usize,
    pub end: usize,
    pub parent: usize,
}

impl SxBlock {
    pub(crate) const fn hash(self, mut hash: u64) -> u64 {
        hash = match self.modifier {
            SxModifier::Selector(selector) => hash_str(hash_u8(hash, 8), selector),
            SxModifier::Condition(condition) => hash_str(hash_u8(hash, 9), condition),
            SxModifier::Breakpoint(size) => hash_str(hash_u8(hash, 10), size.breakpoint_value()),
        };
        hash = hash_u8(hash, (self.start & 0xFF) as u8);
        hash = hash_u8(hash, (self.end & 0xFF) as u8);
        hash_u8(hash, (self.parent & 0xFF) as u8)
    }
}
