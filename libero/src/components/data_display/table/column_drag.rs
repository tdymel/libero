use std::collections::BTreeMap;

use dioxus::{
    core::{ScopeId, current_scope_id},
    prelude::*,
};
use pictogram_icons_lucide as lucide;

use super::{
    column_order::{as_headers, dropped},
    use_table::StateSlice,
};
use crate::{
    components::{accessibility::Announcer, common::Glyph},
    context::IconSlot,
    hooks::{
        DragMove, DragOptions, DragStart, ElementHandle, edge_scroll_step, use_drag_cancel,
        use_element, use_escape_dismiss, use_interval, use_localization, use_subscription_slot,
    },
    platform::{ElementApi, PlatformError, ScrollSubscription, scroll},
    utils::bump,
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
    /// Every column's key and header in its current rank, as `oncolumnorderchange` lists them.
    pub ranked: Vec<(String, String)>,
    /// The shown, unpinned columns in display order: header index and key.
    pub unpinned: Vec<(usize, String)>,
    /// The shown columns pinned before them, and all shown: a drop says its place.
    pub lead: usize,
    pub shown: usize,
}

impl DropPlan {
    /// Column `key` dropped at gap `gap` of the unpinned keys `over`: the header order and
    /// its place among the shown columns from 1, as the menu's Move says it. `None` when
    /// nothing moves or namesakes would pass one another, which the header order cannot hold.
    fn drop(&self, over: &[String], key: &str, gap: usize) -> Option<(Vec<String>, Option<usize>)> {
        let keys: Vec<String> = self.ranked.iter().map(|(key, _)| key.clone()).collect();
        let next = dropped(&keys, over, key, gap)?;
        let place = next
            .iter()
            .filter(|next| self.unpinned.iter().any(|(_, shown)| shown == *next))
            .position(|next| next == key)
            .map(|at| self.lead + at + 1);
        let headers = as_headers(self.ranked.iter().map(|(key, header)| (key, header)), &next)?;
        Some((headers, place))
    }
}

/// A drag under way: the measured layout, once read, and the pointer.
#[derive(Clone, Copy, PartialEq)]
struct Dragging {
    /// The dragging grip: its scope follows its column, an index shifts when the host's columns change.
    grip: ScopeId,
    start_x: f64,
    x: f64,
    y: f64,
    /// How far the region scrolled sideways since the start: the edge scroll's or a wheel's.
    scrolled: f64,
    gap: Option<usize>,
    /// The table's client offset the geometry follows: a page scroll redraws the ghost.
    origin: (f64, f64),
    /// Let go before the measure came back (the WebView reads over IPC): it drops on arrival.
    released: bool,
    /// The geometry arrived: the grip redraws with its ghost.
    measured: bool,
}

/// The layout a drag measured at its start, moved with the table since.
#[derive(Clone, Default, PartialEq)]
struct Geometry {
    /// The table's client offset the boxes below were read at.
    origin: (f64, f64),
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

