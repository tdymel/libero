mod declaration;
mod sx;
mod sx_builder_props;
pub mod sx_modifier;

pub use declaration::{Property, SxModifierKey, SxPropertyKey};
pub use sx::{StaticSx, Sx, SxEntry, SxInput, sx};
pub use sx_modifier::SxModifier;

/*
 * TODO:
 * - Support breakpoints within individual declarations, e.g. color: { sm: "red"; md: "blue" }
 */
