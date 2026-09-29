use std::rc::Rc;

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{
    cell_value::{CellAlign, SortDirection, SortKey},
    column::{Column, ColumnDefaults},
    column_drag::{ColumnDrag, ColumnDragGrip},
    detail::SlidingDetail,
    groups::{HeaderCell, header_rows},
    header_filters::filter_row,
    overlay::EmptyBody,
    pinning::{CellPin, pin_edge_at, span_pin},
    resize::{ColumnResize, ColumnWidths, ResizeHandle, ResizeLimits},
    row_reorder::{ReorderRow, ReorderSlot, RowReorder},
    use_table::StateSlice,
    window::{BodyRows, render_window},
};
use crate::{
    components::{accessibility::VisuallyHidden, common::Glyph},
    context::IconSlot,
    localization::fill,
};

/// One entry of a [`Table`](super::Table)'s sort: a column, named by its header text.
///
/// ```rust
/// # use libero::components::{SortDirection, TableSort};
/// let by_age = vec![TableSort::new("Age", SortDirection::Descending)];
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct TableSort {
    pub column: String,
    pub direction: SortDirection,
}

impl TableSort {
    pub fn new(column: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            column: column.into(),
            direction,
        }
    }
}

/// A per-row closure for a [`Table`](super::Table) prop, as `row_key`, or none
/// by default. Two set ones compare equal, as a column's closures do.
///
/// ```rust
/// # use libero::components::RowFn;
/// # struct User { id: u64 }
/// let key: RowFn<User, String> = (|u: &User| u.id.to_string()).into();
/// ```
pub struct RowFn<T, R>(Option<RowClosure<T, R>>);

type RowClosure<T, R> = Rc<dyn Fn(&T) -> R>;

impl<T, R> RowFn<T, R> {
    /// `None` when unset.
    pub(super) fn call(&self, row: &T) -> Option<R> {
        self.0.as_ref().map(|f| f(row))
    }

    pub(super) fn is_set(&self) -> bool {
        self.0.is_some()
    }
}

impl<T, R, F: Fn(&T) -> R + 'static> From<F> for RowFn<T, R> {
    fn from(f: F) -> Self {
        Self(Some(Rc::new(f)))
    }
}

// Hand-written: a derive would demand `T: Default`, `T: Clone` and `R: Clone`.
impl<T, R> Default for RowFn<T, R> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T, R> Clone for RowFn<T, R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// Set against unset differs, so turning a prop on re-renders the table (1440).
impl<T, R> PartialEq for RowFn<T, R> {
    fn eq(&self, other: &Self) -> bool {
        self.is_set() == other.is_set()
    }
}

/// What the non-generic body needs from a `Column<T>`, once `T` is gone.
pub(super) struct HeaderSpec {
    pub header: String,
    pub align: CellAlign,
    pub sortable: bool,
    pub row_header: bool,
    pub hideable: bool,
    /// Named in `hidden_columns`: no cells drawn, its sort still applies.
    pub hidden: bool,
    /// The resolved `width`, which a pinned column further in is inset by.
    pub width: Option<String>,
    /// The header cell's inline `width`/`min-width`, which size the column.
    pub style: Option<String>,
    /// A caller's header body, which replaces the text.
    pub body: Option<Element>,
    /// The group headers above it, outermost first.
    pub groups: Vec<String>,
    /// Set by [`pin_columns`](super::pinning::pin_columns).
    pub pin: Option<CellPin>,
    /// With `resizable_columns`, unless the column opted out: its resize limits.
    pub resize: Option<ResizeLimits>,
}

/// How the table sizes its columns: `resizable_columns`, the resized widths,
/// and a drag's width before it ends, by header index.
pub(super) struct WidthSpec<'a> {
    pub resizable: bool,
    pub widths: &'a ColumnWidths,
    pub preview: Option<(usize, f64)>,
}

pub(super) fn header_specs<T>(
    columns: &[Column<T>],
    defaults: &ColumnDefaults,
    hidden: &[String],
    sizing: WidthSpec,
) -> Vec<HeaderSpec> {
    columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            let resolved = column.resolve(defaults);
            let hidden = hidden.contains(&column.header);
            let resized = match sizing.preview {
                Some((at, width)) if at == index => Some(width),
                _ => sizing.widths.get(&column.header).copied(),
            }
            .map(|width| format!("{width}px"));
            let width = resized.as_deref().or(resolved.width);
            HeaderSpec {
                header: column.header.clone(),
                align: resolved.align,
                sortable: column.sortable,
                row_header: column.row_header,
                hideable: column.hideable,
                hidden,
                width: width.map(str::to_string),
                style: width_style(width, resolved.min_width),
                body: match hidden {
                    true => None,
                    false => column.header_render.as_ref().map(|render| render()),
                },
                groups: column.groups.clone(),
                pin: None,
                resize: (sizing.resizable && column.resizable).then_some(column.resize_limits),
            }
        })
        .collect()
}

