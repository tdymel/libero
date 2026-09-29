use std::{
    collections::{BTreeMap, HashSet},
    rc::Rc,
};

use dioxus::prelude::*;

use super::super::sortable::SortableMove;
use crate::{
    components::{
        accessibility::use_announcer,
        common::{
            ClassList, HtmlTag, Input, LogicalTextAlign, Parts, States, Variables, attr,
            inset_focus_ring_sx, names_itself, use_name_warning, variables,
        },
        form::use_checkbox_look,
        layout::{ScrollArea, ScrollAreaBase, ScrollAreaHandle, scroll_area_base, use_box},
        overlay::MenuPart,
    },
    hooks::{
        listener, use_debounced_callback, use_element, use_id, use_localization,
        use_resize_fallback, use_theme,
    },
    platform::{
        SCROLL_PADDING_VARS, drags_table_columns, lays_out_captions, sticks_table_heads,
        widens_sized_tables,
    },
    sx::{StaticSx, Sx, sx},
    theme::{CHECKBOX_BOX_SIZE, NamedColorCss, ScrollAxis, Size, TABLE_PAD_X, TableDefaults},
    utils::warn,
};

use super::{
    column::{Column, ColumnDefaults},
    column_drag::{DropPlan, use_column_drag},
    column_filter::{ColumnFilter, FilterLogic, cell_tests},
    column_menu::{ColumnMenu, MenuColumn, use_menu_focus},
    column_order::{moved, order_unpinned, ranked},
    core::{
        BodySpec, CaptionSpec, CellSpec, RowFn, RowSpec, SortedRows, TableSort, WidthSpec,
        active_sort, header_specs, render_body,
    },
    csv::shown_csv,
    detail::Details,
    filter::{FilteredRows, QuickFilter, query_words},
    filter_panel::{FilterPanel, FilterPanelButton, PanelColumn, use_panel_control},
    filter_popover::FilterTarget,
    groups::spanned,
    header_filters::HeaderFilter,
    overlay::{EmptyBody, LoadingBar, SKELETON_ROWS},
    paging::{TablePager, clamp_page, page_rows, use_page_reset, use_page_size_reseed},
    pinning::{PinSide, PinnedColumns, pin_columns, pin_runs, span_pin},
    resize::{ColumnResize, ColumnWidths, MenuWidth},
    row_reorder::{ReorderSlot, RowReorder},
    selection::Selection,
    toolbar::{TableToolbar, TableTools, ToolView},
    use_table::{TableConfig, use_table},
    window::{
        BodyRows, RowWindow, TABLE_ROW_HEIGHT_VAR, use_row_focus, window_attributes, windowed_sx,
    },
};

