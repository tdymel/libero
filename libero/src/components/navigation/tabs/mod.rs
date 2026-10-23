mod core;
mod tabs;

pub use crate::theme::{TabsActivation, TabsPlacement};
pub(crate) use core::{TabSpec, TabsView, render_tabs};
pub use tabs::{Tabs, TabsPart, TabsProps};
