use crate::theme::Sizes;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogDefaults {
    pub size: Sizes<u16>,
}

impl DialogDefaults {
    pub const fn new(size: Sizes<u16>) -> Self {
        Self { size }
    }
}