    /// The boxes moved with the table, now at client offset `origin`: a page scroll mid-drag.
    fn follow(&mut self, origin: (f64, f64)) {
        let (dx, dy) = (origin.0 - self.origin.0, origin.1 - self.origin.1);
        self.origin = origin;
        for rect in [&mut self.cell, &mut self.area] {
            (rect.x, rect.y) = (rect.x + dx, rect.y + dy);
        }
        for gap in &mut self.gaps {
            *gap += dx;
        }
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
    /// The header cells' boxes by column key, owned by the table: every grip reads them all.
    boxes: CopyValue<BTreeMap<String, ElementHandle>>,
    owner: ScopeId,
    announcer: Announcer,
    dragging: Signal<Option<Dragging>>,
    geometry: CopyValue<Option<Geometry>>,
}

pub(super) fn use_column_drag(order: StateSlice<Vec<String>>, announcer: Announcer) -> ColumnDrag {
    ColumnDrag {
        order,
        plan: use_hook(|| CopyValue::new(DropPlan::default())),
        table: use_element(),
        region: use_element(),
        boxes: use_hook(|| CopyValue::new(BTreeMap::new())),
        owner: current_scope_id(),
        announcer,
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
pub(super) fn ColumnDragGrip(
    index: usize,
    header: String,
    /// The column's key: two columns may share a header.
    column: String,
    drag: ColumnDrag,
) -> Element {
    // Fills the header cell, to measure it. Kept per column, a hidden column's unread.
    let mut boxes = drag.boxes;
    let cell = *boxes
        .write()
        .entry(column.clone())
        .or_insert_with(|| ElementHandle::new_in_scope(drag.owner));
    let labels = use_localization().table;
    // The unpinned columns the drag measured its gaps over: a column the host adds or
    // removes meanwhile shifts the plan's.
    let mut measured_over = use_hook(|| CopyValue::new(Vec::<String>::new()));
    let grip = use_element();
    let scope = current_scope_id();
    let mut dragging = drag.dragging;
    let mut geometry = drag.geometry;
    let ours_now = move || (*dragging.peek()).filter(|active| active.grip == scope);
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
    // Through a memo: a move redraws the dragged grip only, not every header's (todo 2584).
    let mine = use_memo(move || dragging().filter(|active| active.grip == scope));
    let ours = mine.read().is_some();
    // A wheel or page scroll mid-drag moves the columns under the pointer: follow it, as Sortable does (2540).
    let scroll_tick = use_signal(|| 0_u64);
    let listening = use_subscription_slot::<dyn ScrollSubscription>();
    use_effect(move || {
        if !mine().is_some_and(|active| active.measured) {
            listening.clear();
            return;
        }
        if !listening.is_some()
            && let Some(api) = scroll()
        {
            listening.set(Some(api.on_scroll(Box::new(move || bump(scroll_tick)))));
        }
    });
    use_effect(move || {
        if scroll_tick() == 0 || ours_now().is_none() {
            return;
        }
        let (at, offset) = (drag.table.client_offset(), drag.region.scroll_offset());
        spawn(async move {
            let (Ok(at), Ok(offset)) = (at.await, offset.await) else {
                return;
            };
            let (Some(mut active), Some(mut measured)) = (ours_now(), geometry.peek().clone())
            else {
                return;
            };
            measured.follow(at);
            active.origin = at;
            if let Some(scroll) = measured.scroll {
                active.scrolled = offset.0 - scroll.from.0;
            }
            active.gap = measured.gap_at(active.x, active.y, active.scrolled);
            geometry.set(Some(measured));
            dragging.set(Some(active));
        });
    });
    let _ = use_escape_dismiss(ours, true, cancel);
    let (end_header, end_key) = (header.clone(), column.clone());
    let drop_at = use_callback(move |gap: usize| {
        let plan = drag.plan.peek();
        if let Some((next, place)) = plan.drop(&measured_over.peek(), &end_key, gap) {
            drag.order.set(next);
            if let Some(place) = place {
                drag.announcer
                    .say((labels.column_moved)(&end_header, place, plan.shown));
            }
        }
    });
    // The OS revoked the pointer: no reorder, as Escape; a measure still running finds it gone.
    let revoked = use_callback(move |()| {
        if (*dragging.peek()).is_some_and(|active| active.grip == scope) {
            cancel.call(());
        }
    });
    let key_start = column.clone();
    let options = DragOptions {
        capture: grip,
        onstart: Callback::new(move |start: DragStart| {
            let plan = drag.plan.peek().clone();
            let rtl = cell.is_rtl();
            let boxes = drag.boxes.peek().clone();
            let columns: Vec<_> = plan
                .unpinned
                .iter()
                .filter_map(|(_, key)| boxes.get(key).map(|element| read_rect(element)))
                .collect();
            measured_over.set(plan.unpinned.iter().map(|(_, key)| key.clone()).collect());
            let Some(own) = boxes.get(&key_start).map(|element| read_rect(element)) else {
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
                grip: scope,
                start_x: start.client.x,
                x: start.client.x,
                y: start.client.y,
                scrolled: 0.0,
                origin: (0.0, 0.0),
                gap: None,
                released: false,
                measured: false,
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
                let Some(mut active) = (*dragging.peek()).filter(|active| active.grip == scope)
                else {
                    return;
                };
                let measured = Geometry {
                    origin: (table.x, table.y),
                    cell,
                    area,
                    gaps,
                    scroll,
                };
                active.gap = measured.gap_at(active.x, active.y, 0.0);
                active.measured = true;
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
            if active.grip != scope {
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
            let Some(mut active) = (*dragging.peek()).filter(|active| active.grip == scope) else {
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
    };
    let handle = use_drag_cancel(options, revoked);
    let active = mine();
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
    use super::{
        super::column_order::{ranked, tests::headers},
        *,
    };

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

    fn plan(columns: &[(&str, &str)], lead: usize) -> DropPlan {
        let ranked: Vec<(String, String)> = columns
            .iter()
            .map(|(key, header)| (key.to_string(), header.to_string()))
            .collect();
        DropPlan {
            unpinned: ranked
                .iter()
                .enumerate()
                .map(|(index, (key, _))| (index, key.clone()))
                .collect(),
            shown: lead + ranked.len(),
            ranked,
            lead,
        }
    }

    fn keys(plan: &DropPlan) -> Vec<String> {
        plan.unpinned.iter().map(|(_, key)| key.clone()).collect()
    }

    /// 4409: a drop says the column's place among all shown, after the pinned lead.
    #[test]
    fn a_drop_names_its_place_after_the_pinned_lead() {
        let plan = plan(&[("A", "A"), ("B", "B"), ("C", "C")], 1);

        let (order, place) = plan.drop(&keys(&plan), "A", 3).unwrap();
        assert_eq!(order, ["B", "C", "A"]);
        assert_eq!(place, Some(4));
        assert_eq!(plan.drop(&keys(&plan), "A", 0), None);
    }

    /// 4417, 4428: two columns titled alike move by key, the other staying put; a drop
    /// passing one over the other is refused, as the header order cannot hold it.
    #[test]
    fn a_repeated_header_drops_by_its_key() {
        let specs = headers(&["Name", "Age", "Name"]);
        let named: Vec<_> = specs
            .iter()
            .map(|s| (s.key.as_str(), s.header.as_str()))
            .collect();
        let plan = plan(&named, 0);

        // The stored order ranks the columns as dropped.
        let (order, place) = plan.drop(&keys(&plan), "Name\u{1f}1", 1).unwrap();
        assert_eq!(ranked(&specs, &order), [0, 2, 1]);
        assert_eq!(place, Some(2));
        let (order, place) = plan.drop(&keys(&plan), "Name", 2).unwrap();
        assert_eq!(ranked(&specs, &order), [1, 0, 2]);
        assert_eq!(place, Some(2));
        assert_eq!(plan.drop(&keys(&plan), "Name\u{1f}1", 0), None);
        assert_eq!(plan.drop(&keys(&plan), "Name", 3), None);
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
