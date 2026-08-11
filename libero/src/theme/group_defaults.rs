use super::Size;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupDefaults {
    pub align: &'static str,
    pub justify: &'static str,
    pub gap: Size,
    pub wrap: bool,
}

impl GroupDefaults {
    pub const fn new(align: &'static str, justify: &'static str, gap: Size, wrap: bool) -> Self {
        Self {
            align,
            justify,
            gap,
            wrap,
        }
    }
}
