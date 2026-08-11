use super::Size;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackDefaults {
    pub align: &'static str,
    pub justify: &'static str,
    pub gap: Size,
}

impl StackDefaults {
    pub const fn new(align: &'static str, justify: &'static str, gap: Size) -> Self {
        Self {
            align,
            justify,
            gap,
        }
    }
}
