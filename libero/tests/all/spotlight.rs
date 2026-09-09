//! `use_spotlight`'s markup: the search box is the combobox and names a
//! listbox and an option that exist, groups are named by a real element,
//! nothing is ever `aria-selected`, and the status region is there before it
//! has anything to announce. Keys and the hotkey need a browser: `e2e/tests/all/spotlight.rs`.

use std::cell::{Cell, RefCell};

use crate::common::{attributes_of, body};
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{SpotlightAction, SpotlightOptions, spotlight_filter, use_spotlight},
    theme::{Size, SpotlightDefaults, Theme},
};

thread_local! {
    static QUERY: RefCell<String> = const { RefCell::new(String::new()) };
    static LIMIT: Cell<Option<usize>> = const { Cell::new(None) };
    static LOADING: Cell<bool> = const { Cell::new(false) };
}

fn actions() -> Vec<SpotlightAction> {
    vec![
        SpotlightAction::new("Home").group("Pages").onclick(|_| {}),
        SpotlightAction::new("Settings")
            .group("Pages")
            .description("Preferences")
            .shortcut("Ctrl ,")
            .onclick(|_| {}),
        SpotlightAction::new("New file")
            .group("Commands")
            .onclick(|_| {}),
        SpotlightAction::new("Help")
            .shortcut("Ctrl+Shift+H")
            .onclick(|_| {}),
    ]
}

#[component]
fn Palette() -> Element {
    let spotlight = use_spotlight(SpotlightOptions {
        actions: Some(Callback::new(|_: String| {
            spotlight_filter(&QUERY.with(|q| q.borrow().clone()), &actions())
        })),
        limit: LIMIT.get(),
        loading: LOADING.get(),
        ..Default::default()
    });
    use_hook(|| spotlight.open());
    rsx! {}
}

fn app() -> Element {
    rsx! {
        LiberoProvider { Palette {} }
    }
}

fn rendered(query: &str, limit: Option<usize>) -> String {
    QUERY.with(|q| *q.borrow_mut() = query.to_string());
    LIMIT.set(limit);
    LOADING.set(false);
    render_app()
}

fn render_app() -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    // The open lands after `use_modal` read its empty slot, and the row count
    // reaches the search box's attributes a pass after that.
    for _ in 0..3 {
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
    body(&dioxus_ssr::render(&dom))
}

fn tags_with<'a>(html: &'a str, needle: &str) -> Vec<&'a str> {
    html.match_indices('<')
        .map(|(at, _)| &html[at..at + html[at..].find('>').unwrap()])
        .filter(|tag| tag.contains(needle))
        .collect()
}