// Not `<col>`: Blitz ignores its width (blitz-table-probe). Border-box, as
// Blitz reads a cell's width.
fn width_style(width: Option<&str>, min_width: Option<&str>) -> Option<String> {
    if width.is_none() && min_width.is_none() {
        return None;
    }
    let mut style = String::from("box-sizing:border-box;");
    if let Some(width) = width {
        style.push_str(&format!("width:{width};"));
    }
    if let Some(min_width) = min_width {
        style.push_str(&format!("min-width:{min_width};"));
    }
    Some(style)
}

/// The sorted columns by header index, in priority order.
pub(super) type ActiveSort = Vec<(usize, SortDirection)>;

/// `data`'s indices in display order: source order unless a column is sorted.
pub(super) fn row_order<T>(
    data: &[T],
    columns: &[Column<T>],
    active: &[(usize, SortDirection)],
) -> Vec<usize> {
    if active.is_empty() {
        return (0..data.len()).collect();
    }
    let keys: Vec<(Vec<SortKey>, SortDirection)> = active
        .iter()
        .map(|&(index, direction)| {
            let sort_key = &columns[index].sort_key;
            (data.iter().map(|row| sort_key(row)).collect(), direction)
        })
        .collect();
    sorted_order(&keys)
}

/// [`row_order`] across renders: a page, selection or unrelated prop change
/// reuses it, so sort keys are computed only when the rows, columns or sort change.
pub(super) struct SortedRows<T> {
    input: Option<SortInput<T>>,
    order: Vec<usize>,
}

type SortInput<T> = (Rc<Vec<T>>, Vec<Column<T>>, ActiveSort);

impl<T> Default for SortedRows<T> {
    fn default() -> Self {
        Self {
            input: None,
            order: Vec::new(),
        }
    }
}

impl<T: PartialEq> SortedRows<T> {
    pub fn order(
        &mut self,
        data: &Rc<Vec<T>>,
        columns: &[Column<T>],
        active: &ActiveSort,
    ) -> Vec<usize> {
        if active.is_empty() {
            *self = Self::default();
            return (0..data.len()).collect();
        }
        let fresh = self.input.as_ref().is_some_and(|(rows, cols, sort)| {
            sort == active && cols.as_slice() == columns && (Rc::ptr_eq(rows, data) || rows == data)
        });
        if !fresh {
            self.order = row_order(data, columns, active);
            self.input = Some((data.clone(), columns.to_vec(), active.clone()));
        }
        self.order.clone()
    }
}

/// What a click on header `index` asks for: ascending, descending, then unsorted.
/// `add` keeps the other sorted columns and appends a new one last.
pub(super) fn next_sort(
    active: &[(usize, SortDirection)],
    headers: &[HeaderSpec],
    index: usize,
    add: bool,
) -> Vec<TableSort> {
    let current = active
        .iter()
        .find(|(column, _)| *column == index)
        .map(|(_, direction)| *direction);
    let next = match current {
        None => Some(SortDirection::Ascending),
        Some(SortDirection::Ascending) => Some(SortDirection::Descending),
        Some(SortDirection::Descending) => None,
    };
    let entry = |(column, direction): (usize, SortDirection)| {
        TableSort::new(&headers[column].header, direction)
    };
    if !add {
        return next
            .map(|direction| entry((index, direction)))
            .into_iter()
            .collect();
    }
    active
        .iter()
        .filter_map(|&(column, direction)| match column == index {
            true => next.map(|next| (column, next)),
            false => Some((column, direction)),
        })
        .chain(
            current
                .is_none()
                .then_some((index, SortDirection::Ascending)),
        )
        .map(entry)
        .collect()
}

