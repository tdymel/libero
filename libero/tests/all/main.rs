//! Every integration test, in one binary.
//!
//! Each file under `tests/` is its own crate that links dioxus and libero, so
//! one binary per component made a full run write dozens of them. The
//! modules share one link instead. Run a single unit by its path:
//!
//! ```text
//! cargo test -p libero --test all accordion::
//! ```
//!
//! `render_cost` stays its own binary: it is a release-mode measurement, not
//! part of the suite.

mod accordion;
mod action_icon;
mod alert;
mod anchor;
mod aspect_ratio;
mod attributes;
mod autocomplete;
mod avatar;
mod badge;
mod blockquote;
mod r#box;
mod burger;
mod button;
mod carousel;
mod cascader;
mod checkbox_card;
mod chip;
mod code;
mod code_block;
mod collapse;
mod color_picker;
mod combobox;
mod common;
mod container;
mod data_list;
mod date;
mod dialog;
mod divider;
mod drag;
mod drawer;
mod events;
mod field;
mod file_field;
mod flex;
mod float;
mod floating_window;
mod focus_trap;
mod grid;
mod header;
mod hover_card;
mod icon;
mod ids;
mod image;
mod image_list;
mod indicator;
mod kbd;
mod layer_order;
mod lightbox;
mod list;
mod loader;
mod mark;
mod marquee;
mod md_examples;
mod menu;
mod menubar;
mod modal;
mod nav_link;
mod no_has_selector;
mod notifications;
mod overlay;
mod override_vars;
mod pagination;
mod paper;
mod phone_field;
mod polymorphic_tiers;
mod portals;
mod presence;
mod progress_bar;
mod qr_code;
mod rsx_wrapping;
mod scroll_area;
mod scroller;
mod segmented_control;
mod select;
mod sidebar;
mod skeleton;
mod slider;
mod splitter;
mod spotlight;
mod stepper;
mod stylesheet;
mod table;
mod tabs;
mod tags_field;
mod text;
mod timeline;
mod title;
mod tooltip;
mod tree;
mod validation;
mod visually_hidden;
