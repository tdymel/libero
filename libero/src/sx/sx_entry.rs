use super::{Sx, SxModifierKey, SxPropertyKey, ThemeAwareValue};

/// One entry of an [`Sx`]: a declaration or a nested modifier block.
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
