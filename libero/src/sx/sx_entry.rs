use super::{Sx, SxModifierKey, SxPropertyKey, ThemeAwareValue};

/// One entry of an [`Sx`]: a declaration or a nested modifier block.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum SxEntry {
    /// `property: value`.
    Declaration {
        /// The property's name.
        property: SxPropertyKey,
        /// Resolved against the theme when the sheet is written.
        value: ThemeAwareValue,
    },
    /// `sx`'s entries, applied under `modifier` (a pseudo-class, media query, ...).
    Nested {
        /// Where the nested block applies.
        modifier: SxModifierKey,
        /// The nested block.
        sx: Sx,
    },
}
