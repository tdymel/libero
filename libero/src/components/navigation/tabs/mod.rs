mod core;
mod tabs;

pub(crate) use core::{TabSpec, TabsView, render_tabs};
pub use tabs::{Tabs, TabsActivation, TabsProps};
