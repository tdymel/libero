#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Shade,
    Contrast,
}
