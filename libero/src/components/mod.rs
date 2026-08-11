mod hello_world;
mod layout;
mod util;

pub use hello_world::HelloWorld;
pub use layout::*;

/*
 * TODOs:
 * - Custom inline styles lead to variables for every instance to be set.
 *   The SxBuilder is const by design. One Alternative would be to prepare
 *   a combination of const theme alternatives, but this doesnt scale.
 *   Maybe we need some sort of "Runtime Sx"?
 * - Polymorphic components, e.g. Button uses "a" as root html element instead of div
 */