/// A draggable header's grip lane, its press target.
const GRIP_LANE: &str = "24px";

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
            .color("inherit")
            .cursor("pointer"),
    )
    // Inset: the button fills its cell edge to edge.
    .selector(
        "& [data-sort-button]:focus-visible",
        inset_focus_ring_sx("-2px"),
    )
    .selector(
        "& [data-sort-button] svg",
        sx().width("16px")
            .height("16px")
            .flex_shrink("0")
            .opacity("0")
            .transition("opacity 150ms, transform 150ms"),
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
        sx().font_size("0.75em").font_weight("600").opacity("0.75"),
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
            .cursor("pointer"),
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
            .cursor("pointer"),
    )
    .selector("& [data-filtered]:focus-visible", inset_focus_ring_sx("0"))
    .selector(
        "& [data-filter-cell]",
        sx().padding_block("4px").font_weight("400"),
    )
    .selector(
        "& [data-column-menu]:focus-visible",
        inset_focus_ring_sx("0"),
    )
    // The sticky rules below outrank it: a sticky cell places the grip as well.
    .selector(
        "& th[data-resizable], & th[data-draggable]",
        sx().position("relative"),
    )
    // The header cell's box, measured for a resize's start width or a drag's layout.
    .selector(
        "& [data-resize-box], & [data-drag-box]",
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
            .touch_action("none"),
    )
    .rtl(sx().selector(
        "& [data-resize-handle]",
        sx().with("background-position", "left center"),
    ))
    .selector(
        "& th:hover [data-resize-handle], & [data-resize-handle][data-dragging], \
         & [data-resize-handle]:focus-visible",
        sx().color("primary.6").with("background-size", "2px 100%"),
    )
    .selector(
        "& [data-resize-handle]:focus-visible",
        inset_focus_ring_sx("0"),
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
        inset_focus_ring_sx("0"),
    )
    .selector(
        "& [data-detail-button] svg",
        sx().width("16px")
            .height("16px")
            .transition("transform 150ms"),
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
    .when(
        "row-click",
        sx().selector(
            "& tbody tr:not([data-empty]):not([data-detail]):not([data-skeleton])",
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
                .top("0")
                .z_index("1")
                .background(NamedColorCss::SURFACE.value()),
        ),
    )
    // Several header rows can't all stick at the top: the `thead` does.
    .when(
        "sticky-head",
        sx().selector("& thead", sx().position("sticky").top("0").z_index("1"))
            .selector(
                "& thead th, & thead td",
                sx().background(NamedColorCss::SURFACE.value()),
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
            "& [data-pin=\"start\"][data-pin-edge]",
            sx().with("border-inline-end", TableDefaults::border()),
        )
        .selector(
            "& [data-pin=\"end\"][data-pin-edge]",
            sx().with("border-inline-start", TableDefaults::border()),
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

/// As tall as the table, up to `max_height`; not the parent's height.
static TABLE_SCROLL_SX: StaticSx =
    StaticSx::new(|| scroll_area_base(sx().height("auto").max_width("100%")));

#[derive(Props, Clone, PartialEq)]
pub struct TableProps<T: Clone + PartialEq + 'static> {
    /// One row each, in source order until a column is sorted.
    data: Vec<T>,
    /// Built with [`column`](super::column).
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
    /// Scrolls a table wider than its parent sideways, in a `ScrollArea`.
    #[props(default)]
    scroll: bool,
    /// Caps the height, a CSS length: the rows scroll in a `ScrollArea` (both
    /// axes) under a header that stays put.
    #[props(default, into)]
    max_height: Option<String>,
    /// With `max_height`, renders only the rows in view, each clipped to this
    /// height in px with one line per cell. Off with `row_detail` or `onrowreorder`.
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
    /// The selected rows' `row_key`s; set, the selection is controlled.
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
    /// `data` is the current page: the table only draws the page controls.
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
pub fn Table<T: Clone + PartialEq + 'static>(props: TableProps<T>) -> Element {
    let state = use_table(TableConfig {
        sort: props.sort,
        default_sort: props.default_sort,
        onsortchange: props.onsortchange,
        selection: props.selection,
        default_selection: props.default_selection,
        onselectionchange: props.onselectionchange,
        page: props.page,
        default_page: props.default_page,
        onpagechange: props.onpagechange,
        page_size: props.page_size,
        default_page_size: props
            .default_page_size
            .or(props.page_sizes.first().copied())
            .unwrap_or(10),
        onpagesizechange: props.onpagesizechange,
        hidden_columns: props.hidden_columns,
        default_hidden_columns: props.default_hidden_columns,
        onhiddencolumnschange: props.onhiddencolumnschange,
        quick_filter: props.quick_filter,
        default_quick_filter: props.default_quick_filter,
        onquickfilterchange: props.onquickfilterchange,
        column_filters: props.column_filters,
        default_column_filters: props.default_column_filters,
        oncolumnfilterschange: props.oncolumnfilterschange,
        filter_logic: props.filter_logic,
        default_filter_logic: props.default_filter_logic,
        onfilterlogicchange: props.onfilterlogicchange,
        pinned_columns: props.pinned_columns,
        default_pinned_columns: props.default_pinned_columns,
        onpinnedcolumnschange: props.onpinnedcolumnschange,
        expanded: props.expanded,
        default_expanded: props.default_expanded,
        onexpandedchange: props.onexpandedchange,
        column_order: props.column_order,
        default_column_order: props.default_column_order,
        oncolumnorderchange: props.oncolumnorderchange,
        column_widths: props.column_widths,
        default_column_widths: props.default_column_widths,
        oncolumnwidthschange: props.oncolumnwidthschange,
        density: props.density,
        default_density: props.default_density,
        ondensitychange: props.ondensitychange,
    });
    let panel = use_panel_control();
    let tools =
        use_context_provider(|| TableTools::new(state.hidden_columns, state.density, panel));
    let preview = use_signal(|| None);
    let measured = use_hook(|| CopyValue::new(BTreeMap::new()));
    let announcer = use_announcer();
    let column_drag = use_column_drag(state.column_order);
    let menu_focus = use_menu_focus();
    let drags = props.column_menu && drags_table_columns();
    let touch = use_hook(|| CopyValue::new(false));
    use_hook(|| {
        if props.selectable && !props.row_key.is_set() {
            warn(
                "Table: `selectable` without `row_key` keys the selection by row index, so it \
                 moves to other rows when `data` changes.",
            );
        }
        if props.row_detail.is_set() && !props.row_key.is_set() {
            warn(
                "Table: `row_detail` without `row_key` keys the open details by row index, so \
                 they move to other rows when `data` changes.",
            );
        }
        if props.virtual_row_height.is_some() && props.max_height.is_none() {
            warn("Table: `virtual_row_height` without `max_height` renders every row.");
        }
        if props.onbottomreached.is_some() && props.max_height.is_none() {
            warn(
                "Table: `onbottomreached` without `max_height` never fires: the page scrolls, not the table.",
            );
        }
        if props.virtual_row_height.is_some()
            && (props.row_detail.is_set() || props.onrowreorder.is_some())
        {
            warn(
                "Table: `virtual_row_height` with `row_detail` or `onrowreorder` renders every \
                 row: a detail row breaks the one row height, and a drag needs every slot.",
            );
        }
        if props.onrowreorder.is_some() && !props.row_key.is_set() {
            warn(
                "Table: `onrowreorder` without `row_key` keys the rows by index, so a moved row \
                 is rebuilt and its handle loses the focus.",
            );
        }
        if props.resizable_columns && !props.column_menu {
            warn(
                "Table: `resizable_columns` without `column_menu` resizes by drag only, out of \
                 reach of the keyboard and of a pointer that cannot drag (WCAG 2.5.7).",
            );
        }
    });
    let instructions_id = use_id();
    // The shown rows' indices in `data`, which a reorder's slots map to.
    let mut shown_rows = use_hook(|| CopyValue::new(Vec::<usize>::new()));
    let onrowreorder = props.onrowreorder;
    let reorder_rows = use_callback(move |step: SortableMove| {
        let shown = shown_rows.peek();
        if let (Some(onrowreorder), Some(&from), Some(&to)) =
            (onrowreorder, shown.get(step.from), shown.get(step.to))
        {
            onrowreorder.call(SortableMove { from, to });
        }
    });
    let detail_id = use_id();
    let paginated = props.page_size.is_some()
        || props.default_page_size.is_some()
        || !props.page_sizes.is_empty();
    use_page_reset(state);
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
    let labels = use_localization().table;
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
    let filter_target = |index: usize| {
        let column = &props.columns[index];
        column.filterable.then(|| FilterTarget {
            column: column.header.clone(),
            kind: column.filter_kind,
            slice: state.column_filters,
            labels,
            size,
            onfilter,
        })
    };
    let bounded = props.max_height.is_some();
    let mut head = use_signal(|| 0.0);
    let head_element = use_element();
    use_resize_fallback(head_element, move |event| {
        if let (true, Ok(size)) = (bounded, event.get_border_box_size()) {
            head.set(size.height);
        }
    });
    let scrolls = props.scroll || bounded;
    let size_states: Input<States> = States::new().with(size.state_name(), true).into();
    let caption_box = use_box()
        .framework_sx(&TABLE_CAPTION_SX)
        .states(&size_states)
        .prepare();
    let look = use_checkbox_look(size, props.selectable);
    let mut sorted = use_hook(|| CopyValue::new(SortedRows::<T>::default()));
    let mut filtered = use_hook(|| CopyValue::new(FilteredRows::<T>::default()));
    let row_focus = use_row_focus();

    let widths = state.column_widths.read();
    let mut headers = header_specs(
        &props.columns,
        &props.column_defaults,
        &state.hidden_columns.read(),
        WidthSpec {
            resizable: props.resizable_columns,
            widths: &widths,
            preview: resize.and_then(|_| preview()),
        },
    );
    let pinned = state.pinned_columns.read();
    let has_detail = props.row_detail.is_set();
    let has_reorder = props.onrowreorder.is_some();
    let leads: Vec<String> = [
        has_reorder.then(reorder_width),
        has_detail.then(detail_width),
        props.selectable.then(|| select_width(size)),
    ]
    .into_iter()
    .flatten()
    .collect();
    let lead_width = match leads.as_slice() {
        [] => None,
        [one] => Some(one.clone()),
        many => Some(format!("calc({})", many.join(" + "))),
    };
    let (mut layout, unknown) = pin_columns(&mut headers, &pinned, lead_width.as_deref());
    let column_order = state.column_order.read();
    order_unpinned(&headers, &mut layout, &column_order);
    if drags {
        let plan = DropPlan {
            ranked: ranked(&headers, &column_order)
                .into_iter()
                .map(|index| headers[index].header.clone())
                .collect(),
            unpinned: layout
                .iter()
                .filter(|&&index| headers[index].pin.is_none())
                .map(|&index| (index, headers[index].header.clone()))
                .collect(),
        };
        let mut current = column_drag.plan;
        if *current.peek() != plan {
            current.set(plan);
        }
    }
    let pins_start = headers.iter().any(|spec| {
        spec.pin
            .as_ref()
            .is_some_and(|pin| pin.side == PinSide::Start)
    });
    let pins_select = props.selectable && pins_start;
    let pins = headers.iter().any(|spec| spec.pin.is_some());
    let mut warned = use_hook(|| CopyValue::new(Vec::<String>::new()));
    if !unknown.is_empty() && *warned.peek() != unknown {
        warn(&format!(
            "Table: pinned columns {unknown:?} sit past a pinned column without `width`, so \
             their offset is unknown and they overlap it."
        ));
        warned.set(unknown);
    }
    let active = active_sort(&headers, &state.sort.read(), props.multi_sort);
    let group_rows = layout
        .iter()
        .map(|&index| headers[index].groups.len())
        .max()
        .unwrap_or(0);
    let head_rows = group_rows + 1 + usize::from(props.header_filters);
    let menu_columns: Rc<[MenuColumn]> = headers
        .iter()
        .map(|spec| MenuColumn {
            header: spec.header.clone(),
            sortable: spec.sortable,
            hideable: spec.hideable,
            hidden: spec.hidden,
        })
        .collect();
    let tests = cell_tests(&props.columns, &state.column_filters.read());
    let active_filters = tests.len();
    if props.toolbar.is_some() {
        tools.show(ToolView {
            columns: menu_columns.clone(),
            size,
            filters: props.filter_panel.then_some(active_filters),
        });
    }
    let menus = match props.column_menu {
        true => {
            let columns = menu_columns;
            let active = Rc::new(active.clone());
            (0..headers.len())
                .map(|index| {
                    // The next shown column, else the one before.
                    let at = layout.iter().position(|&shown| shown == index);
                    let neighbour = at.and_then(|at| {
                        layout
                            .get(at + 1)
                            .or_else(|| at.checked_sub(1).and_then(|before| layout.get(before)))
                            .copied()
                    });
                    rsx! {
                        ColumnMenu {
                            index,
                            columns: columns.clone(),
                            moves: (
                                moved(&headers, &column_order, index, false),
                                moved(&headers, &column_order, index, true),
                            ),
                            order: state.column_order,
                            width: resize.zip(headers[index].resize).map(|(resize, limits)| MenuWidth {
                                resize,
                                limits,
                                width: widths.get(&headers[index].header).copied(),
                                announcer,
                            }),
                            active: active.clone(),
                            sort: state.sort,
                            multi_sort: props.multi_sort,
                            hidden: state.hidden_columns,
                            pinned: state.pinned_columns,
                            labels,
                            size,
                            parts: props.column_menu_parts.clone(),
                            filter: filter_target(index),
                            panel: props.filter_panel.then_some(panel),
                            focus: menu_focus,
                            neighbour,
                        }
                    }
                })
                .collect()
        }
        false => Vec::new(),
    };
    let filter_cells = props.header_filters.then(|| {
        (0..headers.len())
            .map(|index| {
                filter_target(index).map(|target| {
                    rsx! {
                        HeaderFilter { key: "{target.column}", target }
                    }
                })
            })
            .collect::<Vec<_>>()
    });

    let data = Rc::new(props.data);
    let row_key = props.row_key.clone();
    let key_of = {
        let data = data.clone();
        move |index: usize| {
            row_key
                .call(&data[index])
                .unwrap_or_else(|| index.to_string())
        }
    };
    // The quick filter searches the shown, filterable columns.
    let words = query_words(&state.quick_filter.read());
    let searched: Vec<usize> = props
        .columns
        .iter()
        .zip(&headers)
        .enumerate()
        .filter(|(_, (column, spec))| column.filterable && !spec.hidden)
        .map(|(index, _)| index)
        .collect();
    let panel_columns: Vec<PanelColumn> = props
        .columns
        .iter()
        .filter(|column| column.filterable)
        .map(|column| PanelColumn {
            header: column.header.clone(),
            kind: column.filter_kind,
        })
        .collect();
    let kept = match props.manual_filter {
        true => None,
        false => filtered.write().kept(
            &data,
            &props.columns,
            &searched,
            &words,
            &tests,
            state.filter_logic.read(),
        ),
    };
    let selection = look.map(|look| {
        let keys: Rc<[String]> = (0..data.len()).map(&key_of).collect();
        let scope = match &kept {
            Some(kept) => keys
                .iter()
                .zip(kept.iter())
                .filter(|(_, kept)| **kept)
                .map(|(key, _)| key.clone())
                .collect(),
            None => keys.clone(),
        };
        Selection {
            slice: state.selection,
            keys,
            scope,
            announcer,
            labels,
            size,
            look: Rc::new(look),
        }
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
    // A row's checkbox and toggle are named by its row header, else its first cell.
    let name_column = props
        .columns
        .iter()
        .position(|column| column.row_header)
        .unwrap_or(0);
    let columns = Rc::new(props.columns);
    // Filter, sort, then page; a `manual_*` stage is the caller's. Filtering
    // the sorted order keeps each stage's cache apart: a keystroke never re-sorts.
    let mut order: Vec<usize> = match props.manual_sort {
        true => (0..data.len()).collect(),
        false => sorted.write().order(&data, &columns, &active),
    };
    if let Some(kept) = &kept {
        order.retain(|&index| kept[index]);
    }
    let results = order.len();
    filtered_count.set(results);
    if props.toolbar.is_some() {
        let (data, columns, shown, order) =
            (data.clone(), columns.clone(), layout.clone(), order.clone());
        let mut export = tools.export;
        export.set(Some(Rc::new(move || {
            shown_csv(&columns, &shown, &data, &order)
        })));
    }
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
    let pager = paginated.then(|| {
        let total = match props.manual_pagination {
            true => props.row_count.unwrap_or(results),
            false => results,
        };
        let page_size = state.page_size.read().max(1);
        let page = clamp_page(total, state.page.read(), page_size);
        if !props.manual_pagination {
            let shown = page_rows(total, page, page_size);
            order.truncate(shown.end);
            order.drain(..shown.start);
        }
        rsx! {
            TablePager {
                state,
                total,
                page,
                page_size,
                page_sizes: props.page_sizes,
                caption: props.caption.clone(),
                size,
            }
        }
    });
    if has_reorder {
        shown_rows.set(order.clone());
    }
    let skeleton = (props.loading && order.is_empty()).then(|| match paginated {
        true => state.page_size.read().max(1),
        false => SKELETON_ROWS,
    });
    // Owned, so a virtualised body can project its rows as they scroll in.
    let (row_attrs, row_states, onrowclick) = (props.row_attrs, props.row_states, props.onrowclick);
    let row_selection = selection.clone();
    let key_at: Rc<dyn Fn(usize) -> String> = match &selection {
        Some(selection) => {
            let keys = selection.keys.clone();
            Rc::new(move |index| keys[index].clone())
        }
        None => Rc::new(key_of),
    };
    let has_data = !data.is_empty();
    let headers = Rc::new(headers);
    let (row_headers, row_layout, row_key_at) = (headers.clone(), layout.clone(), key_at.clone());
    let (row_detail, striped, row_details) = (props.row_detail, props.striped, details.clone());
    let row_has_detail = props.row_has_detail;
    // A dragged row carries its detail row itself.
    let animate_details = props.animate_details && !has_reorder;
    let row_of = move |position: usize, index: usize| {
        let row = &data[index];
        let key = row_key_at(index);
        let name = || {
            columns
                .get(name_column)
                .map(|column| (column.text)(row))
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| key.clone())
        };
        let (open, detail) = match row_has_detail.call(row) {
            Some(has) => {
                let open = has.then(|| expanded.contains(&key));
                let detail = match open {
                    Some(true) => row_detail.call(row).flatten(),
                    _ => None,
                };
                (open, detail)
            }
            None => {
                let detail = row_detail.call(row).flatten();
                (detail.as_ref().map(|_| expanded.contains(&key)), detail)
            }
        };
        let detail_cell = row_details
            .as_ref()
            .map(|details| details.row_cell(key.clone(), &name(), index, open, toggle_detail));
        let sliding = row_details
            .as_ref()
            .zip(open)
            .filter(|_| animate_details)
            .map(|(details, open)| (details.row_id(index), open));
        let detail = match (&row_details, open) {
            (Some(details), Some(true)) => detail.map(|body| (details.row_id(index), body)),
            _ => None,
        };
        let mut attributes = row_attrs.call(row).unwrap_or_default();
        if let Some(onrowclick) = onrowclick {
            let data = data.clone();
            attributes.push(listener("onclick", move |_: Event<MouseData>| {
                onrowclick.call(data[index].clone());
            }));
        }
        let is_selected = row_selection.as_ref().map(|_| selected_rows.contains(&key));
        let select = row_selection.as_ref().map(|selection| {
            selection.row_cell(key.clone(), &name(), is_selected == Some(true), toggle)
        });
        let reorder = has_reorder.then(|| ReorderSlot {
            slot: position,
            label: name(),
        });
        RowSpec {
            reorder,
            selected: is_selected,
            select,
            stripe: striped && position % 2 == 1,
            toggle: detail_cell,
            detail,
            sliding,
            states: row_states.call(row).and_then(|states| states.data_state()),
            attributes,
            cells: pin_runs(&row_headers, &row_layout)
                .flat_map(|run| {
                    let mut at = 0;
                    spanned(run, |index| {
                        columns[index].col_span.as_ref().map_or(1, |span| span(row))
                    })
                    .into_iter()
                    .map(move |(index, span)| {
                        let covered = &run[at..at + span];
                        at += span;
                        (index, span, covered)
                    })
                })
                .map(|(index, span, covered)| {
                    let column = &columns[index];
                    let (text, body) = match &column.render {
                        Some(render) => (String::new(), Some(render(row))),
                        None => ((column.text)(row), None),
                    };
                    CellSpec {
                        column: index,
                        text,
                        body,
                        span,
                        pin: span_pin(&row_headers, covered),
                    }
                })
                .collect(),
            key,
        }
    };
    // Detail rows have their own heights, and a drag moves between every slot.
    let row_height = props
        .virtual_row_height
        .filter(|_| bounded && !has_detail && !has_reorder);
    let mut attributes = props.attributes;
    // Only while no row shows: some readers hold back a busy subtree's rows.
    if skeleton.is_some() {
        attributes.push(attr("aria-busy", "true"));
    }
    let rows = match row_height {
        Some(row_height) => {
            attributes.extend(window_attributes(head_rows, order.len()));
            attributes.extend(row_focus.attributes());
            let order: Rc<[usize]> = order.into();
            row_focus.show(head_rows, order.clone());
            BodyRows::Window(RowWindow {
                order,
                row_height,
                row: Rc::new(row_of),
                key: key_at,
                focused: row_focus.focused,
            })
        }
        None => BodyRows::All(
            order
                .into_iter()
                .enumerate()
                .map(|(position, index)| row_of(position, index))
                .collect(),
        ),
    };

    let described_by = props.caption.as_ref().map(|_| caption_id());
    let mut caption = props.caption.map(|text| CaptionSpec {
        text,
        id: caption_id(),
    });
    // The region takes the table's name: its caption, else the caller's label.
    let region_name: Vec<Attribute> = match &caption {
        Some(spec) => vec![attr("aria-labelledby", spec.id.clone())],
        None if scrolls => attributes
            .iter()
            .filter(|a| matches!(a.name, "aria-label" | "aria-labelledby"))
            .cloned()
            .collect(),
        None => Vec::new(),
    };
    // Without a caption, the table's own name tells its quick filter apart (todo 1376).
    let text_attr = |name: &str| {
        attributes
            .iter()
            .find(|a| a.name == name)
            .and_then(|a| match &a.value {
                dioxus::core::AttributeValue::Text(text) => Some(text.clone()),
                _ => None,
            })
    };
    let (described_by, table_label) = match described_by {
        Some(id) => (Some(id), None),
        None => match text_attr("aria-labelledby") {
            Some(ids) => (Some(ids), None),
            None => (None, text_attr("aria-label")),
        },
    };

    // Blitz skips a `<caption>`, and a capped table's would scroll away over its
    // sticky header: a div before the table then, naming it.
    let before = if lays_out_captions() && !bounded {
        None
    } else {
        caption.take()
    };
    if let Some(spec) = &before
        && !names_itself(&attributes)
    {
        attributes.push(attr("aria-labelledby", spec.id.clone()));
    }

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with("striped", props.striped)
        .with("row-click", props.onrowclick.is_some())
        .with(
            "sticky-header",
            bounded && !(head_rows > 1 && sticks_table_heads()),
        )
        .with(
            "sticky-head",
            bounded && head_rows > 1 && sticks_table_heads(),
        )
        .with("pinned", pins)
        .with("pin-select", pins_select)
        .with("pin-detail", has_detail && pins_start)
        .with("pin-reorder", has_reorder && pins_start)
        .with(
            "pin-reorder-detail",
            has_reorder && has_detail && pins_start,
        )
        .with("windowed", row_height.is_some())
        .into();
    let reorder = has_reorder.then(|| RowReorder {
        onreorder: reorder_rows,
        // The shown order is not `data`'s: a move between slots means nothing there.
        disabled: !active.is_empty() || !words.is_empty() || !tests.is_empty(),
        announcer,
        labels,
        instructions: instructions_id(),
    });
    // Rows the filter took away, not missing data: the caller's `empty` does not apply.
    let filtered_out = !(words.is_empty() && tests.is_empty()) && (props.manual_filter || has_data);
    let empty = match (skeleton, filtered_out) {
        (Some(rows), _) => EmptyBody::Skeleton(rows),
        (None, true) => EmptyBody::Message(
            props
                .no_results
                .unwrap_or_else(|| rsx! { "{labels.no_results}" }),
        ),
        (None, false) => {
            EmptyBody::Message(props.empty.unwrap_or_else(|| rsx! { "{labels.no_rows}" }))
        }
    };
    let variables: Input<Variables> = variables()
        .with(
            TABLE_ROW_HEIGHT_VAR,
            row_height.map(|height| format!("{height}px")),
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
                caption,
                headers,
                order: layout,
                rows,
                empty,
                active,
                sort: state.sort,
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
                resize,
                drag: drags.then_some(column_drag),
                head_height: bounded.then(|| EventHandler::new(move |height| head.set(height))),
                head: head_element,
            }),
        );
    // The live region: valid in no part of a table, so beside it.
    let announces = props.selectable
        || props.show_quick_filter
        || has_reorder
        || props.column_menu
        || props.header_filters
        || props.filter_panel
        || onbottomreached.is_some();
    let table = match announces {
        true => rsx! {
            {table}
            {announcer.render()}
            {reorder.as_ref().map(RowReorder::instructions)}
        },
        false => table,
    };
    let before = before.map(|spec| {
        caption_box.render(
            HtmlTag::Div,
            vec![attr("id", spec.id)],
            rsx! { "{spec.text}" },
        )
    });
    // Capped, above the scroll area: the header then starts at its top (todo 1454).
    let (before, above) = match bounded {
        true => (None, before),
        false => (before, None),
    };
    let table = match before {
        // One box, so a flex row does not set them side by side.
        Some(caption) => match scrolls || pager.is_some() {
            true => rsx! {
                {caption}
                {table}
            },
            false => rsx! {
                div { {caption} {table} }
            },
        },
        None => table,
    };
    // A tab stop, a named region, only while it overflows with no header
    // button inside to scroll it from (decision 7).
    let table = match scrolls {
        true => {
            let mut attributes = region_name;
            attributes.push(attr("data-table-scroll", "true"));
            let max_height = props.max_height.clone();
            rsx! {
                ScrollArea {
                    scrollbars: if bounded { ScrollAxis::Both } else { ScrollAxis::Horizontal },
                    framework_sx: ScrollAreaBase(&TABLE_SCROLL_SX),
                    sx: max_height
                        .map(|height| {
                            // The sticky header covers the top; a focus scroll stops below it.
                            let pad = format!("{}px", head());
                            sx().max_height(height)
                                .with("scroll-padding-top", pad.clone())
                                .with(SCROLL_PADDING_VARS[0], pad)
                        })
                        .unwrap_or_default(),
                    onbottomreached: onbottomreached.map(|_| bottom_reached),
                    // The scrollbar runs beside the rows only, not the sticky header (todo 1454).
                    bar_inset_top: bounded.then(|| *head.read()),
                    // Its root, mounted by the area itself, is where a column drop counts.
                    handle: ScrollAreaHandle { element: column_drag.region },
                    attributes,
                    {table}
                }
            }
        }
        false => table,
    };
    let search = props.show_quick_filter.then(|| {
        rsx! {
            QuickFilter {
                slice: state.quick_filter,
                results,
                announcer,
                labels,
                size,
                caption: described_by,
                table_label,
            }
        }
    });
    // A placeholder while off, so toggling `loading` keeps the table's nodes.
    let bar = (props.loading && skeleton.is_none()).then(|| {
        rsx! {
            LoadingBar { labels }
        }
    });
    let table = match above {
        Some(caption) => rsx! {
            div {
                {caption}
                {bar}
                {table}
            }
        },
        None => rsx! {
            {bar}
            {table}
        },
    };
    // With a `toolbar`, its `TableFilterButton` stands in for the button (todo 1448).
    let own_button = props.toolbar.is_none();
    let start = props.filter_panel.then(|| {
        let columns: Rc<[PanelColumn]> = panel_columns.into();
        rsx! {
            if own_button {
                FilterPanelButton { control: panel, labels, size, active: active_filters }
            }
            FilterPanel {
                control: panel,
                columns,
                slice: state.column_filters,
                logic: state.filter_logic,
                labels,
                size,
                onfilter,
            }
        }
    });
    let search = match (start, props.toolbar) {
        (None, None) => search,
        (start, content) => Some(rsx! {
            TableToolbar { start, content, search }
        }),
    };
    // Outside the scroll region, so the controls stay put while it scrolls. One box,
    // or a flex parent squeezes the zero-height loading bar to no width (todo 1461).
    rsx! {
        div {
            {search}
            {table}
            {pager}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TABLE_SX;
    use crate::css::Stylesheet;

    #[test]
    fn pinned_toggles_push_the_checkboxes_past_them() {
        let stylesheet = Stylesheet::from(&*TABLE_SX);
        let css = stylesheet.as_str();
        let rule = |state: &str| {
            css.find(&format!("[data-state~=\"{state}\"] [data-select]{{"))
                .unwrap_or_else(|| panic!("{state}: {css}"))
        };
        // Equally specific: the later rule's inset wins.
        assert!(rule("pin-select") < rule("pin-detail"));
    }
}
