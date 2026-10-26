use super::{
    core::{CascaderLayout, active_descendant, controlled_id},
    dropdown::CASCADER_DROPDOWN_SX,
    rows::CASCADER_COLUMNS_SX,
};
use crate::css::Stylesheet;

/// Todo 3813: a searchable `Cascader` with no name leaves its search box named by the placeholder only.
#[test]
fn a_searchable_cascader_without_a_name_warns() {
    use crate::{LiberoProvider, components::Cascader, components::CascaderOption};
    use dioxus::prelude::*;

    fn warns(app: fn() -> Element) -> bool {
        crate::utils::warnings_of(app)
            .iter()
            .any(|warning| warning.starts_with("Cascader: searchable"))
    }
    fn bare() -> Element {
        rsx! {
            LiberoProvider {
                Cascader { searchable: true, data: vec![CascaderOption::new("tea", "Tea")], onchange: move |_: Option<String>| {} }
            }
        }
    }
    fn named() -> Element {
        rsx! {
            LiberoProvider {
                Cascader { "aria-label": "Category", searchable: true, data: vec![CascaderOption::new("tea", "Tea")], onchange: move |_: Option<String>| {} }
            }
        }
    }

    assert!(warns(bare));
    assert!(!warns(named));
}

/// Columns grow into a wider trigger's room, so chevrons reach the edge. Reads the rule, not a layout.
#[test]
fn the_columns_fill_the_dropdown() {
    let css = Stylesheet::from(&*CASCADER_COLUMNS_SX);
    let css = css.as_str();

    assert!(css.contains("flex:1 0 auto;"), "{css}");
}

/// Below `sm` only the cursor's column shows, under the back header; the width moves to `min-width`.
#[test]
fn a_narrow_screen_drills_into_one_column() {
    let css = Stylesheet::from(&*CASCADER_COLUMNS_SX);
    let css = css.as_str();
    let narrow = css
        .split_once("@media not all and (min-width: 48rem){")
        .map(|(_, rest)| rest)
        .unwrap_or_else(|| panic!("no narrow rule: {css}"));

    assert!(
        narrow.contains("[data-slot='column']:not(:last-child){display:none;}"),
        "{narrow}"
    );
    assert!(
        narrow.contains("[data-slot='drill-back']{display:flex;"),
        "{narrow}"
    );
    assert!(
        narrow.contains("min-width:var(--lsx-cascader-column-width);"),
        "{narrow}"
    );
    // Wide, the header and the parent row stay hidden.
    let (wide, _) = css.split_once("@media").unwrap();
    assert!(
        wide.contains("[data-slot='drill-back']{display:none;}"),
        "{wide}"
    );
    assert!(
        wide.contains("width:var(--lsx-cascader-column-width);"),
        "{wide}"
    );
}

/// Below `sm` the dropdown is a bottom sheet; the popover's class placement, printed first, must lose.
#[test]
fn a_narrow_screen_makes_the_dropdown_a_bottom_sheet() {
    let css = Stylesheet::from(&*CASCADER_DROPDOWN_SX);
    let css = css.as_str();
    let (wide, narrow) = css
        .split_once("@media not all and (min-width: 48rem){")
        .unwrap_or_else(|| panic!("no narrow rule: {css}"));
    assert!(wide.contains("left:var(--lsx-popover-x)"), "{wide}");

    // The foot rides above a browser keyboard (2173), else the screen's.
    for declaration in [
        "left:var(--lsx-sheet-left",
        "right:var(--lsx-sheet-right",
        "bottom:var(--lsx-sheet-foot",
    ] {
        assert!(narrow.contains(declaration), "{declaration} in {narrow}");
    }
    // `vh` first for a browser without `dvh`, which skips the `@supports` block.
    let (plain, dynamic) = narrow
        .split_once("@supports (height: 1dvh){")
        .unwrap_or_else(|| panic!("no dvh override: {narrow}"));
    assert!(plain.contains("max-height:min(70vh,"), "{plain}");
    assert!(dynamic.contains("max-height:min(70dvh,"), "{dynamic}");
}

