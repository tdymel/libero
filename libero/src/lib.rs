//! Libero: a Dioxus component library focused on DX, UX, a11y and configurability.
//!
//! Docs: <https://libero-ui.dev/about/getting-started>
#![allow(non_snake_case)]
// A component's `mod.rs` only re-exports, so `code/code.rs`, `tree/tree.rs`
// etc. are deliberate.
#![allow(clippy::module_inception)]
// A public type reachable only through a private path is a leak: re-export it,
// or seal it with an item-level `allow` (todo 469).
#![warn(unnameable_types)]
// On Android, clippy flags `thread_local!` initializers that already are
// `const { .. }` (7 false positives).
#![cfg_attr(target_os = "android", allow(clippy::missing_const_for_thread_local))]
// With components or hooks feature-gated off, the shared helpers only they use sit idle (todos 3887, 3922).
#![cfg_attr(
    not(all(feature = "all-components", feature = "all-hooks")),
    allow(dead_code, unused_imports, unused_macros, unnameable_types)
)]
#[cfg(all(test, not(feature = "test-features")))]
compile_error!("libero's tests need `--features test-features`");

pub mod components;
pub mod context;
pub mod hooks;
pub mod localization;
#[cfg(doctest)]
mod md_examples;
pub mod platform;

mod css;
mod str_enum;
pub mod sx;
#[cfg(test)]
mod test_converter;
pub mod theme;
mod tokens;
pub mod utils;

/// The date types the date and time components hold, re-exported so an app
/// names the same version.
pub use chrono;
pub(crate) use context::CssLayer;
pub use context::{IconProvider, IconSet, IconSlot, LiberoContext, LiberoProvider};
pub use css::Stylesheet;
#[cfg(feature = "use-color-scheme")]
pub use hooks::{ColorSchemeHandle, use_color_scheme};
pub use hooks::{
    FormatsHandle, LocalizationHandle, ThemeSetHandle, use_formats, use_formats_handle,
    use_localization, use_localization_handle, use_stylesheet, use_theme, use_theme_set,
};
