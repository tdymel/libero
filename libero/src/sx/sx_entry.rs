use super::{Sx, SxModifierKey, SxPropertyKey, ThemeAwareValue};

/// One entry of an [`Sx`]: a declaration or a nested modifier block.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum SxEntry {
    /// `property: value`.
    Declaration {
        property: SxPropertyKey,
        value: ThemeAwareValue,
    },
    /// `sx`'s entries, applied under `modifier` (a pseudo-class, media query, ...).
    Nested { modifier: SxModifierKey, sx: Sx },
}
