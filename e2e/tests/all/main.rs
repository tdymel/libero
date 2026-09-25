//! One test binary sharing one browser and one fixture server. A new component is
//! `tests/all/<unit>.rs` plus its `mod` line here.

mod accordion;
mod action_icon;
mod alert;
mod autocomplete;
mod avatar;
mod badge;
mod boundary;
mod button;
mod button_group;
mod calendar;
mod carousel;
mod cascader;
mod checkbox;
mod chip;
mod chrono_field;
mod code;
mod collapse;
mod color_field;
mod color_picker;
mod combobox;
mod copy_button;
mod data_list;
mod dialog;
mod direction_toggle;
mod divider;
mod docs_shell;
mod drawer;
mod dropdown_parts;
mod editor_ime_probe;
mod editor_probe;
mod elevation;
mod field_frame;
mod field_parts;
mod field_value;
mod file_field;
mod floating_window;
mod flows;
mod focus_contrast;
mod focus_return;
mod focus_start;
mod focus_trap;
mod form;
mod frames;
mod gradient;
mod grid_zone;
mod header;
mod hit_area;
mod home;
mod hover_card;
mod icon;
mod icon_provider;
mod image;
mod image_list;
mod isolation;
mod journal;
mod layout;
mod lightbox;
mod loader;
mod long_labels;
mod mark;
mod marquee;
mod menu;
mod menubar;
mod modal;
mod multi_select;
mod native_select;
mod nav_link;
mod negative;
mod nested_provider;
mod notifications;
mod number_field;
mod pagination;
mod perf;
mod phone_field;
mod picker_dialog;
mod picker_parts;
mod pin_field;
mod planted;
mod popover;
mod progress_bar;
mod qr_code;
mod radio_group;
mod range_slider;
mod rating;
mod refused;
mod repo_button;
mod rtl_keys;
mod rtl_layout;
mod scroll_area;
mod scroller;
mod segmented_control;
mod select;
mod skeleton;
mod slider;
mod sortable;
mod splitter;
mod spotlight;
mod stepper;
mod switch;
mod table;
mod tabs;
mod tags_field;
mod text_field;
mod textarea;
mod theme_toggle;
mod time_picker;
mod timeline;
mod tldr;
mod toolbar;
mod tooltip;
mod trailing_button;
mod transition;
mod tree;
mod typography;
mod use_accessibility;
mod use_hotkeys;
mod use_intersection;
mod use_long_press;
mod use_media_query;
mod use_timers;
mod visually_hidden;

/// Every `.rs` file here needs a `mod` line: an unregistered test file silently never runs.
/// Lives in `main.rs` so it cannot itself go unregistered; `libero/tests/all` has a copy.
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

    // `mod foo;` takes `foo.rs` or `foo/mod.rs`; anything else (`snapshots/`) is ignored.
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
