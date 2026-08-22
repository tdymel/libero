mod core;
mod tab_value;
mod tabs;

pub use tab_value::TabValue;
pub use tabs::{Tabs, TabsProps};
// The derive and the trait share a name and one import, the way serde's do.
pub use libero_macros::TabValue;
