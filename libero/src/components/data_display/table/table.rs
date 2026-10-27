use std::{
    collections::{BTreeMap, HashMap, HashSet},
    rc::Rc,
};

use dioxus::prelude::*;

use super::super::sortable::{FixedSlots, SortableMove};
use crate::{
    components::{
        accessibility::use_announcer,
        common::{
            ClassList, HtmlTag, Input, LogicalTextAlign, Parts, Sides, States, Variables, attr,
            coarse_hit_area_sx, inset_focus_ring_sx, inset_ring_sides_sx, min_hit_area_sx,
            names_itself, use_name_warning, variables,
        },
        form::use_checkbox_look,
        layout::{scroll_area_base, use_box},
        overlay::MenuPart,
    },
    hooks::{
        use_debounced_callback, use_element, use_formats, use_id, use_localization,
        use_resize_fallback, use_theme,
    },
    platform::{
        ElementApi, SCROLL_MARGIN_BOTTOM_VAR, SCROLL_MARGIN_VAR, spaces_inline_boxes,
        sticks_table_heads, widens_sized_tables,
    },
    sx::{StaticSx, Sx, sx},
    theme::{
        CHECKBOX_BOX_SIZE, CssVar, NamedColorCss, Size, TABLE_GROUP, TABLE_PAD_X, TableDefaults,
        ios_text_floor_sx,
    },
};

use super::{
    cell_focus::{GridRow, head_levels},
    column::{Column, ColumnDefaults},
    column_drag::use_column_drag,
    column_filter::{ColumnFilter, FilterLogic, cell_tests},
    column_menu::{MenuColumn, use_menu_focus, use_pin_news},
    core::{BodySpec, RowFn, TableSort, UserSort, lead_count, render_body},
    detail::Details,
    edit::RowUpdate,
    filter_panel::use_panel_control,
    footer::Foot,
    grid::{GridRows, HeadGrid, col_indices, use_grid_focus},
    header_groups::header_rows,
    overlay::{EmptyBody, SKELETON_ROWS},
    paging::{use_page_reset, use_page_size_reseed},
    pinning::{PinSide, PinnedColumns, pin_edge_at},
    pipeline::{VisibleRow, selection_scope},
    resize::{ColumnResize, ColumnWidths},
    row::RowContext,
    row_reorder::{ROW_ANIMATION, ROW_TRANSFORM, RowReorder},
    selection::{Selection, TreeSelect, toggle_row},
    sticky::use_edge_height,
    toolbar::{TableTools, ToolView},
    use_table::{use_removed_columns, use_table},
    window::{
        TABLE_MIN_WIDTH_VAR, TABLE_ROW_HEIGHT_VAR, reveal_slot, use_row_focus, window_attributes,
        windowed_min_width, windowed_sx,
    },
};

mod chrome;
mod config;
mod layout;
mod row_specs;
mod visible_rows;
mod warnings;

use chrome::Chrome;
use layout::{FilterBase, HeaderLayout, Measures, MenuSpec, column_menus, header_layout};
use row_specs::{
    GroupSpec, RowsSpec, StateSpec, TreeSpec, body_rows, grid_window, group_row, header_action,
    row_state, select_action, tree_nav,
};
use visible_rows::{KeptRows, OrderedRows, keep_rows, order_rows, page_window};

/// A draggable header's grip lane, its press target.
const GRIP_LANE: &str = "24px";

const COARSE: &str = "(pointer: coarse)";

const ROW_RING_CELLS: &str = "& tbody > tr[aria-level][data-cell-ring]:focus-visible > *";
const ROW_RING_FIRST: &str =
    "& tbody > tr[aria-level][data-cell-ring]:focus-visible > :first-child";
const ROW_RING_LAST: &str = "& tbody > tr[aria-level][data-cell-ring]:focus-visible > :last-child";
const ROW_RING_ONLY: &str = "& tbody > tr[aria-level][data-cell-ring]:focus-visible > :only-child";

/// A row's cell wears its share of one ring round the row (todo 3341): its top and bottom, and `sides`.
fn row_ring_cell_sx(sides: Sides) -> Sx {
    inset_ring_sides_sx("-2px", sides)
}

/// Where the header filters row sticks on Blitz: the column headers' height.
const FILTERS_TOP_VAR: CssVar = CssVar::new("--lsx-table-filters-top");

/// How far below the viewport's top a `sticky_header` sticks; unset, a capped table's 0.
const STICKY_TOP_VAR: CssVar = CssVar::new("--lsx-table-sticky-top");

/// The sticky header's height, which a focus scroll in the page leaves clear.
const HEAD_HEIGHT_VAR: CssVar = CssVar::new("--lsx-table-head-height");

fn sticky_top() -> String {
    format!("var({}, 0px)", STICKY_TOP_VAR.name())
}

/// A bare `0` is no length inside a `calc()`.
fn sticky_length(offset: &str) -> String {
    match offset.trim() {
        "0" => "0px".to_string(),
        offset => offset.to_string(),
    }
}

/// The body's focusable parts, which a page scroll to focus would leave under the sticky header.
const PAGE_FOCUS_TARGETS: &str = "& tbody tr, & tbody td, & tbody th, \
     & tbody :is(a[href], button, input, select, textarea, [tabindex])";

