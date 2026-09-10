//! One binary for every native test, as `libero/tests/all` is. A unit mounts
//! one component; run one with `cargo test -p native-tests --test all switch::`.

mod menu;
mod slider;
mod switch;
mod tabs;
