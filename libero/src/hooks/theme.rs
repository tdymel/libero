use dioxus::prelude::*;

use crate::{context::LiberoContext, theme::Theme};

/// The active theme.
///
/// Reactive: a component that reads a spacing number, a label or a `HexColor`
/// off it re-renders when the theme changes, so a Rust-side read never
/// disagrees with the CSS vars that changed without it.
pub fn use_theme() -> &'static Theme {
    *use_context::<LiberoContext>().theme.read()
}