static TABLE_SX: StaticSx = StaticSx::new(|| {
    let vars = TableDefaults::theme_vars();
    match widens_sized_tables() {
        true => vars.width("100%"),
        false => vars.min_width("100%"),
    }
    // Separate, with the row lines on the cells: Blitz's collapsing model paints
    // one grid from the first cell's top border, a black 3px one here (todo 772).
    .border_collapse("separate")
    .border_spacing("0")
    .selector(
        "& th, & td",
        sx().text_align_start().vertical_align("middle"),
    )
    .selector("& thead th", sx().font_weight("600"))
    .selector(
        "& tfoot td",
        sx().font_weight("600")
            .background(NamedColorCss::SURFACE.value()),
    )
    // The aggregate's name reads as a label to its value.
    .selector(
        "& [data-footer-name]",
        // Inline-block: Blitz ignores an inline box's horizontal margin.
        sx().display("inline-block")
            .font_size("0.85em")
            .font_weight("400")
            .color("text-dimmed")
            .with("margin-inline-end", "6px"),
    )
    // On a group row's tinted fill the dimmed token is 4.08:1: the row's ink, lighter by weight.
    .selector(
        "& tbody tr[data-row-group] [data-footer-name]",
        sx().color("inherit"),
    )
    // A row header is semantics, not a look: it reads like its row.
    .selector("& tbody th", sx().font_weight("inherit"))
    .selector("& caption", caption_sx())
    .selector(
        "& th[data-align=\"center\"], & td[data-align=\"center\"]",
        sx().text_align("center"),
    )
    .selector(
        "& th[data-align=\"end\"], & td[data-align=\"end\"]",
        sx().text_align_end(),
    )
    // The button takes the padding, so all of it clicks. Marked from Rust:
    // `th:has(button)` never matches natively.
    .selector("& th[data-sortable], & th[data-menu]", sx().padding("0"))
    .selector(
        "& [data-sort-button]",
        // Blitz's UA sheet centres a button's content: start it, as its cells.
        sx().display("flex")
            .align_items("center")
            .justify_content("flex-start")
            .gap("4px")
            .width("100%")
            .padding(TableDefaults::padding())
            .background("none")
            .border("0")
            .font("inherit")
            .letter_spacing("inherit")
            .color("inherit")
            .cursor("pointer"),
    )
    // Inset: the button fills its cell edge to edge.
    .selector(
        "& [data-sort-button]:focus-visible",
        inset_focus_ring_sx("-2px"),
    )
    // No transition on the arrow or the detail chevron: each frame of one repaints the
    // whole table's layer, 40-100 ms on 1000 rows (todo 1984).
    .selector(
        "& [data-sort-button] svg",
        sx().width("16px")
            .height("16px")
            .flex_shrink("0")
            .opacity("0"),
    )
    // `aria-sort`, on the sorted header only, doubles as the styling state.
    .selector(
        "& th[data-sortable]:not([aria-sort]) [data-sort-button]:hover svg, \
             & th[data-sortable]:not([aria-sort]) [data-sort-button]:focus svg",
        sx().opacity("0.5"),
    )
    .selector(
        "& th[aria-sort=\"ascending\"] [data-sort-button] svg, \
             & th[aria-sort=\"descending\"] [data-sort-button] svg",
        sx().opacity("1"),
    )
    .selector(
        "& th[aria-sort=\"ascending\"] [data-sort-button] svg",
        sx().transform("rotate(180deg)"),
    )
    // `text-align` doesn't position flex items.
    .selector(
        "& th[data-align=\"center\"] [data-sort-button]",
        sx().justify_content("center"),
    )
    .selector(
        "& th[data-align=\"end\"] [data-sort-button]",
        sx().justify_content("end"),
    )
    .selector(
        "& [data-sort-button] [data-sort-order]",
        sx().font_size("0.75em")
            .font_weight("600")
            .color("text-dimmed"),
    )
    // The label takes the room, the menu button its glyph's.
    .selector(
        "& [data-header]",
        sx().display("flex").align_items("center"),
    )
    .selector(
        "& [data-header] > [data-sort-button], & [data-header-text]",
        sx().flex("1").min_width("0"),
    )
    .selector(
        "& [data-header-text]",
        sx().padding(TableDefaults::padding()),
    )
    .selector(
        "& [data-column-menu]",
        sx().display("flex")
            .align_items("center")
            .justify_content("center")
            .width("24px")
            .height("24px")
            .margin_inline("4px")
            .padding("0")
            .border("0")
            .border_radius("4px")
            .background("none")
            .color("inherit")
            .cursor("pointer")
            .position("relative")
            .and(coarse_hit_area_sx("::before")),
    )
    .selector(
        "& [data-column-menu] svg, & [data-filtered] svg",
        sx().width("16px").height("16px"),
    )
    // A filtered header's button: as the menu button, but always shown.
    .selector(
        "& [data-filtered]",
        sx().display("flex")
            .align_items("center")
            .justify_content("center")
            .width("24px")
            .height("24px")
            .padding("0")
            .border("0")
            .border_radius("4px")
            .background("none")
            .color("inherit")
            .cursor("pointer")
            .position("relative")
            .and(coarse_hit_area_sx("::before")),
    )
    .selector(
        "& [data-filtered]:focus-visible",
        inset_focus_ring_sx("-2px"),
    )
    // A grid cell's ring, inset: a pinned or clipped cell would cut an outer one.
    .selector(
        "& tbody td[tabindex]:focus-visible, & tbody th[tabindex]:focus-visible, \
         & thead th[tabindex]:focus-visible, & tbody > tr[aria-level]:focus-visible",
        inset_focus_ring_sx("-2px"),
    )
    .selector(ROW_RING_CELLS, row_ring_cell_sx(Sides::NONE))
    .selector(ROW_RING_FIRST, row_ring_cell_sx(Sides::LEFT))
    .selector(ROW_RING_LAST, row_ring_cell_sx(Sides::RIGHT))
    .selector(ROW_RING_ONLY, row_ring_cell_sx(Sides::BOTH))
    .rtl(
        sx().selector(ROW_RING_FIRST, row_ring_cell_sx(Sides::RIGHT))
            .selector(ROW_RING_LAST, row_ring_cell_sx(Sides::LEFT))
            .selector(ROW_RING_ONLY, row_ring_cell_sx(Sides::BOTH)),
    )
    // An editing cell's field fills it, in the cell's own type.
    .selector(
        "& [data-cell-editor]",
        sx().display("flex").align_items("center").gap("2px"),
    )
    .selector(
        "& [data-cell-editor] input",
        sx().flex("1 1 auto")
            .min_width("0")
            .width("100%")
            .padding("0 4px")
            .font("inherit")
            .and(ios_text_floor_sx("1em"))
            .color("inherit")
            .text_align("inherit")
            .background("transparent")
            .border("1px solid currentColor")
            .border_radius("2px"),
    )
    .selector(
        "& [data-cell-editor] input[type='checkbox']",
        sx().flex("none")
            .width("1.25em")
            .height("1.25em")
            .min_width("24px")
            .min_height("24px")
            .margin("0")
            .padding("0")
            .border("revert")
            .background("revert"),
    )
    .selector(
        "& [data-cell-editor] input:focus-visible",
        inset_focus_ring_sx("-2px"),
    )
    .selector(
        "& [data-cell-editor] input[aria-invalid]",
        sx().border_color("error.6"),
    )
    .selector(
        "& [data-cell-error]",
        sx().color("error.6")
            .font_size("0.85em")
            .text_align("start"),
    )
    // A finger has no Escape: only a coarse pointer gets the cancel button.
    .selector(
        "& [data-cell-editor-cancel]",
        sx().display("none")
            .media(
                COARSE,
                sx().display("inline-flex")
                    .align_items("center")
                    .justify_content("center")
                    .flex("none")
                    .width("24px")
                    .height("24px")
                    .padding("0")
                    .border("0")
                    .border_radius("4px")
                    .background("none")
                    .color("inherit")
                    .cursor("pointer")
                    .position("relative"),
            )
            .and(coarse_hit_area_sx("::before")),
    )
    .selector(
        "& [data-cell-editor-cancel] svg",
        sx().width("16px").height("16px"),
    )
    .selector(
        "& [data-cell-editor-cancel]:focus-visible",
        inset_focus_ring_sx("-2px"),
    )
    .selector(
        "& [data-filter-cell]",
        sx().padding_block("4px").font_weight("400"),
    )
    .selector(
        "& [data-column-menu]:focus-visible",
        inset_focus_ring_sx("-2px"),
    )
    // The sticky rules below outrank it: a sticky cell places the grip as well.
    .selector(
        "& th[data-resizable], & th[data-draggable]",
        sx().position("relative"),
    )
    // The header cell's box, measured for a resize's start width, a drag's layout or a pin's room.
    .selector(
        "& [data-resize-box], & [data-drag-box], & [data-pin-box]",
        sx().position("absolute")
            .top("0")
            .bottom("0")
            .with("inset-inline-start", "0")
            .with("inset-inline-end", "0")
            .with("pointer-events", "none"),
    )
    .selector(
        "& [data-resize-handle]",
        sx().position("absolute")
            .top("0")
            .bottom("0")
            .with("inset-inline-end", "0")
            .width("8px")
            // A short line at rest, so the edge reads as draggable (todo 1451).
            .color("muted.6")
            .with(
                "background-image",
                "linear-gradient(currentColor, currentColor)",
            )
            .with("background-repeat", "no-repeat")
            .with("background-size", "2px 50%")
            .with("background-position", "right center")
            .cursor("col-resize")
            .touch_action("none")
            // A finger gets the grip lane; the header's content ends before it (todo 2015).
            .media(COARSE, sx().width(GRIP_LANE)),
    )
    .selector(
        "& th[data-resizable]",
        sx().media(
            COARSE,
            sx().with(
                "padding-inline-end",
                format!("max({GRIP_LANE}, {})", TABLE_PAD_X.value()),
            ),
        ),
    )
    .selector(
        "& th[data-resizable][data-sortable]",
        sx().media(COARSE, sx().with("padding-inline-end", GRIP_LANE)),
    )
    // The menu button's 4px end margin is part of the lane.
    .selector(
        "& th[data-resizable][data-menu]",
        sx().media(
            COARSE,
            sx().with("padding-inline-end", format!("calc({GRIP_LANE} - 4px)")),
        ),
    )
    .selector(
        "& th[data-resizable] [data-sort-button], & th[data-resizable] [data-header-text]",
        sx().media(
            COARSE,
            sx().with(
                "padding-inline-end",
                format!("max(0px, calc({} - {GRIP_LANE}))", TABLE_PAD_X.value()),
            ),
        ),
    )
    .rtl(sx().selector(
        "& [data-resize-handle]",
        sx().with("background-position", "left center"),
    ))
    // `:active` makes Chromium's touch adjustment count the grip as a target (as 1463).
    .selector(
        "& th:hover [data-resize-handle], & [data-resize-handle][data-dragging], \
         & [data-resize-handle]:active, & [data-resize-handle]:focus-visible",
        sx().color("primary.6").with("background-size", "2px 100%"),
    )
    .selector(
        "& [data-resize-handle]:focus-visible",
        inset_focus_ring_sx("-2px"),
    )
    // The grip owns a 24px lane (WCAG 2.5.8): the header's content, its menu
    // button too, starts past it (todo 1483).
    .selector(
        "& th[data-draggable]",
        sx().with(
            "padding-inline-start",
            format!("max({GRIP_LANE}, {})", TABLE_PAD_X.value()),
        ),
    )
    .selector(
        "& th[data-draggable][data-sortable], & th[data-draggable][data-menu]",
        sx().with("padding-inline-start", GRIP_LANE),
    )
    .selector(
        "& th[data-draggable] [data-sort-button], & th[data-draggable] [data-header-text]",
        sx().with(
            "padding-inline-start",
            format!("max(0px, calc({} - {GRIP_LANE}))", TABLE_PAD_X.value()),
        ),
    )
    .selector(
        "& [data-drag-handle]",
        sx().position("absolute")
            .top("0")
            .bottom("0")
            .with("inset-inline-start", "0")
            .width(GRIP_LANE)
            .display("flex")
            .align_items("center")
            .justify_content("center")
            .cursor("grab")
            .touch_action("none")
            .opacity("0.6"),
    )
    .selector(
        "& [data-drag-handle] svg",
        sx().width("12px").height("12px"),
    )
    // An `:active` style makes Chromium's touch adjustment count the grip as a
    // tap target; without one a touch on it snaps to the sort button (1463).
    .selector(
        "& [data-drag-handle][data-dragging], & [data-drag-handle]:active",
        sx().cursor("grabbing"),
    )
    // Fixed: it follows the pointer over the scrolled and pinned cells alike.
    .selector(
        "& [data-drag-ghost]",
        sx().position("fixed")
            .z_index("10")
            .box_sizing("border-box")
            .display("flex")
            .align_items("center")
            .padding(TableDefaults::padding())
            .overflow("hidden")
            .white_space("nowrap")
            .opacity("0.8")
            .background(NamedColorCss::SURFACE.value())
            .border(TableDefaults::border())
            .with("pointer-events", "none"),
    )
    .selector(
        "& [data-drop-line]",
        sx().position("fixed")
            .z_index("10")
            .width("0")
            .with("border-inline-start", "2px solid")
            .border_color("primary.6")
            .with("pointer-events", "none"),
    )
    // With a mouse, shown on its header's hover or focus (todo 1260); a touch
    // screen has no hover. Faded, not hidden, so Tab still reaches it.
    .media(
        "(hover: hover) and (pointer: fine)",
        sx().selector(
            "& th[data-menu]:not(:hover):not(:focus-within) \
             [data-column-menu]:not([aria-expanded=\"true\"])",
            sx().opacity("0"),
        )
        .selector(
            "& th[data-draggable]:not(:hover) [data-drag-handle]",
            sx().opacity("0"),
        ),
    )
    // As wide as its box and padding, spelled out: Blitz sizes a `width: 1px`
    // cell below its content.
    .per_size(|size| {
        sx().selector(
            "& [data-select]",
            sx().box_sizing("border-box").width(select_width(size)),
        )
    })
    // The boxes have no label: no gap for one, which would widen the column
    // past `select_width`, the inset of a start-pinned column. Outranks the
    // field's own `[data-state~=inline]` gap whatever the stylesheet order.
    .selector("& [data-select] > [data-state]", sx().column_gap("0"))
    // A select-all restyles every row's box at once, and a thousand fades doubled
    // its main-thread time (todo 1462): only the box under the pointer or focus fades.
    .selector(
        "& tbody [data-select]:not(:hover, :focus-within) [data-slot=\"box\"], \
         & tbody [data-select]:not(:hover, :focus-within) [data-slot=\"box\"]::before, \
         & tbody [data-select]:not(:hover, :focus-within) [data-slot=\"box\"] > svg",
        sx().transition("none"),
    )
    .selector(
        "& [data-detail-toggle]",
        sx().box_sizing("border-box")
            .width(detail_width())
            .padding_block("0"),
    )
    .selector(
        "& [data-detail-button]",
        sx().display("flex")
            .align_items("center")
            .justify_content("center")
            .width("24px")
            .height("24px")
            .padding("0")
            .border("0")
            .border_radius("4px")
            .background("none")
            .color("inherit")
            .cursor("pointer"),
    )
    .selector(
        "& [data-detail-button]:focus-visible",
        inset_focus_ring_sx("-2px"),
    )
    .selector(
        "& [data-detail-button] svg",
        sx().width("16px").height("16px"),
    )
    // Closed, the chevron points along the reading direction.
    .selector(
        "& [data-detail-button][aria-expanded=\"false\"] svg",
        sx().transform("rotate(-90deg)"),
    )
    .rtl(sx().selector(
        "& [data-detail-button][aria-expanded=\"false\"] svg",
        sx().transform("rotate(90deg)"),
    ))
    // The padding moves inside the `Collapse`, so a shut row has no height left.
    .selector("& tr[data-sliding] > td", sx().padding("0"))
    .selector(
        "& [data-detail-body]",
        sx().padding(TableDefaults::padding()),
    )
    .selector(
        "& [data-reorder]",
        sx().box_sizing("border-box")
            .width(reorder_width())
            .padding_block("0"),
    )
    .selector(
        "& [data-reorder-controls]",
        sx().display("flex").align_items("center"),
    )
    // As `Sortable`'s items: the neighbours slide only while a row drags.
    .selector(
        "& tbody[data-sorting] > tr",
        sx().transition("transform 150ms ease")
            .media("(prefers-reduced-motion: reduce)", sx().transition("none")),
    )
    .selector(
        "& tbody > tr[data-dragging]",
        sx().position("relative")
            .z_index("2")
            .transition("none")
            .background(NamedColorCss::SURFACE.value()),
    )
    // Natively a `tr` has no box: its cells move and lift in its place (todo 1520).
    .selector(
        "& tbody > tr[data-cell-moved] > *",
        sx().transform(format!("var({ROW_TRANSFORM})"))
            .with("animation", format!("var({ROW_ANIMATION}, none)")),
    )
    .selector(
        "& tbody[data-sorting] > tr[data-cell-moved] > *",
        sx().transition("transform 150ms ease")
            .media("(prefers-reduced-motion: reduce)", sx().transition("none")),
    )
    .selector(
        "& tbody > tr[data-dragging][data-cell-moved] > *",
        sx().z_index("2").transition("none").background("inherit"),
    )
    .selector(
        "& tbody > tr[data-dragging][data-cell-moved] > :not([data-pin])",
        sx().position("relative"),
    )
    .when(
        "row-click",
        sx().selector(
            "& tbody tr:not([data-empty]):not([data-detail]):not([data-skeleton]):hover",
            sx().cursor("pointer"),
        ),
    )
    // On each `th`: a sticky `thead` or `tr` has no box natively. Opaque and
    // above the rows, which scroll under it. Group headers scroll away there.
    .when(
        "sticky-header",
        sx().selector(
            "& thead th:not([data-group])",
            sx().position("sticky")
                .top(sticky_top())
                .z_index("1")
                .background(NamedColorCss::SURFACE.value()),
        ),
    )
    // Several header rows can't all stick at the top: the `thead` does.
    .when(
        "sticky-head",
        sx().selector(
            "& thead",
            sx().position("sticky").top(sticky_top()).z_index("1"),
        )
        .selector(
            "& thead th, & thead td",
            sx().background(NamedColorCss::SURFACE.value()),
        ),
    )
    .when(
        "sticky-filters",
        sx().selector(
            "& thead tr[data-filters] > td",
            sx().position("sticky")
                .top(format!(
                    "calc({} + {})",
                    sticky_top(),
                    FILTERS_TOP_VAR.value()
                ))
                .z_index("1")
                .background(NamedColorCss::SURFACE.value()),
        ),
    )
    // A focus scroll in the page stops below the header, not under it (WCAG 2.4.11).
    .when(
        "sticky-page",
        sx().selector(PAGE_FOCUS_TARGETS, {
            let clearance = format!(
                "calc({} + var({}, 0px))",
                sticky_top(),
                HEAD_HEIGHT_VAR.name()
            );
            // Blitz's focus scroll reads the custom property on every side: zero below.
            sx().with("scroll-margin-top", clearance.clone())
                .with(SCROLL_MARGIN_VAR, clearance)
                .with(SCROLL_MARGIN_BOTTOM_VAR, "0px")
        }),
    )
    // On each `td`, like the header's `th`: the rows scroll under it.
    .when(
        "sticky-footer",
        sx().selector(
            "& tfoot td",
            sx().position("sticky")
                .bottom("0")
                .z_index("1")
                .border_top(TableDefaults::border()),
        )
        // Above the other footer cells scrolling under it, as a pinned header cell is.
        .selector(
            "& tfoot td[data-pin], & tfoot td[data-footer-lead]",
            sx().z_index("2"),
        ),
    )
    // Opaque over the scrolled cells: a body cell takes its row's colour, which
    // is the surface unless hovered, striped or selected.
    .when(
        "pinned",
        sx().selector(
            "& tbody tr",
            sx().background(NamedColorCss::SURFACE.value()),
        )
        .selector(
            "& [data-pin]",
            sx().position("sticky").z_index("1").background("inherit"),
        )
        // Above the body cells scrolling under a sticky header, or a sticky `thead`.
        .selector("& thead", sx().z_index("2"))
        .selector("& thead th, & thead td", sx().z_index("2"))
        .selector(
            "& thead th[data-pin], & thead td[data-pin]",
            sx().z_index("3").background(NamedColorCss::SURFACE.value()),
        )
        .selector(
            "& tfoot td[data-pin]",
            sx().background(NamedColorCss::SURFACE.value()),
        )
        .selector(
            "& [data-pin=\"start\"][data-pin-edge]",
            sx().with("border-inline-end", TableDefaults::border()),
        )
        .selector(
            "& [data-pin=\"end\"][data-pin-edge]",
            sx().with("border-inline-start", TableDefaults::border()),
        ),
    )
    // After the hover and the pinned rows' surface, which it outranks by order alone.
    .selector(
        "& tbody tr[data-row-group]",
        sx().background(TABLE_GROUP.value()).font_weight("600"),
    )
    .selector(
        "& [data-group-toggle]",
        sx().display("inline-flex")
            .align_items("center")
            .gap("4px")
            .padding("0")
            .border("0")
            .border_radius("4px")
            .background("none")
            .font("inherit")
            .letter_spacing("inherit")
            .color("inherit")
            .white_space("nowrap")
            .cursor("pointer"),
    )
    .selector(
        "& [data-group-toggle]:focus-visible",
        inset_focus_ring_sx("-2px"),
    )
    .selector(
        "& [data-group-toggle] svg",
        sx().width("16px").height("16px").flex_shrink("0"),
    )
    .selector(
        "& [data-group-toggle][aria-expanded=\"false\"] svg",
        sx().transform("rotate(-90deg)"),
    )
    .rtl(sx().selector(
        "& [data-group-toggle][aria-expanded=\"false\"] svg",
        sx().transform("rotate(90deg)"),
    ))
    // A tree row's toggle, or a leaf's spacer as wide, before its first cell's text.
    .selector(
        "& [data-node-toggle]",
        sx().display("inline-flex")
            .with("vertical-align", "middle")
            .with("margin-inline-end", "4px")
            .padding("0")
            .border("0")
            .border_radius("4px")
            .background("none")
            .color("inherit")
            .cursor("pointer")
            // The 16px chevron takes presses in a 24px box (WCAG 2.5.8).
            .position("relative")
            .and(min_hit_area_sx("::before")),
    )
    .selector(
        "& [data-node-toggle]:focus-visible",
        inset_focus_ring_sx("-2px"),
    )
    .selector(
        "& [data-node-spacer]",
        sx().display("inline-block")
            .width("16px")
            .with("margin-inline-end", "4px"),
    )
    .selector(
        "& [data-node-toggle] svg",
        sx().width("16px").height("16px").flex_shrink("0"),
    )
    .selector(
        "& [data-node-toggle][aria-expanded=\"false\"] svg",
        sx().transform("rotate(-90deg)"),
    )
    .rtl(sx().selector(
        "& [data-node-toggle][aria-expanded=\"false\"] svg",
        sx().transform("rotate(90deg)"),
    ))
    .selector("& [data-group-count]", sx().font_weight("400"))
    // Apart from the toggle when the toggle's column aggregates too.
    .selector(
        "& [data-group-toggle] + [data-group-aggregate]",
        match spaces_inline_boxes() {
            true => sx(),
            // Blitz drops an inline box's side margin.
            false => sx().display("inline-block"),
        }
        .with("margin-inline-start", "12px"),
    )
    // The footer's and the group rows' one cell over the lead columns holds with them.
    .when(
        "pin-footer-lead",
        sx().selector(
            "& tfoot td[data-footer-lead]",
            sx().position("sticky")
                .with("inset-inline-start", "0")
                .z_index("1"),
        )
        .selector(
            "& tbody td[data-group-lead]",
            sx().position("sticky")
                .with("inset-inline-start", "0")
                .z_index("1")
                .background("inherit"),
        ),
    )
    .when(
        "pin-select",
        sx().selector(
            "& [data-select]",
            sx().position("sticky")
                .with("inset-inline-start", "0")
                .z_index("1")
                .background("inherit"),
        )
        .selector(
            "& thead th[data-select], & thead td[data-select]",
            sx().z_index("3").background(NamedColorCss::SURFACE.value()),
        ),
    )
    // The toggles lead, before the checkboxes, which they push inwards.
    .when(
        "pin-detail",
        sx().selector(
            "& [data-detail-toggle]",
            sx().position("sticky")
                .with("inset-inline-start", "0")
                .z_index("1")
                .background("inherit"),
        )
        .selector(
            "& thead th[data-detail-toggle], & thead td[data-detail-toggle]",
            sx().z_index("3").background(NamedColorCss::SURFACE.value()),
        )
        .selector(
            "& [data-select]",
            sx().with("inset-inline-start", detail_width()),
        ),
    )
    // The handles lead all: the toggles and checkboxes move in past them.
    .when(
        "pin-reorder",
        sx().selector(
            "& [data-reorder]",
            sx().position("sticky")
                .with("inset-inline-start", "0")
                .z_index("1")
                .background("inherit"),
        )
        .selector(
            "& thead th[data-reorder], & thead td[data-reorder]",
            sx().z_index("3").background(NamedColorCss::SURFACE.value()),
        )
        .selector(
            "& [data-detail-toggle], & [data-select]",
            sx().with("inset-inline-start", reorder_width()),
        ),
    )
    .when(
        "pin-reorder-detail",
        sx().selector(
            "& [data-select]",
            sx().with(
                "inset-inline-start",
                format!("calc({} + {})", reorder_width(), detail_width()),
            ),
        ),
    )
    .when("windowed", windowed_sx())
});