/// A disabled row is dimmed text on top of `ComboboxOption`'s disabled look (todo 1132).
#[test]
fn a_disabled_row_dims_its_text() {
    let css = Stylesheet::from(&*CASCADER_COLUMNS_SX);

    assert!(
        css.as_str()
            .contains("[data-state~='disabled']{color:var(--lsx-text-dimmed);}"),
        "{}",
        css.as_str()
    );
}

/// `aria-controls` must name the column that holds the `aria-activedescendant` row.
#[test]
fn aria_controls_names_the_column_the_active_row_is_in() {
    let cursor = vec![1, 0];
    let controls = controlled_id("x-listbox", CascaderLayout::Columns, &cursor);
    let active = active_descendant("x", CascaderLayout::Columns, &cursor, None, false).unwrap();
    assert_eq!(controls, "x-listbox-1");
    // Column `{listbox_id}-{level}`, row `{id}-option-{level}-{index}`: the levels match.
    assert_eq!(active, "x-option-1-0");
    // The narrow parent row heads that same column.
    let parent = active_descendant("x", CascaderLayout::Columns, &cursor, None, true).unwrap();
    assert_eq!(parent, "x-option-1-parent");

    // Nothing highlighted: the one root column is named.
    assert_eq!(
        controlled_id("x-listbox", CascaderLayout::Columns, &[]),
        "x-listbox-0"
    );
    // `Paths` is one listbox, and it carries `listbox_id` itself.
    assert_eq!(
        controlled_id("x-listbox", CascaderLayout::Paths, &cursor),
        "x-listbox"
    );
}

/// Wider than a phone, the columns scroll inside the viewport-capped dropdown (`tests/all/combobox.rs`).
#[test]
fn the_columns_stay_inside_the_viewport() {
    let columns = Stylesheet::from(&*CASCADER_COLUMNS_SX);
    assert!(
        columns.as_str().contains("overflow-x:auto;"),
        "{}",
        columns.as_str()
    );
}

