#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectorBlock {
    pub selector: &'static str,
    pub start: usize,
    pub end: usize,
}