/// The reorder column's width: a handle and two move buttons, 24px each, and the cell padding.
fn reorder_width() -> String {
    format!("calc(72px + 2 * {})", TABLE_PAD_X.value())
}

/// The detail toggles' column width: a 24px button and the cell padding.
fn detail_width() -> String {
    format!("calc(24px + 2 * {})", TABLE_PAD_X.value())
}

/// The checkbox column's width, which a start-pinned column is inset by.
fn select_width(size: Size) -> String {
    format!(
        "calc({} + 2 * {})",
        CHECKBOX_BOX_SIZE.value(size),
        TABLE_PAD_X.value()
    )
}

fn caption_sx() -> Sx {
    sx().text_align_start()
        .font_weight("600")
        .padding(TableDefaults::padding())
}

/// The caption drawn before the table where Blitz skips `<caption>`, with the
/// font size and padding the web caption inherits from the table.
static TABLE_CAPTION_SX: StaticSx = StaticSx::new(|| caption_sx().per_size(TableDefaults::size_sx));

/// A flex parent's item at its width, not the table's: the toolbar wraps there and a
/// `scroll` table scrolls inside it (todo 2084).
static TABLE_ROOT_SX: StaticSx = StaticSx::new(|| sx().min_width("0"));

/// As tall as the table, up to `max_height`; not the parent's height.
static TABLE_SCROLL_SX: StaticSx =
    StaticSx::new(|| scroll_area_base(sx().height("auto").max_width("100%")));

