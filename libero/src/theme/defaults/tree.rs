use crate::theme::Size;

/// Theme defaults for `Tree`, set on [`Theme`](crate::theme::Theme).
/// It renders through `List`, so `size` picks one of `List`'s levels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeDefaults {
    pub size: Size,
}

impl TreeDefaults {
    pub const DEFAULT: Self = Self { size: Size::Md };
}
