use crate::theme::Sizes;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawerDefaults {
    pub size: Sizes<u16>,
}

impl DrawerDefaults {
    pub const fn new(size: Sizes<u16>) -> Self {
        Self { size }
    }
}
