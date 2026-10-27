mod floating_window;
mod geometry;
mod options;
mod page_switch;
mod steps;
mod styles;
#[cfg(test)]
mod tests;
mod title_bar;

pub(crate) use floating_window::{FloatingWindow, WINDOW_MARKER};
pub use options::{FloatingWindowOptions, FloatingWindowPart, WindowRect};
