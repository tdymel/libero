//! One binary for every native test, as `libero/tests/all` is. A unit mounts
//! one component; run one with `cargo test -p native-tests --test all switch::`.

mod carousel;
mod chip;
mod color_scheme;
mod dismiss;
mod focus_return;
mod hover_card;
mod keyboard;
mod lightbox;
mod menu;
mod pointer;
mod segmented_control;
mod select;
mod slider;
mod spotlight;
mod switch;
mod table;
mod tabs;
mod tree;
