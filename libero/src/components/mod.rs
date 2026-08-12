mod hello_world;
mod layout;
mod util;

pub use hello_world::HelloWorld;
pub use layout::*;

/*
 * TODOs:
 * - Polymorphic components, e.g. Button uses "a" as root html element instead of div
 * - Currently the UserDynamic props overwrite the StaticUserProps too much.
 *   It should only actively overwrite it, when the defaults are different.
 * - We should separate "Inputs" from the SxValues going forward. This will scale better.
 * - Right now we gain nothing with "str" inputs like JustifyInput. Sure one could use it, but anything is allowed.
 *   -> Not allow variables to be set there? -> Only valid values. -> Panic?! -> Keine gute idee
 *   -> Vlt ja einfach ne Art SxValue, welches nen str, oder zahl sein kann? CSS zu typisieren ist unsinn.
 *   -> ThemeAwareValue brauchen wir hier wohl :/
 */
