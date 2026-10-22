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
