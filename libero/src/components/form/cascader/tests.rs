use super::{
    core::{CascaderLayout, active_descendant, controlled_id},
    dropdown::CASCADER_DROPDOWN_SX,
    rows::CASCADER_COLUMNS_SX,
};
use crate::css::Stylesheet;

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

/// Below `sm` the dropdown is a bottom sheet; the inline position of the popover must lose.
#[test]
fn a_narrow_screen_makes_the_dropdown_a_bottom_sheet() {
    let css = Stylesheet::from(&*CASCADER_DROPDOWN_SX);
    let css = css.as_str();
    let (_, narrow) = css
        .split_once("@media not all and (min-width: 48rem){")
        .unwrap_or_else(|| panic!("no narrow rule: {css}"));

    for declaration in [
        "left:0 !important;",
        "bottom:0 !important;",
        "width:100% !important;",
    ] {
        assert!(narrow.contains(declaration), "{declaration} in {narrow}");
    }
    // `vh` first for a browser without `dvh`, which skips the `@supports` block.
    let (plain, dynamic) = narrow
        .split_once("@supports (height: 1dvh){")
        .unwrap_or_else(|| panic!("no dvh override: {narrow}"));
    assert!(plain.contains("max-height:70vh;"), "{plain}");
    assert!(dynamic.contains("max-height:70dvh;"), "{dynamic}");
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
    let active = active_descendant("x", CascaderLayout::Columns, &cursor, None).unwrap();
    assert_eq!(controls, "x-listbox-1");
    // Column `{listbox_id}-{level}`, row `{id}-option-{level}-{index}`: the levels match.
    assert_eq!(active, "x-option-1-0");

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
