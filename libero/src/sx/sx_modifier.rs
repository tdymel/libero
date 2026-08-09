use super::breakpoint::Breakpoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SxModifier {
    Selector(&'static str),
    Breakpoint(Breakpoint),
}
