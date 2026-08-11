use super::Size;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackAxisDefaults {
    pub align: &'static str,
    pub justify: &'static str,
    pub spacing: Size,
    pub wrap: bool,
}

impl StackAxisDefaults {
    pub const fn new(
        align: &'static str,
        justify: &'static str,
        spacing: Size,
        wrap: bool,
    ) -> Self {
        Self {
            align,
            justify,
            spacing,
            wrap,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackDefaults {
    pub column: StackAxisDefaults,
    pub row: StackAxisDefaults,
}

impl StackDefaults {
    pub const fn new(column: StackAxisDefaults, row: StackAxisDefaults) -> Self {
        Self { column, row }
    }
}
