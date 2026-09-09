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
mod color_scheme;
mod color_scheme_button;
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
mod field_slot_ring;
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
mod theme_defaults;
mod theme_set;
mod timeline;
mod title;
mod tooltip;
mod tree;
mod validation;
mod visually_hidden;

/// Every `.rs` file in this directory must have a `mod` line above it.
///
/// A test file no `mod` reaches is not part of the crate: it never compiles and
/// never runs, and `cargo test` stays green with a count that looks right. Every
/// other registration hunk on the team checklist is a build error when you forget
/// it; this one is the single silent member, which is why it needs a guard rather
/// than a rule. `libero/tests/all/theme_set.rs` - 164 lines, five tests - was dead
/// from the day it was written for exactly this reason.
///
/// The guard sits in `main.rs` rather than a module of its own on purpose: a
/// module file has to be registered, so the guard would be dead in precisely the
/// case it exists to catch.
///
/// Only the first direction - a file nothing declares - is a silent defect. The
/// second - a `mod` line with no file - is a compile error, so it is reported
/// only to make the message unambiguous when the two lists differ. `e2e/tests/all/main.rs` carries its own copy for the
/// same reason - there is no crate the two test binaries could share it through.
#[test]
fn every_test_file_is_registered() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/all");
    let source = std::fs::read_to_string(dir.join("main.rs")).unwrap();

    // `mod r#box;` declares `box.rs`: the raw prefix is spelling, not part of the
    // file name. Comparing the literal token would fail on a correct file.
    let mut declared: Vec<String> = source
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("mod ")?.strip_suffix(';'))
        .map(|name| name.trim_start_matches("r#").to_owned())
        .collect();

    // A `mod foo;` is satisfied by `foo.rs` or by `foo/mod.rs`, so both count as
    // a file on disk. Anything else in the directory (`snapshots/`) is not a
    // module and is ignored.
    let mut on_disk: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension().is_some_and(|extension| extension == "rs")
                || path.join("mod.rs").is_file()
        })
        .map(|path| path.file_stem().unwrap().to_string_lossy().into_owned())
        .filter(|stem| stem != "main")
        .collect();

    declared.sort();
    on_disk.sort();

    let unregistered: Vec<&String> = on_disk
        .iter()
        .filter(|name| !declared.contains(name))
        .collect();
    let missing: Vec<&String> = declared
        .iter()
        .filter(|name| !on_disk.contains(name))
        .collect();

    assert!(
        unregistered.is_empty() && missing.is_empty(),
        "tests/all/main.rs and tests/all/ disagree.\n  \
         on disk with no `mod` line (these tests never run): {unregistered:?}\n  \
         declared with no file: {missing:?}"
    );
}
