mod core;
mod tabs;

pub use crate::theme::TabsActivation;
pub(crate) use core::{TabSpec, TabsView, render_tabs};
pub use tabs::{Tabs, TabsPart, TabsProps};
