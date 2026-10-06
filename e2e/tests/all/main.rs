//! One test binary sharing one browser and one fixture server. A new component is
//! `tests/all/<unit>.rs` plus its `mod` line here.

mod accordion;
mod action_icon;
mod alert;
mod audio;
mod autocomplete;
mod avatar;
mod badge;
mod bottom_navigation;
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
mod copy;
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
mod field_props;
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
mod image_cropper;
mod image_list;
mod isolation;
mod journal;
mod kanban;
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
mod narrow_rows;
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
mod pictogram;
mod pin_field;
mod planted;
mod popover;
mod progress_bar;
mod qr_code;
mod radio_group;
mod range_slider;
mod rating;
mod refused;
mod repository;
mod rich_text_editor;
mod rtl_keys;
mod rtl_layout;
mod save_file;
mod scroll_area;
mod scroller;
mod segmented_control;
mod select;
mod settle;
mod shortcut_help;
mod sidebar;
mod skeleton;
mod slider;
mod sortable;
mod splitter;
mod spotlight;
mod stepper;
mod style_rules;
mod switch;
mod table;
mod table_detail;
mod table_groups;
mod table_overlay;
mod table_perf;
mod table_reorder;
mod table_resize;
mod table_toolbar;
mod tabs;
mod tags_field;
mod text_field;
mod textarea;
mod theme_switcher;
mod time_picker;
mod timeline;
mod tldr;
mod toolbar;
mod tooltip;
mod tour;
mod trailing_button;
mod transition;
mod tree;
mod typography;
mod use_accessibility;
mod use_back;
mod use_geolocation;
mod use_hotkeys;
mod use_intersection;
mod use_local_storage;
mod use_long_press;
mod use_media_query;
mod use_swipe;
mod use_system_notification;
mod use_timers;
mod use_user_media;
mod video;
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

/// Every baseline in `snapshots/` names a suite and a state the sources still quote (todo
/// 1722): a renamed suite or state leaves its old `.snap` behind, read by nothing.
#[test]
fn every_ax_baseline_is_referenced() {
    fn sources(dir: &std::path::Path, into: &mut Vec<(std::path::PathBuf, String)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                sources(&path, into);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                into.push((path.clone(), std::fs::read_to_string(&path).unwrap()));
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    sources(&root.join("tests/all"), &mut files);
    let tests = files.len();
    sources(&root.join("src"), &mut files);
    let quoted = |text: &str, literal: &str| text.contains(&format!("\"{literal}\""));
    // A state may come from a shared helper in `src/`, a suite's name never does.
    let shared_state = |state: &str| files[tests..].iter().any(|(_, text)| quoted(text, state));

    let mut unreferenced = Vec::new();
    for entry in std::fs::read_dir(root.join("tests/all/snapshots")).unwrap() {
        let file = entry.unwrap().file_name().to_string_lossy().into_owned();
        let Some(stem) = file.strip_suffix(".snap") else {
            continue;
        };
        // A plain `insta::assert_snapshot!` in `tests/all/<module>.rs`.
        if let Some((module, name)) = stem.strip_prefix("all__").and_then(|s| s.split_once("__")) {
            if !files.iter().any(|(path, text)| {
                path.file_stem().is_some_and(|stem| stem == module) && quoted(text, name)
            }) {
                unreferenced.push(file);
            }
            continue;
        }
        let (stem, dark) = match stem.strip_suffix("_dark") {
            Some(stem) => (stem, true),
            None => (stem, false),
        };
        let Some(base) = ["_desktop", "_mobile"]
            .into_iter()
            .find_map(|viewport| stem.strip_suffix(viewport))
        else {
            unreferenced.push(file);
            continue;
        };
        let referenced = base.match_indices('_').any(|(at, _)| {
            let (suite, state) = (&base[..at], &base[at + 1..]);
            files[..tests].iter().any(|(_, text)| {
                quoted(text, suite)
                    && (state == "rest" || quoted(text, state) || shared_state(state))
                    && (!dark || text.contains(".dark_snapshot("))
            })
        });
        if !referenced {
            unreferenced.push(file);
        }
    }
    unreferenced.sort();
    assert!(
        unreferenced.is_empty(),
        "baselines no suite or state in tests/all names any more; delete them, or \
         rename them with the suite or state:\n  {}",
        unreferenced.join("\n  ")
    );
}
