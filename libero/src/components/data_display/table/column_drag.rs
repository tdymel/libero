use std::collections::BTreeMap;

use dioxus::{
    core::{ScopeId, current_scope_id},
    prelude::*,
};
use pictogram_icons_lucide as lucide;

use super::{column_order::dropped, use_table::StateSlice};
use crate::{
    components::common::Glyph,
    context::IconSlot,
    hooks::{
        DragMove, DragOptions, DragStart, ElementHandle, use_drag, use_element, use_escape_dismiss,
    },
    platform::{ElementApi, PlatformError},
};

/// A box in client coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Rect {
    fn right(&self) -> f64 {
        self.x + self.width
    }

    fn bottom(&self) -> f64 {
        self.y + self.height
    }

    fn contains(&self, x: f64, y: f64) -> bool {
        (self.x..=self.right()).contains(&x) && (self.y..=self.bottom()).contains(&y)
    }

    /// The overlap with `other`, or `self` when they don't meet.
    fn clip(self, other: Rect) -> Rect {
        let (x, y) = (self.x.max(other.x), self.y.max(other.y));
        let (right, bottom) = (
            self.right().min(other.right()),
            self.bottom().min(other.bottom()),
        );
        match right > x && bottom > y {
            true => Rect {
                x,
                y,
                width: right - x,
                height: bottom - y,
            },
            false => self,
        }
    }
}

/// What a drop needs, refreshed by the table each render.
#[derive(Clone, Default, PartialEq)]
pub(super) struct DropPlan {
    /// Every header in its current rank, as `oncolumnorderchange` lists them.
    pub ranked: Vec<String>,
    /// The shown, unpinned columns in display order, by header index.
    pub unpinned: Vec<(usize, String)>,
}

/// A drag under way: the measured layout, once read, and the pointer.
#[derive(Clone, Copy, PartialEq)]
struct Dragging {
    index: usize,
    start_x: f64,
    x: f64,
    gap: Option<usize>,
}

/// The layout a drag measured at its start.
#[derive(Clone, Default, PartialEq)]
struct Geometry {
    cell: Rect,
    /// The table, cut to its scroll region: where a drop counts.
    area: Rect,
    /// Each gap's x, gap `n` before the `n`th unpinned column; `None` out of view.
    gaps: Vec<Option<f64>>,
}

/// A table's column drag state, shared by its header grips.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct ColumnDrag {
    pub order: StateSlice<Vec<String>>,
    pub plan: CopyValue<DropPlan>,
    pub table: ElementHandle,
    pub region: ElementHandle,
    /// The header cells' boxes by header index, owned by the table: every grip reads them all.
    boxes: CopyValue<BTreeMap<usize, ElementHandle>>,
    owner: ScopeId,
    dragging: Signal<Option<Dragging>>,
    geometry: CopyValue<Option<Geometry>>,
}

pub(super) fn use_column_drag(order: StateSlice<Vec<String>>) -> ColumnDrag {
    ColumnDrag {
        order,
        plan: use_hook(|| CopyValue::new(DropPlan::default())),
        table: use_element(),
        region: use_element(),
        boxes: use_hook(|| CopyValue::new(BTreeMap::new())),
        owner: current_scope_id(),
        dragging: use_signal(|| None),
        geometry: use_hook(|| CopyValue::new(None)),
    }
}

/// Gap `n` of columns laid out by `edges` (start, end) in display order sits
/// at the `n`th column's start edge, the last gap at the last column's end.
fn gap_edges(edges: &[(f64, f64)]) -> Vec<f64> {
    let mut gaps: Vec<f64> = edges.iter().map(|(start, _)| *start).collect();
    gaps.extend(edges.last().map(|(_, end)| *end));
    gaps
}

/// The gap in view nearest `x`.
fn nearest_gap(gaps: &[Option<f64>], x: f64) -> Option<usize> {
    (0..gaps.len())
        .filter_map(|gap| gaps[gap].map(|at| (gap, (at - x).abs())))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(gap, _)| gap)
}

/// Starts reading `element`'s client box, in the event handler, as Blitz needs.
fn read_rect(
    element: &dyn ElementApi,
) -> impl Future<Output = Result<Rect, PlatformError>> + use<> {
    let (offset, size) = (element.client_offset(), element.dimensions());
    async move {
        let ((x, y), size) = (offset.await?, size.await?);
        Ok(Rect {
            x,
            y,
            width: size.width,
            height: size.height,
        })
    }
}