#[derive(Props, Clone, PartialEq)]
pub struct TableProps<T: Clone + PartialEq + 'static> {
    /// One row each, in source order until a column is sorted.
    data: Vec<T>,
    /// Built with [`column`](fn@super::column).
    columns: Vec<Column<T>>,
    /// What every column starts from; a column's own setting wins.
    #[props(default)]
    column_defaults: ColumnDefaults,
    /// A visible title above the header row, and the table's accessible name.
    #[props(default, into)]
    caption: Option<String>,
    /// Shown in one full-width row when `data` is empty.
    #[props(default)]
    empty: Option<Element>,
    /// Shown in one full-width row when the filters leave no rows.
    #[props(default)]
    no_results: Option<Element>,
    /// Rows are on their way: placeholder rows while none show, else a
    /// progress bar over the table's top edge.
    #[props(default)]
    loading: bool,
    /// A row above the table, for your controls; the quick filter joins it at
    /// the end. `TableColumnsButton`, `TableDensityButton` and `TableExportButton`
    /// work only in here.
    #[props(default)]
    toolbar: Option<Element>,
    /// The `TableExportButton` file gives a cell starting with `=`, `+`, `-`, `@`, Tab or CR
    /// a leading `'`, so a spreadsheet reads it as text (as [`table_csv`](fn@super::table_csv)).
    /// `false` exports cells as they are.
    #[props(default = true)]
    csv_formula_guard: bool,
    /// Scrolls a table wider than its parent sideways, in a `ScrollArea`.
    #[props(default)]
    scroll: bool,
    /// Caps the height, a CSS length: the rows scroll in a `ScrollArea` (both
    /// axes) under a header that stays put.
    #[props(default, into)]
    max_height: Option<String>,
    /// Keeps the header in view while the page, or the scrolling container around the table,
    /// scrolls past the rows, this far below its top edge: a CSS length, say `"0"` or a fixed
    /// app header's height. It sticks to the nearest ancestor with an `overflow` other than
    /// `visible`, even one that only scrolls sideways. No effect with `max_height`, whose header already stays put, or with
    /// `scroll`, whose area holds it instead of the page; a debug build warns.
    #[props(default, into)]
    sticky_header: Option<String>,
    /// With `max_height`, renders only the rows in view, each clipped to this
    /// height in px with one line per cell, group rows too. Off with `row_detail`.
    #[props(default)]
    virtual_row_height: Option<f64>,
    /// With `max_height`, called when the rows scroll to their bottom, to append
    /// the next batch. Not while `loading`.
    #[props(default)]
    onbottomreached: Option<EventHandler<()>>,
    /// The sorted column, empty for source order; set, the sort is controlled.
    /// Only the first entry naming a sortable header applies.
    #[props(default)]
    sort: Option<Vec<TableSort>>,
    /// Seeds the sort once. Ignored when `sort` is set.
    #[props(default)]
    default_sort: Vec<TableSort>,
    /// The sort a header click asks for: ascending, descending, then empty.
    #[props(default)]
    onsortchange: Option<EventHandler<Vec<TableSort>>>,
    /// Sorts by several columns: Shift, Ctrl or Cmd click, or any tap on a touch
    /// screen, adds a column after the sorted ones; a plain click sorts by it alone.
    #[props(default)]
    multi_sort: bool,
    /// Adds a checkbox column, with a select-all box in its header.
    #[props(default)]
    selectable: bool,
    /// The selected rows' `row_key`s; set, the selection is controlled. Keys of rows that
    /// leave `data` stay, so it can span a server's pages: drop them when deleting rows.
    #[props(default)]
    selection: Option<Vec<String>>,
    /// Seeds the selection once. Ignored when `selection` is set.
    #[props(default)]
    default_selection: Vec<String>,
    /// The selection a checkbox asks for. Select-all keeps keys of rows not in `data`.
    #[props(default)]
    onselectionchange: Option<EventHandler<Vec<String>>>,
    /// A row's identity, unique per row: its DOM node follows it through a sort
    /// or a data change. Unset, the row's index in `data`.
    #[props(default, into)]
    row_key: RowFn<T, String>,
    /// Called with the clicked row. Pointer only: give keyboard users a button
    /// or link in a cell for the same action.
    #[props(default)]
    onrowclick: Option<EventHandler<T>>,
    /// A row's states, as its `data-state`, for `sx().selector("& tbody tr", sx().when(..))`.
    #[props(default, into)]
    row_states: RowFn<T, States>,
    /// Extra attributes on a row's `tr`.
    #[props(default, into)]
    row_attrs: RowFn<T, Vec<Attribute>>,
    /// A row's detail, shown in a full-width row under it: `Some` gives the row
    /// a toggle in a leading column. Called for the shown rows, or only the
    /// open ones with `row_has_detail`.
    #[props(default, into)]
    row_detail: RowFn<T, Option<Element>>,
    /// Whether a row has a detail, without building it: set, it decides the
    /// toggles and `row_detail` is called for the open rows only. For large tables.
    #[props(default, into)]
    row_has_detail: RowFn<T, bool>,
    /// The `row_key`s of the rows whose detail shows; set, it is controlled.
    #[props(default)]
    expanded: Option<Vec<String>>,
    /// Seeds the open details once. Ignored when `expanded` is set.
    #[props(default)]
    default_expanded: Vec<String>,
    /// The open details a toggle asks for.
    #[props(default)]
    onexpandedchange: Option<EventHandler<Vec<String>>>,
    /// Slides the detail rows open and shut, as a `Collapse`; at once under
    /// reduced motion. Not with `onrowreorder`.
    #[props(default)]
    animate_details: bool,
    /// Adds a leading column of drag handles and move buttons; called with a
    /// move by positions in `data`, which you apply (`step.apply(&mut rows)`).
    /// Off while the rows are sorted or filtered. Not dragged on Blitz.
    #[props(default)]
    onrowreorder: Option<EventHandler<SortableMove>>,
    /// Called with an edited clone of a row when an
    /// [`editable`](Column::editable) cell commits; store it, `Table` keeps no copy.
    /// An editable column turns `cell_focus` on.
    #[props(default)]
    onrowupdate: Option<EventHandler<RowUpdate<T>>>,
    /// The headers whose cell texts group the rows, outermost first; set, the
    /// grouping is controlled. Each group gets a row with a toggle, groups follow
    /// the sorted rows and empty cells form one group. With `selectable`, a group's
    /// box selects its rows. Rows don't reorder while grouped.
    #[props(default)]
    row_grouping: Option<Vec<String>>,
    /// Seeds the grouping once. Ignored when `row_grouping` is set.
    #[props(default)]
    default_row_grouping: Vec<String>,
    /// The grouping a pick asks for.
    #[props(default)]
    onrowgroupingchange: Option<EventHandler<Vec<String>>>,
    /// The open groups' ids, their texts from the outermost in, joined by `/`
    /// (`"Sales/Berlin"`, a `/` or `\` in a text escaped by `\`); set, it is controlled.
    #[props(default)]
    expanded_groups: Option<Vec<String>>,
    /// Seeds the open groups once. Ignored when `expanded_groups` is set.
    #[props(default)]
    default_expanded_groups: Vec<String>,
    /// The open groups a group toggle asks for.
    #[props(default)]
    onexpandedgroupschange: Option<EventHandler<Vec<String>>>,
    /// How many levels of groups open the first time rows are grouped, beside
    /// `default_expanded_groups`; 0 starts them all collapsed. Not with `expanded_groups`.
    #[props(default)]
    default_group_expansion: usize,
    /// The `row_key` of a row's parent: the rows nest under their parents, each
    /// level in the sort, and a missing parent makes a root. Needs `row_key`. Over
    /// `row_grouping`; the rows neither reorder nor show a `row_detail`. Makes the
    /// table a `role="treegrid"` whose rows the arrows move, open and close. With
    /// `selectable`, a row's box selects its descendants, mixed with some selected.
    #[props(default, into)]
    parent_key: RowFn<T, Option<String>>,
    /// Whether a tree row has children not in `data` yet: it gets a toggle, and
    /// opening it asks via `onexpandednodeschange`. Append its children to `data`,
    /// or return `false` once it has none: the open row stays busy until then.
    #[props(default, into)]
    row_has_children: RowFn<T, bool>,
    /// The `row_key`s of the open tree rows; set, they are controlled.
    #[props(default)]
    expanded_nodes: Option<Vec<String>>,
    /// Seeds the open tree rows once. Ignored when `expanded_nodes` is set.
    #[props(default)]
    default_expanded_nodes: Vec<String>,
    /// The open tree rows a toggle asks for.
    #[props(default)]
    onexpandednodeschange: Option<EventHandler<Vec<String>>>,
    /// How many tree levels open the first time there are rows, beside
    /// `default_expanded_nodes`; 0 starts them all collapsed. Not with `expanded_nodes`.
    #[props(default)]
    default_node_expansion: usize,
    /// Makes the table a `role="grid"` with one header or body cell as its tab stop,
    /// the first header cell at first: the arrows, Home, End, Ctrl+Home, Ctrl+End,
    /// PageUp and PageDown move it. Enter or Space on a header cell sorts it,
    /// Ctrl+Enter opens its column menu, Shift+Space selects a body cell's row.
    #[props(default)]
    cell_focus: bool,
    /// Cell padding and font size.
    #[props(default, into)]
    size: Input<Size>,
    /// The size a `TableDensityButton` picked, over `size`; set, it is controlled.
    #[props(default)]
    density: Option<Size>,
    /// Seeds the density once. Ignored when `density` is set.
    #[props(default)]
    default_density: Option<Size>,
    /// The density a `TableDensityButton` pick asks for.
    #[props(default)]
    ondensitychange: Option<EventHandler<Size>>,
    /// Shades every other body row.
    #[props(default)]
    striped: bool,
    /// The shown page, 1-based; set, the page is controlled. Clamped into range.
    #[props(default)]
    page: Option<u32>,
    /// Seeds the page once. Ignored when `page` is set.
    #[props(default = 1)]
    default_page: u32,
    /// The page a page button, a sort or a page-size pick asks for.
    #[props(default)]
    onpagechange: Option<EventHandler<u32>>,
    /// Rows per page; set, the page size is controlled. Turns on pagination.
    #[props(default)]
    page_size: Option<usize>,
    /// Seeds the page size once. Turns on pagination.
    #[props(default)]
    default_page_size: Option<usize>,
    /// The size a pick in the page-size picker asks for.
    #[props(default)]
    onpagesizechange: Option<EventHandler<usize>>,
    /// The page-size picker's choices; empty hides it. Turns on pagination,
    /// the first one seeding the size.
    #[props(default)]
    page_sizes: Vec<usize>,
    /// `data` comes sorted: a header click only asks via `onsortchange`.
    #[props(default)]
    manual_sort: bool,
    /// `data` is the current page: the table only draws the page controls. A
    /// column's `aggregate` then covers the loaded rows only.
    #[props(default)]
    manual_pagination: bool,
    /// Rows over all pages with `manual_pagination`. Unset, `data`'s length.
    #[props(default)]
    row_count: Option<usize>,
    /// The hidden columns' headers; set, visibility is controlled. A hidden
    /// sorted column keeps sorting.
    #[props(default)]
    hidden_columns: Option<Vec<String>>,
    /// Seeds the hidden columns once. Ignored when `hidden_columns` is set.
    #[props(default)]
    default_hidden_columns: Vec<String>,
    /// The hidden columns a column menu pick asks for.
    #[props(default)]
    onhiddencolumnschange: Option<EventHandler<Vec<String>>>,
    /// The columns held at the table's start and end edges while the rest
    /// scroll sideways; set, pinning is controlled. Pair with `scroll` or
    /// `max_height`, and give each pinned column but the outermost a `width`.
    #[props(default)]
    pinned_columns: Option<PinnedColumns>,
    /// Seeds the pinned columns once. Ignored when `pinned_columns` is set.
    #[props(default)]
    default_pinned_columns: PinnedColumns,
    /// The pinned columns a column menu pick asks for.
    #[props(default)]
    onpinnedcolumnschange: Option<EventHandler<PinnedColumns>>,
    /// The headers in display order; set, the order is controlled. Unlisted
    /// columns follow the listed ones in `columns` order. Pinned columns keep
    /// their pinned order.
    #[props(default)]
    column_order: Option<Vec<String>>,
    /// Seeds the column order once. Ignored when `column_order` is set.
    #[props(default)]
    default_column_order: Vec<String>,
    /// The order a column menu's Move left or Move right, or a header drag,
    /// asks for, every header listed.
    #[props(default)]
    oncolumnorderchange: Option<EventHandler<Vec<String>>>,
    /// A drag grip on each header's end edge, and Widen, Narrow and Reset width
    /// in the `column_menu`, its keyboard and drag-free way. Off per column
    /// with `Column::resizable(false)`.
    #[props(default)]
    resizable_columns: bool,
    /// Resized widths in px by header, over the columns' own `width`; set,
    /// they are controlled.
    #[props(default)]
    column_widths: Option<ColumnWidths>,
    /// Seeds the resized widths once. Ignored when `column_widths` is set.
    #[props(default)]
    default_column_widths: ColumnWidths,
    /// The widths a drag's end or a column menu pick asks for.
    #[props(default)]
    oncolumnwidthschange: Option<EventHandler<ColumnWidths>>,
    /// A menu button in each header: sort, hide the column, show or hide others.
    /// Off Blitz, unpinned headers also get a pointer-only drag grip to move them.
    #[props(default)]
    column_menu: bool,
    /// The column menus' `parts`: they open in a portal, out of `sx`'s reach.
    #[props(default, into)]
    column_menu_parts: Input<Parts<MenuPart>>,
    /// The quick filter's text; set, it is controlled. A row stays when every
    /// word occurs in one of its shown, `filterable` cells, ignoring case.
    #[props(default, into)]
    quick_filter: Option<String>,
    /// Seeds the quick filter once. Ignored when `quick_filter` is set.
    #[props(default, into)]
    default_quick_filter: String,
    /// The text typed into the quick-filter field.
    #[props(default)]
    onquickfilterchange: Option<EventHandler<String>>,
    /// A search field above the table that drives `quick_filter`.
    #[props(default)]
    show_quick_filter: bool,
    /// `data` comes filtered: the quick filter and the column filters only ask
    /// via their change handlers.
    #[props(default)]
    manual_filter: bool,
    /// The column filters, one per column, all applying; set, they are
    /// controlled. A hidden column's filter keeps filtering.
    #[props(default)]
    column_filters: Option<Vec<ColumnFilter>>,
    /// Seeds the column filters once. Ignored when `column_filters` is set.
    #[props(default)]
    default_column_filters: Vec<ColumnFilter>,
    /// The column filters a filter popover or a header filter asks for.
    #[props(default)]
    oncolumnfilterschange: Option<EventHandler<Vec<ColumnFilter>>>,
    /// How the column filters join: a row passes all of them, or any one.
    /// Set, it is controlled. A column may hold several filters.
    #[props(default)]
    filter_logic: Option<FilterLogic>,
    /// Seeds the filter logic once. Ignored when `filter_logic` is set.
    #[props(default)]
    default_filter_logic: FilterLogic,
    /// The filter logic a pick asks for.
    #[props(default)]
    onfilterlogicchange: Option<EventHandler<FilterLogic>>,
    /// A row of filter fields under the headers, one per `filterable` column.
    #[props(default)]
    header_filters: bool,
    /// A dialog of every column filter, several per column, joined by
    /// `filter_logic`, opened by a Filters button at the toolbar's start, or
    /// with a `toolbar`, by its `TableFilterButton`. `column_menu`'s Filter opens it too.
    #[props(default)]
    filter_panel: bool,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default, into)]
    class: Input<ClassList>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
}

