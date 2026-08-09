use super::sx_modifier::SxModifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SxBlock {
    pub modifier: SxModifier,
    pub start: usize,
    pub end: usize,
    pub parent: usize,
}
