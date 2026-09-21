//! Interaction archetypes: one parameterised keyboard and focus contract per APG pattern.
//! Assert only what the pattern guarantees; component specifics belong in that component's test.

mod combobox;
mod overlay;
mod radio_set;
mod roving;
mod tree_walk;

pub use combobox::Combobox;
pub use overlay::Overlay;
pub use radio_set::RadioSet;
pub use roving::{Orientation, RovingTabindex, count_tab_stops, reset_tab_position};
pub use tree_walk::TreeWalk;
