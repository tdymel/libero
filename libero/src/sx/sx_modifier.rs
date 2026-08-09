use crate::theme::Size;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SxModifier {
    Selector(&'static str),
    Condition(&'static str),
    Breakpoint(Size),
}