/// A row's cells and its DOM key: the caller's `row_key`, else its index in
/// `data`. A cell is its text, or a caller's body.
pub(super) struct RowSpec {
    pub key: String,
    /// The row's `data-state`.
    pub states: Option<String>,
    pub attributes: Vec<Attribute>,
    pub cells: Vec<CellSpec>,
    /// With `selectable`: whether it is selected, and its checkbox cell.
    pub selected: Option<bool>,
    pub select: Option<Element>,
    /// With `striped`: an odd row among the shown ones, detail rows not counted.
    pub stripe: bool,
    /// With `row_detail`: its toggle cell, and while open its detail row's id and body.
    pub toggle: Option<Element>,
    pub detail: Option<(String, Element)>,
    /// With `animate_details`, for a row with a toggle: its detail row's id and
    /// whether it is open; the row then slides open and shut.
    pub sliding: Option<(String, bool)>,
    /// With `onrowreorder`: its slot among the shown rows and its name.
    pub reorder: Option<ReorderSlot>,
}

/// A body cell of header `column`: its text, or a caller's body, over `span`
/// shown columns.
pub(super) struct CellSpec {
    pub column: usize,
    pub text: String,
    pub body: Option<Element>,
    pub span: usize,
    pub pin: Option<CellPin>,
}

/// Row indices in sorted order, one key column per sorted column, the first
/// deciding. Stable, so equal keys keep source order, and `Empty` sinks to the
/// bottom either way.
pub(super) fn sorted_order(keys: &[(Vec<SortKey>, SortDirection)]) -> Vec<usize> {
    let len = keys.first().map_or(0, |(column, _)| column.len());
    let mut order: Vec<usize> = (0..len).collect();
    order.sort_by(|&a, &b| {
        keys.iter()
            .map(|(column, direction)| {
                let (a, b) = (&column[a], &column[b]);
                match (a.is_empty(), b.is_empty()) {
                    (true, true) => std::cmp::Ordering::Equal,
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, true) => std::cmp::Ordering::Less,
                    (false, false) => match direction {
                        SortDirection::Ascending => a.compare(b),
                        SortDirection::Descending => a.compare(b).reverse(),
                    },
                }
            })
            .find(|ordering| ordering.is_ne())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    order
}

/// The sorted columns, keyed by header text so reordering can't move them.
/// With duplicate headers the first wins; entries naming no sortable header are
/// skipped. Without `multi`, only the first entry left sorts.
pub(super) fn active_sort(headers: &[HeaderSpec], sort: &[TableSort], multi: bool) -> ActiveSort {
    let mut active = ActiveSort::new();
    for sort in sort {
        let Some(index) = headers
            .iter()
            .position(|spec| spec.sortable && spec.header == sort.column)
        else {
            continue;
        };
        if !active.iter().any(|(column, _)| *column == index) {
            active.push((index, sort.direction));
        }
        if !multi {
            break;
        }
    }
    active
}

fn align_attr(align: CellAlign) -> Option<&'static str> {
    (align != CellAlign::Start).then(|| align.as_str())
}

/// The caption's text and the id the scroll region names itself by.
pub(super) struct CaptionSpec {
    pub text: String,
    pub id: String,
}

/// Everything inside the `<table>`, with `T` gone.
pub(super) struct BodySpec {
    pub caption: Option<CaptionSpec>,
    pub headers: Rc<Vec<HeaderSpec>>,
    /// The shown headers in display order; each row's cells come in it.
    pub order: Vec<usize>,
    pub rows: BodyRows,
    /// Drawn when no row shows.
    pub empty: EmptyBody,
    pub active: ActiveSort,
    pub sort: StateSlice<Vec<TableSort>>,
    /// Set with `multi_sort`: whether the press before a header click was a touch.
    pub touch: Option<CopyValue<bool>>,
    pub sort_order: &'static str,
    /// The select-all header cell, with `selectable`.
    pub select_all: Option<Element>,
    /// The detail toggles' header cell, with `row_detail`.
    pub detail_header: Option<Element>,
    /// With `column_menu`: each header's menu, by header index.
    pub menus: Vec<Element>,
    /// With `onrowreorder`: the leading column of handles, and its header cell.
    pub reorder: Option<RowReorder>,
    pub reorder_header: Option<Element>,
    /// With `header_filters`: each header's filter field, by header index.
    pub filters: Option<Vec<Option<Element>>>,
    /// With `resizable_columns`: what the headers' resize grips share.
    pub resize: Option<ColumnResize>,
    /// With `column_menu`, off Blitz: what the unpinned headers' drag grips share.
    pub drag: Option<ColumnDrag>,
    /// With a sticky header: told its height, so a focus scroll clears it.
    pub head_height: Option<EventHandler<f64>>,
}

/// Whether a header click adds its column to the others: a modifier, or a touch,
/// which has none. The touch mark is spent by the click.
fn adds_column(touch: Option<CopyValue<bool>>, event: &MouseEvent) -> bool {
    let Some(mut touch) = touch else {
        return false;
    };
    let touched = *touch.read();
    touch.set(false);
    let modifiers = event.modifiers();
    touched || modifiers.shift() || modifiers.ctrl() || modifiers.meta()
}

