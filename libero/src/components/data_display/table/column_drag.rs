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
        DragMove, DragOptions, DragStart, ElementHandle, edge_scroll_step, use_drag, use_element,
        use_escape_dismiss, use_interval,
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
    y: f64,
    /// How far the region scrolled since the start, by the drag's own edge scroll.
    scrolled: f64,
    gap: Option<usize>,
    /// Let go before the measure came back (the WebView reads over IPC): it drops on arrival.
    released: bool,
}

/// The layout a drag measured at its start.
#[derive(Clone, Default, PartialEq)]
struct Geometry {
    cell: Rect,
    /// The table, cut to its scroll region: where a drop counts.
    area: Rect,
    /// Each gap's x at the start, gap `n` before the `n`th unpinned column.
    gaps: Vec<f64>,
    /// Set when the region scrolls sideways: a drag near its edge scrolls it.
    scroll: Option<Scroll>,
}

/// A region's `scrollLeft`, `scrollTop` at the drag's start, and the `scrollLeft` range.
#[derive(Clone, Copy, Default, PartialEq)]
struct Scroll {
    from: (f64, f64),
    range: (f64, f64),
}

impl Geometry {
    /// Gap `gap`'s x once the region scrolled by `scrolled`; `None` out of view.
    fn gap_x(&self, gap: usize, scrolled: f64) -> Option<f64> {
        let x = self.gaps[gap] - scrolled;
        // A px of slack: the last column's end sums to a hair past the table's.
        (self.area.x - 1.0..=self.area.right() + 1.0)
            .contains(&x)
            .then_some(x)
    }

    /// The gap a pointer at `x`, `y` points at; outside the table a drop cancels.
    fn gap_at(&self, x: f64, y: f64, scrolled: f64) -> Option<usize> {
        let gaps: Vec<_> = (0..self.gaps.len())
            .map(|gap| self.gap_x(gap, scrolled))
            .collect();
        self.area
            .contains(x, y)
            .then(|| nearest_gap(&gaps, x))
            .flatten()
    }

    /// Px per tick the region scrolls by with the pointer at `x`.
    fn edge_step(&self, x: f64) -> f64 {
        self.scroll
            .map_or(0.0, |_| edge_scroll_step(x, self.area.x, self.area.width))
    }
}

/// The edge scroll's tick.
const AUTO_SCROLL_MS: u64 = 40;

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
    let ours_now = move || (*dragging.peek()).filter(|active| active.index == index);
    let auto_scroll = use_interval(
        move || {
            let Some(active) = ours_now() else {
                return;
            };
            let Some((step, scroll)) = geometry
                .peek()
                .as_ref()
                .and_then(|geometry| Some((geometry.edge_step(active.x), geometry.scroll?)))
            else {
                return;
            };
            let now = scroll.from.0 + active.scrolled;
            let next = (now + step).clamp(scroll.range.0, scroll.range.1);
            if next == now {
                return;
            }
            let _ = drag.region.scroll_to(next, scroll.from.1);
            let at = drag.region.scroll_offset();
            spawn(async move {
                let (Ok((x, _)), Some(mut active)) = (at.await, ours_now()) else {
                    return;
                };
                active.scrolled = x - scroll.from.0;
                active.gap = geometry
                    .peek()
                    .as_ref()
                    .and_then(|geometry| geometry.gap_at(active.x, active.y, active.scrolled));
                dragging.set(Some(active));
            });
        },
        AUTO_SCROLL_MS,
    );
    let cancel = use_callback(move |()| {
        auto_scroll.stop();
        dragging.set(None);
        geometry.set(None);
    });
    let ours = dragging.read().is_some_and(|active| active.index == index);
    let _ = use_escape_dismiss(ours, true, cancel);
    let end_header = header.clone();
    let drop_at = use_callback(move |gap: usize| {
        let plan = drag.plan.peek();
        let names: Vec<String> = plan.unpinned.iter().map(|(_, h)| h.clone()).collect();
        if let Some(next) = dropped(&plan.ranked, &names, &end_header, gap) {
            drag.order.set(next);
        }
    });
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
            let (content, offset) = (drag.region.scroll_size(), drag.region.scroll_offset());
            dragging.set(Some(Dragging {
                index,
                start_x: start.client.x,
                x: start.client.x,
                y: start.client.y,
                scrolled: 0.0,
                gap: None,
                released: false,
            }));
            let fail = move || {
                start.cancel.call(());
                cancel.call(());
            };
            spawn(async move {
                let mut edges = Vec::with_capacity(columns.len());
                for column in columns {
                    match column.await {
                        Ok(rect) => edges.push(match rtl {
                            true => (rect.right(), rect.x),
                            false => (rect.x, rect.right()),
                        }),
                        Err(_) => return fail(),
                    }
                }
                let (Ok(cell), Ok(table)) = (own.await, table.await) else {
                    return fail();
                };
                let region = region.await.ok();
                let area = region.map_or(table, |region| table.clip(region));
                let (content, offset) = (content.await, offset.await);
                // Scrolls sideways: wider content than its box.
                let scroll = match (region, content, offset) {
                    (Some(region), Ok(content), Ok(from)) if content.width > region.width + 0.5 => {
                        let most = content.width - region.width;
                        // Right to left, `scrollLeft` runs from 0 down to minus the overflow.
                        let range = if rtl { (-most, 0.0) } else { (0.0, most) };
                        Some(Scroll { from, range })
                    }
                    _ => None,
                };
                let gaps = gap_edges(&edges);
                // Still ours: an Escape may have come first.
                let Some(mut active) = (*dragging.peek()).filter(|active| active.index == index)
                else {
                    return;
                };
                let measured = Geometry {
                    cell,
                    area,
                    gaps,
                    scroll,
                };
                active.gap = measured.gap_at(active.x, active.y, 0.0);
                if !active.released {
                    dragging.set(Some(active));
                    return geometry.set(Some(measured));
                }
                dragging.set(None);
                if let Some(gap) = active.gap {
                    drop_at.call(gap);
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
            (active.x, active.y) = (step.client.x, step.client.y);
            // Outside the table a drop cancels: no line.
            let (gap, step) = geometry.peek().as_ref().map_or((None, 0.0), |geometry| {
                (
                    geometry.gap_at(active.x, active.y, active.scrolled),
                    geometry.edge_step(active.x),
                )
            });
            active.gap = gap;
            dragging.set(Some(active));
            match (step != 0.0, auto_scroll.active()) {
                (true, false) => auto_scroll.start(),
                (false, true) => auto_scroll.stop(),
                _ => {}
            }
        }),
        onend: Callback::new(move |()| {
            let Some(mut active) = (*dragging.peek()).filter(|active| active.index == index) else {
                return;
            };
            // Unmeasured yet: the measure drops it where it was let go.
            if geometry.peek().is_none() {
                active.released = true;
                dragging.set(Some(active));
                return;
            }
            cancel.call(());
            if let Some(gap) = active.gap {
                drop_at.call(gap);
            }
        }),
    });
    let active = (*dragging.read()).filter(|active| active.index == index);
    let shown = geometry.read().clone().zip(active);
    let overlay = shown.map(|(geometry, active)| {
        let (cell, area) = (geometry.cell, geometry.area);
        let left = cell.x + active.x - active.start_x;
        let line = active
            .gap
            .and_then(|gap| geometry.gap_x(gap, active.scrolled))
            .map(|x| x - 1.0);
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
