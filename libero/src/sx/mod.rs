mod declaration;
mod hash;
mod selector_block;
mod sx;
pub mod sx_block;
pub mod sx_modifier;
mod sx_props;

pub use declaration::{Declaration, DeclarationProperty, Property, ThemeAwareValue};
pub(crate) use selector_block::ROOT_BLOCK_PARENT;
pub use selector_block::SelectorBlock;
pub use sx::{Sx, sx};

/*
 * TODO:
 * - Support breakpoints within individual declarations, e.g. color: { sm: "red"; md: "blue" }
 * - Support TypedValues, but this requires const traits.
 * - Breakpoints: SmallerThan and LargerThan + Merging really required?
 *   => Offering both will make merging more difficult.
 */