/// `use_cascader` reads `data` and `value` fresh each render, not from its first call.
#[test]
fn the_headless_state_follows_data_and_value() {
    use crate::{
        LiberoProvider,
        components::{CascaderDropdown, CascaderOption, use_cascader},
    };
    use dioxus::prelude::*;
    use std::cell::RefCell;

    thread_local! {
        static SEEN: RefCell<Vec<(Vec<String>, Vec<String>)>> = const { RefCell::new(Vec::new()) };
    }

    #[allow(non_snake_case)]
    fn Picker() -> Element {
        let mut tick = use_signal(|| 0);
        let ore = match tick() {
            0 => "iron",
            _ => "gold",
        };
        let mut ores = vec![CascaderOption::new("iron".to_string(), "Iron ore")];
        if tick() > 0 {
            ores.push(CascaderOption::new("gold".to_string(), "Gold ore"));
        }
        let state = use_cascader(
            vec![CascaderOption::new("t1".to_string(), "Tier 1").children(ores)],
            Some(ore.to_string()),
            |_| {},
        );
        let names: Vec<String> = state
            .a11y_attributes()
            .iter()
            .map(|attribute| attribute.name.to_string())
            .collect();
        SEEN.with_borrow_mut(|seen| seen.push((state.labels(), names)));
        use_effect(move || {
            if *tick.peek() == 0 {
                tick.set(1);
            }
        });
        rsx! {
            CascaderDropdown { state: state.clone(),
                button { "Ore" }
            }
        }
    }
    fn app() -> Element {
        rsx! {
            LiberoProvider { Picker {} }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    let seen = SEEN.take();
    let (first_labels, first_names) = &seen[0];
    assert_eq!(first_labels, &["Tier 1", "Iron ore"]);
    for name in ["id", "role", "aria-haspopup", "aria-expanded"] {
        assert!(first_names.iter().any(|n| n == name), "{first_names:?}");
    }
    let (last_labels, _) = seen.last().unwrap();
    assert_eq!(last_labels, &["Tier 1", "Gold ore"], "{seen:?}");
}

/// Todo 4235: a value no option holds still posts, so the field is not empty and the x shows.
#[test]
fn a_value_missing_from_data_still_fills_the_field() {
    use crate::{LiberoProvider, components::Cascader, components::CascaderOption};
    use dioxus::prelude::*;

    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Cascader {
                    label: "Category",
                    name: "category",
                    clearable: true,
                    data: vec![CascaderOption::new("tea", "Tea")],
                    value: Some("ghost".to_string()),
                    onchange: move |_: Option<String>| {},
                }
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("value=\"ghost\""), "{html}");
    assert!(html.contains("-clear\""), "{html}");
}

/// Todo 4236: a `Paths` row has no column to drill into, so a click on a branch closes as Enter does.
#[test]
fn a_paths_click_commits_and_closes_like_enter() {
    use super::rows::click_commit;

    // (whole_path, has_children, any_level, is_committed)
    assert_eq!(click_commit(true, true, true, false), Some(true));
    assert_eq!(click_commit(true, true, true, true), Some(true));
    assert_eq!(click_commit(true, false, false, false), Some(true));
    // Columns: a branch drills in, picked only under `any_level` and never deselected.
    assert_eq!(click_commit(false, true, true, false), Some(false));
    assert_eq!(click_commit(false, true, true, true), None);
    assert_eq!(click_commit(false, true, false, false), None);
    assert_eq!(click_commit(false, false, false, false), Some(true));
}

/// Todo 4237: a closed list walks no paths; an open one walks them all.
#[test]
fn a_closed_list_walks_no_paths() {
    use std::{cell::RefCell, rc::Rc};

    use super::{
        core::{CascaderConfig, open_paths},
        option::erase,
    };
    use crate::components::CascaderOption;
    use dioxus::prelude::*;

    thread_local! {
        static SEEN: RefCell<Vec<Option<usize>>> = const { RefCell::new(Vec::new()) };
    }

    fn app() -> Element {
        let tree = [CascaderOption::<String>::new("t", "Tea").children(vec![
            CascaderOption::new("g", "Green"),
            CascaderOption::new("b", "Black"),
        ])];
        let config = CascaderConfig {
            options: Rc::new(erase(&tree)),
            committed: None,
            onpick: EventHandler::new(|_| {}),
            node: None,
            filter: None,
            separator: " / ".into(),
            any_level: false,
            allow_deselect: false,
            layout: CascaderLayout::Paths,
            column_width: "220px".into(),
            search_placeholder: None,
        };
        SEEN.with(|seen| {
            for opened in [false, true] {
                let rows = open_paths(opened, &config, None).map(|rows| rows.len());
                seen.borrow_mut().push(rows);
            }
        });
        rsx! {}
    }

    VirtualDom::new(app).rebuild_in_place();
    assert_eq!(SEEN.take(), vec![None, Some(2)]);
}

/// Todo 4238: RTL `scrollLeft` counts down from 0, so the strip's end is a negative offset.
#[test]
fn the_column_strip_scrolls_to_its_inline_end() {
    use super::body::strip_end_x;

    assert!(strip_end_x(false) > 0.0);
    assert!(strip_end_x(true) < 0.0);
}

/// Todo 4240: the query is trimmed for a custom `filter` too, not only the default one.
#[test]
fn a_custom_filter_gets_the_trimmed_query() {
    use std::{cell::RefCell, rc::Rc};

    use super::{
        core::{CascaderMatch, visible_paths},
        option::erase,
    };
    use crate::components::CascaderOption;

    let nodes = erase(&[
        CascaderOption::<String>::new("tea", "Tea"),
        CascaderOption::new("milk", "Milk"),
    ]);
    let seen = Rc::new(RefCell::new(Vec::new()));
    let filter = CascaderMatch(Rc::new({
        let seen = seen.clone();
        move |query: &str, _: &[usize], label: String| {
            seen.borrow_mut().push(query.to_string());
            label.to_lowercase().starts_with(query)
        }
    }));
    let rows = visible_paths(&nodes, false, Some("  te "), " / ", Some(&filter));
    assert_eq!(rows.len(), 1);
    assert!(seen.borrow().iter().all(|query| query == "te"), "{seen:?}");
    // The default filter trims the same way.
    assert_eq!(
        visible_paths(&nodes, false, Some(" mil "), " / ", None).len(),
        1
    );
}
