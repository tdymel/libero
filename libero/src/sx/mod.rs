mod declaration;
mod hash;
mod selector_block;
mod sx;
pub mod sx_block;
mod sx_builder;
mod sx_builder_props;
pub mod sx_modifier;

pub use declaration::{Declaration, DeclarationProperty, Property, ThemeAwareValue};
pub(crate) use selector_block::ROOT_BLOCK_PARENT;
pub use selector_block::SelectorBlock;
pub use sx::Sx;
pub use sx_builder::{SxBuilder, sx};

/*
 * TODO:
 * - Support breakpoints within individual declarations, e.g. color: { sm: "red"; md: "blue" }
 * - Support TypedValues, but this requires const traits.
 */
