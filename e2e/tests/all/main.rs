//! One test binary, as `libero/tests/all/main.rs` does it: a full run links one
//! executable instead of one per file, and every test shares the single browser
//! and the single fixture server.
//!
//! A new component is `tests/all/<unit>.rs` plus its `mod` line here.

mod autocomplete;
mod focus_contrast;
mod modal;
mod negative;
mod notifications;
mod slider;
mod tabs;
mod tree;