/// `share` stands in for an unsized column's width.
fn header_style(spec: &HeaderSpec, share: Option<&str>) -> Option<String> {
    let style = spec.style.as_deref().or(share);
    let Some(pin) = &spec.pin else {
        return style.map(str::to_string);
    };
    // An overflowing table shrinks its columns below `width`, which the next
    // pinned column's inset counts on.
    let min_width = spec
        .width
        .as_ref()
        .map(|width| format!("min-width:{width};"))
        .unwrap_or_default();
    Some(format!(
        "{}{min_width}{}",
        style.unwrap_or_default(),
        pin.style()
    ))
}

fn header_body(spec: &HeaderSpec) -> Element {
    match &spec.body {
        Some(body) => body.clone(),
        None => rsx! { "{spec.header}" },
    }
}

pub(super) fn render_body(body: BodySpec) -> Element {
    let BodySpec {
        caption,
        headers,
        order: shown,
        rows,
        empty,
        active,
        sort,
        touch,
        sort_order,
        select_all,
        detail_header,
        menus,
        reorder,
        reorder_header,
        filters,
        resize,
        drag,
        head_height,
    } = body;
    let columns = shown.len().max(1)
        + usize::from(select_all.is_some())
        + usize::from(detail_header.is_some())
        + usize::from(reorder.is_some());
    let empty = rows.is_empty().then_some(empty);
    let windowed = matches!(rows, BodyRows::Window(_));
    let leads = (
        reorder.is_some(),
        detail_header.is_some(),
        select_all.is_some(),
    );
    // Blitz sizes columns by content even in fixed layout: an unsized one gets
    // an even share, so rows scrolling in can't widen it.
    let share = windowed.then(|| format!("width: {}%", 100.0 / shown.len().max(1) as f64));
    // The order shows only when it tells something: with two or more sorted columns.
    let ranked = active.len() > 1;
    let active = Rc::new(active);
    let context = SortContext {
        active: active.clone(),
        headers: headers.clone(),
        sort,
        touch,
        ranked,
        sort_order,
    };
    let with_menu = !menus.is_empty();
    let mut menus = menus.into_iter().map(Some).collect::<Vec<_>>();
    let mut menu_of = move |index: usize| menus.get_mut(index).and_then(Option::take);
    let paths: Vec<(usize, &[String])> = shown
        .iter()
        .map(|&index| (index, headers[index].groups.as_slice()))
        .collect();
    let mut select_all = select_all;
    let mut detail_header = detail_header;
    let mut reorder_header = reorder_header;
    let head_rows: Vec<Element> = header_rows(&paths, |at| pin_edge_at(&headers, &shown, at))
        .into_iter()
        .enumerate()
        .map(|(level, cells)| {
            let cells = cells
                .into_iter()
                .enumerate()
                .map(|(position, cell)| match cell {
                    HeaderCell::Group { name, start, span } => {
                        // Groups split at a pin edge: one over pinned columns sticks with them.
                        let pin = span_pin(&headers, &shown[start..start + span]);
                        rsx! {
                            th {
                                key: "g{position}",
                                scope: "colgroup",
                                colspan: (span > 1).then(|| span.to_string()),
                                "data-group": true,
                                "data-pin": pin.as_ref().map(|pin| pin.side.as_str()),
                                "data-pin-edge": pin.as_ref().filter(|pin| pin.edge).map(|_| true),
                                style: pin.as_ref().map(CellPin::style),
                                "{name}"
                            }
                        }
                    }
                    HeaderCell::Column { index, rowspan } => {
                        let spec = &headers[index];
                        let grip = resize.zip(spec.resize).map(|(resize, limits)| {
                            rsx! {
                                ResizeHandle {
                                    index,
                                    header: spec.header.clone(),
                                    limits,
                                    resize,
                                }
                            }
                        });
                        let resizable = grip.is_some();
                        let draggable = drag.is_some() && spec.pin.is_none();
                        // A pinned column moves by its pin.
                        let mover = drag.filter(|_| spec.pin.is_none()).map(|drag| {
                            rsx! {
                                ColumnDragGrip { index, header: spec.header.clone(), drag }
                            }
                        });
                        rsx! {
                            th {
                                key: "{index}",
                                scope: "col",
                                rowspan: (rowspan > 1).then(|| rowspan.to_string()),
                                "data-align": align_attr(spec.align),
                                "data-sortable": spec.sortable.then_some(true),
                                "data-menu": with_menu.then_some(true),
                                "data-resizable": resizable.then_some(true),
                                "data-draggable": draggable.then_some(true),
                                "data-pin": spec.pin.as_ref().map(|pin| pin.side.as_str()),
                                "data-pin-edge": spec.pin.as_ref().filter(|pin| pin.edge).map(|_| true),
                                // Else the menu button's label joins the name every cell reads out.
                                aria_label: with_menu.then(|| spec.header.clone()),
                                style: header_style(spec, share.as_deref()),
                                // On the sorted columns only (APG): a "none" on every
                                // other one is read out as "not sorted" at each.
                                aria_sort: active
                                    .iter()
                                    .find(|(column, _)| *column == index)
                                    .map(|(_, direction)| direction.aria_value()),
                                {mover}
                                {header_cell(spec, index, &context, menu_of(index))}
                                {grip}
                            }
                        }
                    }
                });
            let cells: Vec<Element> = cells.collect();
            rsx! {
                tr { key: "{level}", aria_rowindex: windowed.then(|| (level + 1).to_string()),
                    {reorder_header.take()}
                    {detail_header.take()}
                    {select_all.take()}
                    {cells.into_iter()}
                }
            }
        })
        .collect();
    // Under the header rows, so it counts after them.
    let filter_row = filters.map(|cells| {
        let (reorder, detail, select) = leads;
        let rowindex = windowed.then_some(head_rows.len() + 1);
        filter_row(&headers, &shown, cells, reorder, detail, select, rowindex)
    });
    let head_levels = head_rows.len() + usize::from(filter_row.is_some());
    rsx! {
        if let Some(spec) = caption {
            caption { id: spec.id, "{spec.text}" }
        }
        thead {
            onresize: move |event: Event<ResizeData>| {
                if let (Some(told), Ok(size)) = (head_height, event.get_border_box_size()) {
                    told.call(size.height);
                }
            },
            {head_rows.into_iter()}
            {filter_row}
        }
        {
            let body = rsx! {
                {empty.map(|empty| empty.render(columns, windowed.then_some(head_levels + 1)))}
                match rows {
                    BodyRows::All(rows) => rsx! {
                        {rows.into_iter().flat_map(|row| body_rows(row, &headers, columns, reorder.as_ref()))}
                    },
                    BodyRows::Window(window) => render_window(window, headers, head_levels),
                }
            };
            match &reorder {
                Some(reorder) => reorder.body(body),
                None => rsx! {
                    tbody { {body} }
                },
            }
        }
    }
}

