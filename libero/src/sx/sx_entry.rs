use super::{Sx, SxModifierKey, SxPropertyKey, ThemeAwareValue};

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum SxEntry {
    Declaration {
        property: SxPropertyKey,
        value: ThemeAwareValue,
    },
    Nested {
        modifier: SxModifierKey,
        sx: Sx,
    },
}
