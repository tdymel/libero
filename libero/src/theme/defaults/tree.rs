use crate::theme::Size;

/// Which of `List`'s size levels a `Tree` uses by default for row
/// gap/indent - `Tree` renders through `List`/`role="group"` underneath, so
/// there's no separate size scale to declare here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeDefaults {
    pub size: Size,
}

impl TreeDefaults {
    pub const fn new(size: Size) -> Self {
        Self { size }
    }
}