/// A row, then its detail row while open: siblings, each keyed.
pub(super) fn body_rows(
    row: RowSpec,
    headers: &[HeaderSpec],
    columns: usize,
    reorder: Option<&RowReorder>,
) -> impl Iterator<Item = Element> {
    let RowSpec {
        key,
        states,
        attributes,
        cells,
        selected,
        select,
        stripe,
        toggle,
        detail,
        sliding,
        reorder: slot,
    } = row;
    let content = rsx! {
            {toggle}
            {select}
            for CellSpec { column , text , body , span , pin } in cells {
                if headers[column].row_header {
                    th {
                        key: "{column}",
                        scope: "row",
                        colspan: (span > 1).then(|| span.to_string()),
                        "data-align": align_attr(headers[column].align),
                        "data-pin": pin.as_ref().map(|pin| pin.side.as_str()),
                        "data-pin-edge": pin.as_ref().filter(|pin| pin.edge).map(|_| true),
                        style: pin.as_ref().map(CellPin::style),
                        "{text}"
                        {body}
                    }
                } else {
                    td {
                        key: "{column}",
                        colspan: (span > 1).then(|| span.to_string()),
                        "data-align": align_attr(headers[column].align),
                        "data-pin": pin.as_ref().map(|pin| pin.side.as_str()),
                        "data-pin-edge": pin.as_ref().filter(|pin| pin.edge).map(|_| true),
                        style: pin.as_ref().map(CellPin::style),
                        // Text inline: a nested node per cell costs ~1 us a sort.
                        "{text}"
                        {body}
                    }
                }
            }
    };
    let (row, detail) = match (reorder, slot) {
        // The row draws its detail row: it moves with the row.
        (Some(reorder), Some(ReorderSlot { slot, label })) => (
            rsx! {
                ReorderRow {
                    key: "{key}",
                    slot,
                    label,
                    disabled: reorder.disabled,
                    instructions: reorder.instructions.clone(),
                    states,
                    stripe,
                    selected,
                    attributes,
                    detail,
                    columns,
                    {content}
                }
            },
            None,
        ),
        _ => (
            rsx! {
            tr {
                key: "{key}",
                "data-state": states,
                "data-stripe": stripe.then_some(true),
                aria_selected: selected.map(|selected| selected.to_string()),
                ..attributes,
                {content}
            }
            },
            match sliding {
                Some((id, open)) => Some(rsx! {
                    SlidingDetail {
                        key: "{key}-detail",
                        id,
                        open,
                        body: detail.map(|(_, body)| body),
                        columns,
                    }
                }),
                None => detail.map(|(id, body)| {
                    rsx! {
                        tr { key: "{key}-detail", id, "data-detail": true,
                            td { colspan: "{columns}", {body} }
                        }
                    }
                }),
            },
        ),
    };
    std::iter::once(row).chain(detail)
}