/// A sortable data table over rows of `T`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Table, column};
/// # #[derive(Clone, PartialEq)] struct User { name: String, age: u32 }
/// # fn app() -> Element {
/// let users = use_signal(Vec::<User>::new);
/// rsx! {
///     Table {
///         caption: "Users",
///         empty: rsx! { "No users yet." },
///         data: users(),
///         columns: vec![
///             column("Name").value(|u: &User| u.name.clone()).sortable().row_header(),
///             column("Age").value(|u: &User| u.age).sortable(),
///         ],
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/table>
// Generic shim: only the projection below compiles per `T`.
#[component]
pub fn Table<T: Clone + PartialEq + 'static>(mut props: TableProps<T>) -> Element {
    let tree = props.parent_key.is_set();
    // Before the config takes the seeded state props out.
    use_hook(|| warnings::warn_misuse(&props));
    let state = use_table(config::table_config(&mut props));
    let panel = use_panel_control();
    let announcer = use_announcer();
    let tools = use_context_provider(|| {
        TableTools::new(state.hidden_columns, state.density, panel, announcer)
    });
    let preview = use_signal(|| None);
    let measured = use_hook(|| CopyValue::new(BTreeMap::new()));
    let column_drag = use_column_drag(state.column_order, announcer);
    let menu_focus = use_menu_focus();
    let pin_news = use_pin_news();
    let drags = props.column_menu;
    let touch = use_hook(|| CopyValue::new(false));
    let instructions_id = use_id();
    // The shown rows, which a reorder's slots map to.
    let mut shown_rows = use_hook(|| CopyValue::new(Vec::<VisibleRow>::new()));
    let onrowreorder = props.onrowreorder;
    let row_focus = use_row_focus(column_drag.table);
    let grid_focus = use_grid_focus(column_drag.table, column_drag.region);
    let reorder_rows = use_callback(move |step: SortableMove| {
        let shown = shown_rows.peek();
        let data_at = |slot: usize| shown.get(slot).and_then(|row| row.data());
        if let (Some(onrowreorder), Some(from), Some(to)) =
            (onrowreorder, data_at(step.from), data_at(step.to))
        {
            let step = SortableMove { from, to };
            onrowreorder.call(step);
            row_focus.follow(step);
        }
    });
    let detail_id = use_id();
    let paginated = props.page_size.is_some()
        || props.default_page_size.is_some()
        || !props.page_sizes.is_empty();
    use_page_reset(state);
    use_removed_columns(
        state,
        props
            .columns
            .iter()
            .map(|column| column.header.clone())
            .collect(),
    );
    use_page_size_reseed(state, &props.page_sizes, props.default_page_size);
    let caption_id = use_id();
    use_name_warning(
        props.caption.is_some() || names_itself(&props.attributes),
        "Table: no `caption`, `aria_label` or `aria-labelledby`, so it is announced without a \
         name.",
    );
    // A picked density wins over `size`.
    let size = state
        .density
        .read()
        .unwrap_or(props.size.copied_or(use_theme().table.size));
    let localization = use_localization();
    let (labels, lang) = (localization.table, localization.lang);
    let user_sort = UserSort::new(state.sort, announcer, labels);
    let resize = props.resizable_columns.then_some(ColumnResize {
        widths: state.column_widths,
        preview,
        measured,
        labels,
    });
    // A column filter change announces the rows left once it settles (WCAG 4.1.3).
    let mut filtered_count = use_hook(|| CopyValue::new(0usize));
    let onfilter = use_debounced_callback(
        move |()| announcer.say((labels.results)(*filtered_count.peek())),
        500,
    );
    let filter = FilterBase {
        state,
        labels,
        size,
        onfilter,
    };
    let bounded = props.max_height.is_some();
    // Under the page's scroll the header sticks too, if no box holds it.
    let page_sticky = props.sticky_header.is_some() && !bounded && !props.scroll;
    let sticks = bounded || page_sticky;
    let sticky_offset = props
        .sticky_header
        .as_deref()
        .filter(|_| page_sticky)
        .map(sticky_length);
    let (head_element, mut head) = use_edge_height(sticks);
    let (foot_element, mut foot) = use_edge_height(bounded);
    let lift = use_signal(|| None::<(usize, usize)>);
    // Blitz sticks each column header, not the `thead`: the filters row sticks under them by hand (todo 1402).
    let sticky_filters = sticks && props.header_filters && !sticks_table_heads();
    let head_cell = use_element();
    let mut head_cell_height = use_signal(|| 0.0);
    use_resize_fallback(head_cell, move |event| {
        if let (true, Ok(size)) = (sticky_filters, event.get_border_box_size())
            && *head_cell_height.peek() != size.height
        {
            head_cell_height.set(size.height);
        }
    });
    let scrolls = props.scroll || bounded;
    let size_states: Input<States> = States::new().with(size.state_name(), true).into();
    let caption_box = use_box()
        .framework_sx(&TABLE_CAPTION_SX)
        .states(&size_states)
        .prepare();
    let root_box = use_box().framework_sx(&TABLE_ROOT_SX).prepare();
    let look = use_checkbox_look(size, props.selectable);
    let caches = visible_rows::use_row_caches::<T>();
    let mut row_context = use_hook(|| CopyValue::new((0u64, None::<Rc<RowContext<T>>>)));
    let mut pin_edges = use_signal(|| [None::<f64>; 2]);
    let pin_edge = use_callback(move |(side, width): (PinSide, f64)| {
        let at = usize::from(side == PinSide::End);
        if pin_edges.peek()[at] != Some(width) {
            pin_edges.write()[at] = Some(width);
        }
    });
    let mut pin_widths = use_signal(HashMap::<String, f64>::new);
    let pin_width = use_callback(move |(header, width): (String, f64)| {
        if pin_widths.peek().get(&header) != Some(&width) {
            pin_widths.write().insert(header, width);
        }
    });
    let area_width = use_signal(|| None::<f64>);
    let lead_px = use_signal(|| 0.0);

    let has_detail = props.row_detail.is_set() && !tree;
    let has_reorder = props.onrowreorder.is_some();
    let warned = use_hook(|| CopyValue::new(Vec::<String>::new()));
    let header = header_layout(
        &props,
        state,
        resize.and_then(|_| preview()),
        size,
        (has_detail, has_reorder),
        Measures {
            area_width,
            lead_px,
            pin_widths,
            pin_edges,
            warned,
            pin_news,
        },
        drags.then_some(column_drag),
    );
    if let Some((column, side, held)) = header.pin_said.clone() {
        let said = match (held, side) {
            (false, _) => labels.column_pin_released,
            (true, PinSide::Start) => labels.column_pinned_start,
            (true, PinSide::End) => labels.column_pinned_end,
        };
        // After this render: a write while it runs would warn.
        spawn(async move { announcer.say(said(&column)) });
    }
    let sortable = header.headers.iter().any(|spec| spec.sortable);
    let menu_columns: Rc<[MenuColumn]> = header
        .headers
        .iter()
        .zip(&props.columns)
        .map(|(spec, column)| MenuColumn {
            header: spec.header.clone(),
            key: spec.key.clone(),
            sortable: spec.sortable,
            hideable: spec.hideable,
            hidden: spec.hidden,
            groupable: column.row_groupable && !tree,
        })
        .collect();
    let editable = props.columns.iter().any(|column| column.edit.is_some());
    let cell_focus = props.cell_focus || editable;
    let formats = use_formats();
    let tests = cell_tests(&props.columns, &state.column_filters.read(), formats);
    let active_filters = tests.len();
    if props.toolbar.is_some() {
        tools.show(ToolView {
            columns: menu_columns.clone(),
            size,
            filters: props.filter_panel.then_some(active_filters),
        });
    }
    let menus = match props.column_menu {
        true => column_menus(
            &props,
            &header,
            MenuSpec {
                columns: menu_columns,
                resize,
                filter,
                panel,
                focus: menu_focus,
                announcer,
                sort: user_sort,
                news: pin_news,
                grid: cell_focus,
            },
        ),
        false => Vec::new(),
    };
    let filter_cells = props
        .header_filters
        .then(|| layout::filter_cells(&props.columns, filter));
    let HeaderLayout {
        headers,
        layout,
        lead_width,
        tracks_area,
        pins_start,
        pins_select,
        pins,
        pinned_pads,
        lead_probe,
        active,
        group_rows,
        head_rows,
        ..
    } = header;

    let columns = Rc::new(std::mem::take(&mut props.columns));
    let kept_rows = keep_rows(
        &mut props,
        caches,
        state,
        &columns,
        |index| headers[index].hidden,
        &tests,
    );
    let OrderedRows {
        mut order,
        results,
        footer,
        grouped,
        groups,
        tree_levels,
    } = order_rows(
        &props,
        caches,
        state,
        &kept_rows,
        &columns,
        &active,
        (&layout, labels, lang),
        tools,
    );
    filtered_count.set(results);
    let KeptRows {
        data,
        keys,
        words,
        kept,
        parents,
        ..
    } = kept_rows;
    let selection = look.map(|look| Selection {
        slice: state.selection,
        scope: selection_scope(&keys, kept.as_deref().map(Vec::as_slice)),
        announcer,
        labels,
        look: Rc::new(look),
        tree: parents.as_ref().map(|parents| {
            let scope = kept.as_deref().map(Vec::as_slice);
            Rc::new(TreeSelect::new(keys.clone(), parents.clone(), scope))
        }),
    });
    // Stable, so a row's box memoizes when its row did not change.
    let current = selection.clone();
    let toggle = use_callback(move |(key, on): (String, bool)| {
        if let Some(selection) = &current {
            selection.toggle(&key, on);
        }
    });
    let selected_keys = match props.selectable {
        true => state.selection.read(),
        false => Vec::new(),
    };
    let selected: HashSet<&str> = selected_keys.iter().map(String::as_str).collect();
    let details = has_detail.then(|| Details {
        slice: state.expanded,
        labels,
        id: detail_id(),
    });
    let current = details.clone();
    let toggle_detail = use_callback(move |(key, open): (String, bool)| {
        if let Some(details) = &current {
            details.toggle(&key, open);
        }
    });
    let expanded_keys = match &details {
        Some(_) => state.expanded.read(),
        None => Vec::new(),
    };
    let expanded: Rc<HashSet<String>> = Rc::new(expanded_keys.iter().cloned().collect());
    let selected_rows: Rc<HashSet<String>> = Rc::new(selected_keys.iter().cloned().collect());
    // A tree row's box is mixed with some of its subtree selected.
    let mixed: Option<Rc<[bool]>> = selection
        .as_ref()
        .and_then(|selection| selection.tree.as_ref())
        .map(|tree| tree.mixed(|row| selected_rows.contains(&keys[row])).into());
    // A row's checkbox and toggle are named by its row header, else its first cell.
    let name_column = columns
        .iter()
        .position(|column| column.row_header)
        .unwrap_or(0);
    let mut aggregates = caches.aggregates;
    let group_values = aggregates.write().values(&data, &columns, &groups);
    let current = (selection.clone(), groups.clone(), keys.clone());
    let select_group = use_callback(move |(id, on): (String, bool)| {
        let (selection, groups, keys) = &current;
        if let (Some(selection), Some(group)) =
            (selection, groups.iter().find(|group| group.id == id))
        {
            let rows: Vec<String> = group.leaves.iter().map(|&at| keys[at].clone()).collect();
            selection.toggle_rows(&rows, on);
        }
    });
    // The count when more rows were asked for: their arrival is said once loading ends.
    let mut asked_at = use_hook(|| CopyValue::new(None::<usize>));
    let loading = props.loading;
    use_effect(use_reactive!(|(results, loading)| {
        if loading {
            return;
        }
        if let Some(before) = asked_at.take()
            && results > before
        {
            announcer.say((labels.results)(results));
        }
    }));
    let onbottomreached = props.onbottomreached;
    let bottom_reached = use_callback(move |()| {
        if let Some(handler) = onbottomreached.filter(|_| !loading) {
            asked_at.set(Some(results));
            handler.call(());
        }
    });
    let (pager, counted) = page_window(&mut props, state, paginated, size, &mut order);
    if has_reorder {
        shown_rows.set(order.clone());
    }
    let skeleton = (props.loading && order.is_empty()).then(|| match paginated {
        true => state.page_size.read().max(1),
        false => SKELETON_ROWS,
    });
    // Not while loading: a count of no rows yet would read as a total.
    let footer = footer.filter(|_| skeleton.is_none());
    let foot_gap = if footer.is_some() { *foot.read() } else { 0.0 };
    let key_at: Rc<dyn Fn(usize) -> String> = {
        let keys = keys.clone();
        Rc::new(move |index| keys[index].clone())
    };
    let has_data = !data.is_empty();
    let headers = Rc::new(headers);
    // Detail rows have their own heights; group and tree rows take the pitch.
    let row_height = props.virtual_row_height.filter(|_| bounded && !has_detail);
    let fixed_slots = row_height.filter(|_| has_reorder).map(|pitch| FixedSlots {
        count: order.len(),
        pitch,
        lift,
        scroller: column_drag.region,
        head: *head.peek(),
        foot: foot_gap,
    });
    let reorder = has_reorder.then(|| RowReorder {
        onreorder: reorder_rows,
        // The shown order is not `data`'s: a move between slots means nothing there.
        disabled: !active.is_empty() || !words.is_empty() || !tests.is_empty() || grouped || tree,
        announcer,
        labels,
        instructions: instructions_id(),
        fixed: fixed_slots,
    });
    let lead_columns = lead_count((has_reorder, has_detail, selection.is_some()));
    // Only with a gap: else each cell's place is its index.
    let colindex = (cell_focus && headers.iter().any(|spec| spec.hidden))
        .then(|| col_indices(&layout, headers.len(), lead_columns));
    let nodes_slice = state.expanded_nodes;
    let toggle_node = use_callback(move |(key, name, open): (String, String, bool)| {
        nodes_slice.set_then(toggle_row(&nodes_slice.peek(), &key, open), move || {
            announcer.say(match open {
                true => (labels.group_expanded)(&name),
                false => (labels.group_collapsed)(&name),
            })
        });
    });
    let context = Rc::new(RowContext {
        columns: columns.clone(),
        headers: headers.clone(),
        layout: layout.clone(),
        span: layout.len().max(1) + lead_columns,
        colindex: colindex.clone(),
        name_column,
        row_attrs: std::mem::take(&mut props.row_attrs),
        row_states: std::mem::take(&mut props.row_states),
        row_detail: if tree {
            RowFn::default()
        } else {
            std::mem::take(&mut props.row_detail)
        },
        row_has_detail: if tree {
            RowFn::default()
        } else {
            std::mem::take(&mut props.row_has_detail)
        },
        toggle_node: tree.then_some(toggle_node),
        onrowclick: props.onrowclick,
        selection: selection.clone(),
        toggle,
        details: details.clone(),
        toggle_detail,
        // A dragged row carries its detail row itself.
        animate_details: props.animate_details && !has_reorder,
        reorder: reorder.clone(),
        grid: cell_focus.then_some(grid_focus),
        tree_focus: tree.then_some(grid_focus),
        onrowupdate: props.onrowupdate,
        editable,
        announcer,
        labels,
        formats,
    });
    // A new generation redraws every row; an equal context keeps the last one.
    let generation = {
        let last = row_context.peek();
        match &*last {
            (generation, Some(last)) if last.same(&context) => *generation,
            (generation, _) => generation + 1,
        }
    };
    row_context.set((generation, Some(context.clone())));
    let groups_slice = state.expanded_groups;
    let toggle_group = use_callback(move |(id, label, open): (String, String, bool)| {
        groups_slice.set_then(toggle_row(&groups_slice.peek(), &id, open), move || {
            announcer.say(match open {
                true => (labels.group_expanded)(&label),
                false => (labels.group_collapsed)(&label),
            })
        });
    });
    let group_row = group_row(GroupSpec {
        groups: &groups,
        values: group_values,
        columns: &columns,
        headers: &headers,
        layout: &layout,
        keys: &keys,
        selected_rows: &selected_rows,
        selection: selection.as_ref(),
        leads: lead_columns,
        colindex: colindex.as_deref(),
        labels,
        toggle: toggle_group,
        select: select_group,
    });
    // A tree is a treegrid: its rows are stops, its cells too with `cell_focus`.
    let grid = (cell_focus || tree).then(|| {
        let (data, context) = (data.clone(), context.clone());
        let paths: Vec<(usize, &[String])> = layout
            .iter()
            .map(|&index| (index, headers[index].groups.as_slice()))
            .collect();
        let levels = match cell_focus {
            true => head_levels(
                &header_rows(&paths, |at| pin_edge_at(&headers, &layout, at)),
                &layout,
            ),
            false => Vec::new(),
        };
        let tree = parents
            .as_ref()
            .zip(tree_levels.as_ref())
            .map(|(parents, levels)| {
                tree_nav(TreeSpec {
                    data: &data,
                    columns: &columns,
                    keys: &keys,
                    parents,
                    levels,
                    order: &order,
                    onrowclick: props.onrowclick,
                    name_column,
                    toggle: toggle_node,
                    cells: cell_focus,
                })
            });
        let header = header_action(
            headers.clone(),
            active.clone(),
            user_sort,
            {
                let headers = headers.clone();
                move |index| menu_focus.open(&headers[index].key)
            },
            props.column_menu,
        );
        let select = selection
            .as_ref()
            .map(|_| select_action(key_at.clone(), selected_rows.clone(), mixed.clone(), toggle));
        GridRows {
            // A group row's toggle is a Tab stop of its own: the arrows pass it by.
            order: (0..levels.len())
                .map(GridRow::Head)
                .chain(
                    order
                        .iter()
                        .copied()
                        .filter(|row| row.data().is_some())
                        .map(GridRow::Body),
                )
                .collect(),
            layout: layout.clone(),
            head: Rc::new(levels),
            spans: Rc::new(move |row| match row {
                VisibleRow::Data(index) => context.spans(&data[index]),
                VisibleRow::Group(_) => Vec::new(),
            }),
            window: row_height.map(|pitch| grid_window(pitch, row_focus.focused, &order)),
            edges: (*head.peek(), foot_gap),
            header,
            select,
            editable: columns.iter().map(|column| column.edit.is_some()).collect(),
            tree,
        }
    });
    // The stop's cell, the first header cell until one took focus; a tree's first row without cells.
    let grid_stop = grid
        .as_ref()
        .and_then(|rows| rows.stop(grid_focus.stop.cloned()));
    if tree {
        grid_focus.follow_shown(grid_stop);
    }
    // A grid on a page: every page's rows count (APG grid).
    let paged = ((cell_focus || tree) && paginated && row_height.is_none()).then_some(counted);
    let striped = props.striped;
    let editing = (editable && cell_focus)
        .then(|| grid_focus.editing.cloned())
        .flatten();
    let state_of = row_state(StateSpec {
        key_at: key_at.clone(),
        selects: selection.is_some(),
        selected_rows: selected_rows.clone(),
        mixed: mixed.clone(),
        expanded,
        tree_levels: tree_levels.clone(),
        striped,
        has_reorder,
        stop: grid_stop,
        editing,
        paged,
        head_rows,
    });
    // A keyboard lift carries the view by slot: its target row may not be rendered (todo 1408).
    let region = column_drag.region;
    use_effect(use_reactive!(|(row_height, foot_gap)| {
        let (Some((_, to)), Some(pitch)) = (lift(), row_height) else {
            return;
        };
        let head = *head.peek();
        let (offset, view) = (region.scroll_offset(), region.dimensions());
        spawn(async move {
            let (Ok((x, top)), Ok(view)) = (offset.await, view.await) else {
                return;
            };
            if let Some(top) = reveal_slot(to, pitch, (head, foot_gap), top, view.height) {
                let _ = region.scroll_to(x, top);
            }
        });
    }));
    let mut attributes = std::mem::take(&mut props.attributes);
    // Only while no row shows: some readers hold back a busy subtree's rows.
    if skeleton.is_some() {
        attributes.push(attr("aria-busy", "true"));
    }
    if let Some(rows) = grid {
        attributes.push(attr("role", if tree { "treegrid" } else { "grid" }));
        if selection.is_some() {
            attributes.push(attr("aria-multiselectable", "true"));
        }
        if !editable {
            attributes.push(attr("aria-readonly", "true"));
        }
        if colindex.is_some() {
            attributes.push(attr(
                "aria-colcount",
                (lead_columns + headers.len()).to_string(),
            ));
        }
        attributes.push(grid_focus.keydown(rows));
        if tree {
            attributes.extend(grid_focus.tree_listeners());
        }
    }
    if let Some((_, total)) = paged {
        attributes.extend(window_attributes(
            head_rows,
            total,
            skeleton.is_some(),
            footer.is_some(),
        ));
    }
    let rows = body_rows(RowsSpec {
        order,
        data,
        groups,
        context,
        generation,
        key_at,
        state_of,
        group_row,
        row_height,
        counted,
        paged,
        head_rows,
        tree,
        skeleton: skeleton.is_some(),
        footer: footer.is_some(),
        stop: grid_stop,
        grid_focus,
        row_focus,
        attributes: &mut attributes,
    });

    let mut names = chrome::names(
        props.caption.take(),
        caption_id(),
        &mut attributes,
        scrolls,
        bounded,
    );

    let states: Input<States> = std::mem::take(&mut props.states)
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with("striped", props.striped)
        .with("row-click", props.onrowclick.is_some())
        .with(
            "sticky-header",
            sticks && !(head_rows > 1 && sticks_table_heads()),
        )
        .with(
            "sticky-head",
            sticks && head_rows > 1 && sticks_table_heads(),
        )
        .with("sticky-page", page_sticky)
        // Under group rows, which scroll away there, it would leave their gap. Unmeasured, it would stick at 0.
        .with(
            "sticky-filters",
            sticky_filters && group_rows == 0 && head_cell_height() > 0.0,
        )
        .with("pinned", pins)
        .with("sticky-footer", bounded)
        .with(
            "pin-footer-lead",
            pins_start && (props.selectable || has_detail || has_reorder),
        )
        .with("pin-select", pins_select)
        .with("pin-detail", has_detail && pins_start)
        .with("pin-reorder", has_reorder && pins_start)
        .with(
            "pin-reorder-detail",
            has_reorder && has_detail && pins_start,
        )
        .with("windowed", row_height.is_some())
        .into();
    // Rows the filter took away, not missing data: the caller's `empty` does not apply.
    let filtered_out = !(words.is_empty() && tests.is_empty()) && (props.manual_filter || has_data);
    let empty = match (skeleton, filtered_out) {
        (Some(rows), _) => EmptyBody::Skeleton(rows),
        (None, true) => EmptyBody::Message(
            props
                .no_results
                .take()
                .unwrap_or_else(|| rsx! { "{labels.no_results}" }),
        ),
        (None, false) => EmptyBody::Message(
            props
                .empty
                .take()
                .unwrap_or_else(|| rsx! { "{labels.no_rows}" }),
        ),
    };
    let variables: Input<Variables> = variables()
        .with(
            TABLE_ROW_HEIGHT_VAR,
            row_height.map(|height| format!("{height}px")),
        )
        .with(
            TABLE_MIN_WIDTH_VAR,
            row_height.map(|_| windowed_min_width(lead_width.as_deref(), &headers, &layout)),
        )
        .with(
            FILTERS_TOP_VAR,
            sticky_filters.then(|| format!("{}px", head_cell_height())),
        )
        .with(STICKY_TOP_VAR, sticky_offset)
        .with(
            HEAD_HEIGHT_VAR,
            page_sticky.then(|| format!("{}px", head())),
        )
        .into();
    let table = use_box()
        .framework_sx(&TABLE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .prepare()
        .event("onmounted", column_drag.table.mount())
        .render(
            HtmlTag::Table,
            attributes,
            render_body(BodySpec {
                caption: names.caption.take(),
                headers,
                order: layout,
                rows,
                empty,
                footer,
                foot: Foot {
                    element: foot_element,
                    told: bounded.then(|| EventHandler::new(move |height| foot.set(height))),
                },
                active,
                sort: user_sort,
                touch: props.multi_sort.then_some(touch),
                sort_order: labels.sort_order,
                select_all: selection
                    .as_ref()
                    .map(|selection| selection.header_cell(&selected, group_rows + 1)),
                detail_header: details
                    .as_ref()
                    .map(|details| details.header_cell(group_rows + 1)),
                menus,
                reorder_header: reorder
                    .as_ref()
                    .map(|reorder| reorder.header_cell(group_rows + 1)),
                reorder: reorder.clone(),
                filters: filter_cells,
                head_probe: sticky_filters.then_some(head_cell),
                resize,
                drag: drags.then_some(column_drag),
                on_head_height: sticks.then(|| EventHandler::new(move |height| head.set(height))),
                thead: head_element,
                pin_edge,
                pin_width,
                grid: cell_focus.then_some(HeadGrid {
                    focus: grid_focus,
                    stop: grid_stop,
                    colindex,
                }),
                paged: paged.map(|(_, total)| total),
            }),
        );
    // The live region: valid in no part of a table, so beside it.
    let announces = props.selectable
        || sortable
        || props.show_quick_filter
        || has_reorder
        || editable
        || props.column_menu
        || props.header_filters
        || props.filter_panel
        || props.toolbar.is_some()
        || onbottomreached.is_some()
        || grouped;
    let loading_bar = props.loading && skeleton.is_none();
    chrome::chrome(
        props,
        table,
        Chrome {
            names,
            caption_box,
            root_box,
            announcer,
            reorder,
            announces,
            size,
            row_height,
            pinned_pads,
            head,
            foot_gap,
            bottom_reached,
            region: column_drag.region,
            tracks_area,
            area_width,
            lead_probe,
            lead_px,
            state,
            results,
            labels,
            loading_bar,
            panel,
            active_filters,
            columns,
            onfilter,
            pager,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::TABLE_SX;
    use crate::css::Stylesheet;

    #[test]
    fn a_cell_editor_is_at_least_16px_on_ios_so_focus_does_not_zoom() {
        let stylesheet = Stylesheet::from(&*TABLE_SX);
        let css = stylesheet.as_str();
        let editor = css
            .find("[data-cell-editor] input{")
            .unwrap_or_else(|| panic!("no editor rule: {css}"));
        let rule = &css[editor..];
        let supports = rule
            .find("@supports (-webkit-touch-callout: none)")
            .unwrap_or_else(|| panic!("no iOS query: {css}"));
        assert!(rule[supports..].contains("max(16px, 1em)"), "{css}");
    }

    /// An offset of `0` made the inset ring's `calc` a number, so the browser dropped the
    /// `box-shadow`: the cell editor and seven other controls had no ring.
    #[test]
    fn every_inset_ring_draws_a_stripe() {
        let stylesheet = Stylesheet::from(&*TABLE_SX);
        let css = stylesheet.as_str();
        assert!(!css.contains("(-1 * (0"), "{css}");
        assert!(
            css.contains("[data-cell-editor] input:focus-visible{"),
            "{css}"
        );
    }
}