/// A header's column drag grip at its inline start, with the ghost header and
/// the drop line while it drags. Pointer only: the column menu's Move left and
/// Move right are the keyboard and drag-free way (WCAG 2.5.7).
#[component]
pub(super) fn ColumnDragGrip(index: usize, header: String, drag: ColumnDrag) -> Element {
    // Fills the header cell, to measure it. Kept per index, a hidden column's unread.
    let mut boxes = drag.boxes;
    let cell = *boxes
        .write()
        .entry(index)
        .or_insert_with(|| ElementHandle::new_in_scope(drag.owner));
    let grip = use_element();
    let mut dragging = drag.dragging;
    let mut geometry = drag.geometry;
    let cancel = use_callback(move |()| {
        dragging.set(None);
        geometry.set(None);
    });
    let ours = dragging.read().is_some_and(|active| active.index == index);
    let _ = use_escape_dismiss(ours, true, cancel);
    let end_header = header.clone();
    let handle = use_drag(DragOptions {
        capture: grip,
        onstart: Callback::new(move |start: DragStart| {
            let plan = drag.plan.peek().clone();
            let rtl = cell.is_rtl();
            let boxes = drag.boxes.peek().clone();
            let columns: Vec<_> = plan
                .unpinned
                .iter()
                .filter_map(|(at, _)| boxes.get(at).map(|element| read_rect(element)))
                .collect();
            let Some(own) = boxes.get(&index).map(|element| read_rect(element)) else {
                start.cancel.call(());
                return;
            };
            if columns.len() != plan.unpinned.len() {
                start.cancel.call(());
                return;
            }
            let (table, region) = (read_rect(&drag.table), read_rect(&drag.region));
            dragging.set(Some(Dragging {
                index,
                start_x: start.client.x,
                x: start.client.x,
                gap: None,
            }));
            spawn(async move {
                let mut edges = Vec::with_capacity(columns.len());
                for column in columns {
                    match column.await {
                        Ok(rect) => edges.push(match rtl {
                            true => (rect.right(), rect.x),
                            false => (rect.x, rect.right()),
                        }),
                        Err(_) => return start.cancel.call(()),
                    }
                }
                let (Ok(cell), Ok(table)) = (own.await, table.await) else {
                    return start.cancel.call(());
                };
                let area = match region.await {
                    Ok(region) => table.clip(region),
                    Err(_) => table,
                };
                // Out of the region's view, a gap is no target.
                let gaps = gap_edges(&edges)
                    .into_iter()
                    .map(|x| (area.x..=area.right()).contains(&x).then_some(x))
                    .collect();
                // Still ours: an Escape or a release may have come first.
                if dragging.peek().is_some_and(|active| active.index == index) {
                    geometry.set(Some(Geometry { cell, area, gaps }));
                }
            });
        }),
        onmove: Callback::new(move |step: DragMove| {
            let Some(mut active) = *dragging.peek() else {
                return;
            };
            if active.index != index {
                return;
            }
            active.x = step.client.x;
            // Outside the table a drop cancels: no line.
            active.gap = geometry.peek().as_ref().and_then(|geometry| {
                geometry
                    .area
                    .contains(step.client.x, step.client.y)
                    .then(|| nearest_gap(&geometry.gaps, step.client.x))
                    .flatten()
            });
            dragging.set(Some(active));
        }),
        onend: Callback::new(move |()| {
            let active = dragging.take();
            geometry.set(None);
            let Some(Dragging {
                index: dragged,
                gap: Some(gap),
                ..
            }) = active
            else {
                return;
            };
            if dragged != index {
                return;
            }
            let plan = drag.plan.peek();
            let names: Vec<String> = plan.unpinned.iter().map(|(_, h)| h.clone()).collect();
            if let Some(next) = dropped(&plan.ranked, &names, &end_header, gap) {
                drag.order.set(next);
            }
        }),
    });
    let active = (*dragging.read()).filter(|active| active.index == index);
    let shown = geometry.read().clone().zip(active);
    let overlay = shown.map(|(geometry, active)| {
        let Geometry { cell, area, gaps } = geometry;
        let left = cell.x + active.x - active.start_x;
        let line = active.gap.and_then(|gap| gaps[gap]).map(|x| x - 1.0);
        rsx! {
            div {
                "data-drag-ghost": true,
                aria_hidden: "true",
                style: "left:{left}px;top:{cell.y}px;width:{cell.width}px;height:{cell.height}px;",
                "{header}"
            }
            if let Some(line) = line {
                div {
                    "data-drop-line": true,
                    aria_hidden: "true",
                    style: "left:{line}px;top:{area.y}px;height:{area.height}px;",
                }
            }
        }
    });
    let held = (handle.dragging)() && active.is_some();
    // Siblings: Blitz hits nothing under a `pointer-events: none` box.
    rsx! {
        div { "data-drag-box": true, aria_hidden: "true", onmounted: cell.mount() }
        div {
            "data-drag-handle": true,
            "data-dragging": held.then_some(true),
            aria_hidden: "true",
            onmounted: grip.mount(),
            onpointerdown: move |event| handle.onpointerdown.call(event),
            onpointermove: move |event| handle.onpointermove.call(event),
            onpointerup: move |event| handle.onpointerup.call(event),
            onpointercancel: move |event| handle.onpointercancel.call(event),
            Glyph { slot: IconSlot::Grip, icon: lucide::grip_vertical::outlined }
        }
        {overlay}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gaps_sit_at_each_start_edge_and_after_the_last_column() {
        assert_eq!(
            gap_edges(&[(0.0, 50.0), (50.0, 120.0)]),
            vec![0.0, 50.0, 120.0]
        );
        // Right to left: starts on the right.
        assert_eq!(
            gap_edges(&[(120.0, 70.0), (70.0, 0.0)]),
            vec![120.0, 70.0, 0.0]
        );
        assert!(gap_edges(&[]).is_empty());
    }

    #[test]
    fn the_nearest_gap_in_view_wins() {
        let gaps = [Some(0.0), Some(50.0), Some(120.0)];

        assert_eq!(nearest_gap(&gaps, 20.0), Some(0));
        assert_eq!(nearest_gap(&gaps, 90.0), Some(2));
        assert_eq!(nearest_gap(&[Some(0.0), None], 90.0), Some(0));
        assert_eq!(nearest_gap(&[], 90.0), None);
    }

    #[test]
    fn the_drop_area_is_the_table_cut_to_its_region() {
        let table = Rect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 1000.0,
        };
        let region = Rect {
            x: 10.0,
            y: 20.0,
            width: 300.0,
            height: 200.0,
        };

        assert_eq!(table.clip(region), region);
        assert!(!table.clip(region).contains(5.0, 50.0));
    }
}