/// What a header's sort button needs, shared by every header.
#[derive(Clone)]
struct SortContext {
    active: Rc<ActiveSort>,
    headers: Rc<Vec<HeaderSpec>>,
    sort: StateSlice<Vec<TableSort>>,
    touch: Option<CopyValue<bool>>,
    /// Whether the order badges show: two or more sorted columns.
    ranked: bool,
    sort_order: &'static str,
}

/// A header cell's body: the sort button or the text, then the column menu.
fn header_cell(
    spec: &HeaderSpec,
    index: usize,
    context: &SortContext,
    menu: Option<Element>,
) -> Element {
    let label = match spec.sortable {
        true => sort_button(spec, index, context.clone()),
        false => header_body(spec),
    };
    match menu {
        Some(menu) => {
            let label = match spec.sortable {
                true => label,
                false => rsx! {
                    span { "data-header-text": true, {label} }
                },
            };
            // An end-aligned header keeps its text over its cells: the menu goes
            // first, in the DOM too, so Tab follows what is seen (todo 1261).
            match spec.align {
                CellAlign::End => rsx! {
                    div { "data-header": true, {menu} {label} }
                },
                _ => rsx! {
                    div { "data-header": true, {label} {menu} }
                },
            }
        }
        None => label,
    }
}

fn sort_button(spec: &HeaderSpec, index: usize, context: SortContext) -> Element {
    let SortContext {
        active,
        headers,
        sort,
        touch,
        ranked,
        sort_order,
    } = context;
    let order = active
        .iter()
        .position(|(column, _)| *column == index)
        .filter(|_| ranked)
        .map(|order| order + 1);
    rsx! {
        button {
            r#type: "button",
            "data-sort-button": true,
            onpointerdown: move |event: PointerEvent| {
                if let Some(mut touch) = touch {
                    touch.set(event.pointer_type() != "mouse");
                }
            },
            onclick: move |event: MouseEvent| {
                let add = adds_column(touch, &event);
                sort.set(next_sort(&active, &headers, index, add));
            },
            {header_body(spec)}
            // Always rendered, so sorting can't change the width;
            // `aria-sort` shows and flips it.
            Glyph { slot: IconSlot::ArrowDown, icon: lucide::arrow_down::outlined }
            if let Some(order) = order {
                span { "data-sort-order": true, aria_hidden: "true", "{order}" }
                VisuallyHidden { {fill(sort_order, &[("n", &order)])} }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::column::column, *};
    use SortDirection::{Ascending, Descending};

    fn keys(values: &[Option<f64>]) -> Vec<SortKey> {
        values
            .iter()
            .map(|value| value.map_or(SortKey::Empty, SortKey::num))
            .collect()
    }

    fn sorted(keys: &[SortKey], direction: SortDirection) -> Vec<usize> {
        sorted_order(&[(keys.to_vec(), direction)])
    }

    #[test]
    fn sorts_numbers_both_ways() {
        let keys = keys(&[Some(3.0), Some(1.0), Some(2.0)]);

        assert_eq!(sorted(&keys, Ascending), vec![1, 2, 0]);
        assert_eq!(sorted(&keys, Descending), vec![0, 2, 1]);
    }

    #[test]
    fn empty_keys_sink_in_both_directions() {
        let keys = keys(&[None, Some(2.0), Some(1.0)]);

        assert_eq!(sorted(&keys, Ascending), vec![2, 1, 0]);
        assert_eq!(sorted(&keys, Descending), vec![1, 2, 0]);
    }

    #[test]
    fn equal_keys_keep_source_order() {
        let keys = keys(&[Some(1.0), Some(1.0), Some(1.0)]);

        assert_eq!(sorted(&keys, Ascending), vec![0, 1, 2]);
        assert_eq!(sorted(&keys, Descending), vec![0, 1, 2]);
    }

    #[test]
    fn a_second_column_breaks_ties_of_the_first() {
        let first = keys(&[Some(1.0), Some(2.0), Some(1.0), Some(2.0)]);
        let second = keys(&[Some(5.0), Some(6.0), Some(7.0), None]);

        let order = sorted_order(&[(first.clone(), Ascending), (second.clone(), Descending)]);
        assert_eq!(order, vec![2, 0, 1, 3]);
        let order = sorted_order(&[(first, Descending), (second, Ascending)]);
        assert_eq!(order, vec![1, 3, 0, 2]);
    }

    #[test]
    fn the_sorted_order_is_reused_until_rows_columns_or_sort_change() {
        use std::cell::Cell;

        let calls = Rc::new(Cell::new(0));
        let counted = calls.clone();
        let columns = vec![column("N").value(move |n: &u32| {
            counted.set(counted.get() + 1);
            *n
        })];
        let mut sorted = SortedRows::default();
        let data = Rc::new(vec![3, 1, 2]);
        let up = vec![(0, Ascending)];

        assert_eq!(sorted.order(&data, &columns, &up), vec![1, 2, 0]);
        let first = calls.get();
        // An equal copy of the rows, as each render's props are.
        assert_eq!(
            sorted.order(&Rc::new(vec![3, 1, 2]), &columns, &up),
            vec![1, 2, 0]
        );
        assert_eq!(calls.get(), first);

        assert_eq!(
            sorted.order(&data, &columns, &vec![(0, Descending)]),
            vec![0, 2, 1]
        );
        assert_eq!(
            sorted.order(&Rc::new(vec![0, 1, 2]), &columns, &up),
            vec![0, 1, 2]
        );
        assert_eq!(calls.get(), first * 3);
    }

    fn headers(names: &[&str]) -> Vec<HeaderSpec> {
        names
            .iter()
            .map(|name| HeaderSpec {
                header: name.to_string(),
                align: CellAlign::Start,
                sortable: true,
                row_header: false,
                hideable: true,
                hidden: false,
                width: None,
                style: None,
                body: None,
                groups: Vec::new(),
                pin: None,
                resize: None,
            })
            .collect()
    }

    static NO_WIDTHS: ColumnWidths = ColumnWidths::new();

    fn unsized_spec() -> WidthSpec<'static> {
        WidthSpec {
            resizable: false,
            widths: &NO_WIDTHS,
            preview: None,
        }
    }

    #[test]
    fn a_resized_width_wins_and_a_drag_previews_over_it() {
        let (_, mut columns) = ages();
        columns[0] = columns[0].clone().width("10rem");
        columns[1] = columns[1].clone().resizable(false);
        let widths = ColumnWidths::from([("Name".to_string(), 120.0)]);
        let spec = |preview| WidthSpec {
            resizable: true,
            widths: &widths,
            preview,
        };

        let headers = header_specs(&columns, &ColumnDefaults::new(), &[], spec(None));
        assert_eq!(headers[0].width.as_deref(), Some("120px"));
        assert_eq!(headers[0].resize, Some(ResizeLimits::default()));
        assert_eq!(headers[1].resize, None);

        let headers = header_specs(&columns, &ColumnDefaults::new(), &[], spec(Some((0, 90.0))));
        assert_eq!(headers[0].width.as_deref(), Some("90px"));
        assert_eq!(
            headers[0].style.as_deref(),
            Some("box-sizing:border-box;width:90px;")
        );
    }

    #[test]
    fn the_sort_follows_its_header_not_its_position() {
        let sort = [TableSort::new("Age", Descending)];

        let active = active_sort(&headers(&["Name", "Age"]), &sort, false);
        assert_eq!(active, [(1, Descending)]);

        let reordered = active_sort(&headers(&["Age", "Name"]), &sort, false);
        assert_eq!(reordered, [(0, Descending)]);

        assert!(active_sort(&headers(&["Name"]), &sort, false).is_empty());
    }

    #[test]
    fn duplicate_headers_sort_the_first_sortable_match() {
        let sort = [TableSort::new("Age", Ascending)];
        let mut specs = headers(&["Age", "Name", "Age"]);

        assert_eq!(active_sort(&specs, &sort, false), [(0, Ascending)]);
        specs[0].sortable = false;
        assert_eq!(active_sort(&specs, &sort, false), [(2, Ascending)]);
    }

    #[test]
    fn the_first_entry_naming_a_sortable_header_sorts() {
        let sort = [
            TableSort::new("Height", Ascending),
            TableSort::new("Name", Descending),
            TableSort::new("Age", Ascending),
        ];

        let active = active_sort(&headers(&["Age", "Name"]), &sort, false);
        assert_eq!(active, [(1, Descending)]);
        assert!(active_sort(&headers(&["Age"]), &[], false).is_empty());
    }

    #[test]
    fn multi_keeps_every_named_column_once_in_order() {
        let sort = [
            TableSort::new("Name", Descending),
            TableSort::new("Height", Ascending),
            TableSort::new("Age", Ascending),
            TableSort::new("Name", Ascending),
        ];

        let active = active_sort(&headers(&["Age", "Name"]), &sort, true);
        assert_eq!(active, [(1, Descending), (0, Ascending)]);
    }

    #[test]
    fn text_sorts_case_insensitively() {
        let keys = vec![SortKey::text("beta"), SortKey::text("Alpha")];

        assert_eq!(sorted(&keys, Ascending), vec![1, 0]);
    }

    #[test]
    fn a_header_click_cycles_ascending_descending_unsorted() {
        let specs = headers(&["Name", "Age"]);
        let asc = next_sort(&[], &specs, 1, false);
        assert_eq!(asc, [TableSort::new("Age", Ascending)]);

        let desc = next_sort(&[(1, Ascending)], &specs, 1, false);
        assert_eq!(desc, [TableSort::new("Age", Descending)]);

        assert!(next_sort(&[(1, Descending)], &specs, 1, false).is_empty());
    }

    #[test]
    fn another_header_starts_ascending() {
        let next = next_sort(&[(0, Descending)], &headers(&["Name", "Age"]), 1, false);

        assert_eq!(next, [TableSort::new("Age", Ascending)]);
    }

    #[test]
    fn a_plain_click_on_a_secondary_column_sorts_by_it_alone() {
        let next = next_sort(
            &[(0, Ascending), (1, Ascending)],
            &headers(&["Name", "Age"]),
            1,
            false,
        );

        assert_eq!(next, [TableSort::new("Age", Descending)]);
    }

    #[test]
    fn an_added_click_appends_cycles_and_removes_in_place() {
        let specs = headers(&["Name", "Age", "City"]);

        let appended = next_sort(&[(0, Descending)], &specs, 2, true);
        assert_eq!(
            appended,
            [
                TableSort::new("Name", Descending),
                TableSort::new("City", Ascending)
            ]
        );
        let flipped = next_sort(&[(0, Ascending), (2, Ascending)], &specs, 0, true);
        assert_eq!(
            flipped,
            [
                TableSort::new("Name", Descending),
                TableSort::new("City", Ascending)
            ]
        );
        let removed = next_sort(&[(0, Descending), (2, Ascending)], &specs, 0, true);
        assert_eq!(removed, [TableSort::new("City", Ascending)]);
    }

    fn ages() -> (Vec<u32>, Vec<Column<u32>>) {
        let columns = vec![
            column("Name").value(|age: &u32| format!("n{age}")),
            column("Age").value(|age: &u32| *age).sortable(),
        ];
        (vec![30, 10, 20], columns)
    }

    #[test]
    fn rows_keep_source_order_until_sorted() {
        let (data, columns) = ages();

        assert_eq!(row_order(&data, &columns, &[]), vec![0, 1, 2]);
        assert_eq!(row_order(&data, &columns, &[(1, Ascending)]), vec![1, 2, 0]);
    }

    #[test]
    fn a_sort_on_an_unknown_or_unsortable_header_is_inactive() {
        let (_, columns) = ages();
        let headers = header_specs(&columns, &ColumnDefaults::new(), &[], unsized_spec());

        let unknown = [TableSort::new("Height", Ascending)];
        assert!(active_sort(&headers, &unknown, true).is_empty());
        let unsortable = [TableSort::new("Name", Ascending)];
        assert!(active_sort(&headers, &unsortable, true).is_empty());
    }

    #[test]
    fn widths_become_the_header_cells_style() {
        assert_eq!(width_style(None, None), None);
        assert_eq!(
            width_style(Some("6rem"), None).as_deref(),
            Some("box-sizing:border-box;width:6rem;")
        );
        assert_eq!(
            width_style(Some("50%"), Some("4rem")).as_deref(),
            Some("box-sizing:border-box;width:50%;min-width:4rem;")
        );
    }
}
