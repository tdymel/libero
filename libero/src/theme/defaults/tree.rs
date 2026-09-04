use crate::theme::Size;

/// `Tree` renders through `List`, so it has no size scale of its own - this
/// only picks which of `List`'s levels it defaults to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeDefaults {
    pub size: Size,
}

impl TreeDefaults {
    pub const DEFAULT: Self = Self { size: Size::Md };
}
