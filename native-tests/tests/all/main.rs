//! One binary for every native test, as `libero/tests/all` is. A unit mounts
//! one component; run one with `cargo test -p native-tests --test all switch::`.

mod carousel;
mod chip;
mod choice;
mod color;
mod color_scheme;
mod combobox;
mod date;
mod dismiss;
mod focus_return;
mod form;
mod hover_card;
mod keyboard;
mod lightbox;
mod menu;
mod overlays;
mod pointer;
mod range_slider;
mod scroll;
mod segmented_control;
mod select;
mod slider;
mod spotlight;
mod switch;
mod table;
mod tabs;
mod text_field;
mod tree;
