//! One test binary, as `libero/tests/all/main.rs` does it: a full run links one
//! executable instead of one per file, and every test shares the single browser
//! and the single fixture server.
//!
//! A new component is `tests/all/<unit>.rs` plus its `mod` line here.

mod autocomplete;
mod collapse;
mod drawer;
mod focus_contrast;
mod isolation;
mod lightbox;
mod mark;
mod menu;
mod menubar;
mod modal;
mod multi_select;
mod negative;
mod notifications;
mod planted;
mod radio_group;
mod segmented_control;
mod select;
mod slider;
mod spotlight;
mod tabs;
mod tags_field;
mod tree;