#[test]
fn the_search_box_is_a_combobox_over_a_listbox_that_exists() {
    let html = rendered("", None);
    let input = attributes_of(&html, "input");
    assert_eq!(input["role"], "combobox");
    assert_eq!(input["aria-expanded"], "true");
    assert_eq!(input["autocomplete"], "off");
    assert!(input.contains_key("data-autofocus"));
    let listbox = &input["aria-controls"];
    assert!(html.contains(&format!(r#"id="{listbox}""#)));
    assert!(tags_with(&html, &format!(r#"id="{listbox}""#))[0].contains(r#"role="listbox""#));
    // Nothing highlighted until the arrows move.
    assert!(!input.contains_key("aria-activedescendant"));

    let dialog = tags_with(&html, r#"role="dialog""#);
    assert!(dialog[0].contains(r#"aria-label="Command palette""#));
}

#[test]
fn rows_are_options_in_named_groups_and_never_selected() {
    let html = rendered("", None);
    let options = tags_with(&html, r#"role="option""#);
    assert_eq!(options.len(), 4);
    assert!(!html.contains("aria-selected"));

    // The ungrouped "Help" row sits bare in the listbox, with no unnamed group.
    let groups = tags_with(&html, r#"role="group""#);
    assert_eq!(groups.len(), 2);
    for group in groups {
        let at = group.find(r#"aria-labelledby=""#).expect("a named group") + 17;
        let label_id = &group[at..at + group[at..].find('"').unwrap()];
        assert!(html.contains(&format!(r#"id="{label_id}""#)));
    }
    assert!(html.contains(">Pages<") && html.contains(">Commands<"));
}

#[test]
fn a_shortcut_hint_is_one_kbd_per_key() {
    let html = rendered("", None);
    let keys: Vec<&str> = html
        .match_indices("<kbd")
        .map(|(at, _)| {
            let text = &html[at + html[at..].find('>').unwrap() + 1..];
            &text[..text.find("</kbd>").unwrap()]
        })
        .collect();
    assert_eq!(keys, ["Ctrl", ",", "Ctrl", "Shift", "H"], "{html}");
    assert!(html.contains("</kbd> + <kbd"), "{html}");
}

/// `aria-activedescendant` is built by `ComboboxState` from its own id scheme,
/// and the rows spell that scheme out by hand - so a drift between the two
/// would point the search box at a row that does not exist. The highlight
/// itself moves by key, which the browser pass drives.
#[test]
fn rows_use_the_id_scheme_the_search_box_points_with() {
    let html = rendered("", None);
    let input = attributes_of(&html, "input");
    let id = input["aria-controls"]
        .trim_end_matches("-listbox")
        .to_string();
    let options = tags_with(&html, r#"role="option""#);
    for (index, option) in options.iter().enumerate() {
        assert!(
            option.contains(&format!(r#"id="{id}-option-{index}""#)),
            "{option}"
        );
    }
}

/// The region has to exist *before* it has anything to say, or a screen reader
/// never hears it fill. Filling it needs a typed query, which the browser
/// pass drives.
#[test]
fn an_empty_status_region_waits_for_nothing_found() {
    let html = rendered("", None);
    let status = &html[html.find(r#"role="status""#).expect("a status region")..];
    assert!(!status[..status.find("</div>").unwrap()].contains("Nothing found"));

    // An empty list for an empty query is not "nothing found".
    let html = rendered("zzz", None);
    assert!(tags_with(&html, r#"role="option""#).is_empty());
    let status = &html[html.find(r#"role="status""#).expect("a status region")..];
    assert!(!status[..status.find("</div>").unwrap()].contains("Nothing found"));
}

#[test]
fn the_limit_counts_through_groups() {
    let html = rendered("", Some(2));
    assert_eq!(tags_with(&html, r#"role="option""#).len(), 2);
    assert_eq!(tags_with(&html, r#"role="group""#).len(), 1);
}

/// While loading, the last query's rows are gone, the listbox is busy, and the
/// region that always exists says it is searching - never "nothing found".
#[test]
fn loading_says_searching_in_the_status_region_instead_of_rows() {
    QUERY.with(|q| *q.borrow_mut() = String::new());
    LIMIT.set(None);
    LOADING.set(true);
    let html = render_app();
    LOADING.set(false);

    assert!(tags_with(&html, r#"role="option""#).is_empty());
    assert!(tags_with(&html, r#"role="listbox""#)[0].contains(r#"aria-busy="true""#));
    let input = attributes_of(&html, "input");
    assert!(!input.contains_key("aria-activedescendant"));

    let status = &html[html.find(r#"role="status""#).expect("a status region")..];
    assert!(status.contains("Searching"), "{status}");
    assert!(!html.contains("Nothing found"));
    // The loader itself is silent: the region's text is what is said.
    let loader = tags_with(status, "aria-hidden");
    assert!(!loader.is_empty(), "{status}");
}

/// The theme's radius reaches the dialog as a size step. Spotlight used to
/// pass it as a CSS var string into `Dialog`'s `Size` prop, which could not
/// parse it, warned "unknown size" on every open and fell back to md - so a
/// themed radius was ignored (found by the E2E console pass, 2026-09-19).
#[test]
fn the_themed_radius_reaches_the_dialog() {
    static ROUND: Theme = Theme {
        spotlight: SpotlightDefaults {
            radius: Size::Xl,
            ..Theme::DEFAULT.spotlight
        },
        ..Theme::DEFAULT
    };
    fn round() -> Element {
        rsx! { LiberoProvider { theme: &ROUND, Palette {} } }
    }

    QUERY.with(|q| q.borrow_mut().clear());
    LIMIT.set(None);
    LOADING.set(false);
    let mut dom = VirtualDom::new(round);
    dom.rebuild_in_place();
    for _ in 0..3 {
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
    let html = body(&dioxus_ssr::render(&dom));

    let dialog = tags_with(&html, r#"role="dialog""#);
    assert_eq!(dialog.len(), 1, "{html}");
    assert!(
        dialog[0].contains("--lsx-dialog-radius:var(--lsx-radius-xl)"),
        "{}",
        dialog[0]
    );
}
