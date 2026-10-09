use std::ops::Range;

use dioxus::{core::DynamicValues, prelude::*};

use super::viewport::{
    ContentOffsets, ScrollGeometry, ScrollViewport, Window, WindowSpec, probed_pitch,
};
use crate::{hooks::use_theme, platform::ElementApi, utils::warn};

/// Rows in the second probe render: enough to weigh the gap, cheap on a short list.
const PROBE_ROWS: usize = 8;

/// A zero came too early (still `display: contents`); each retry costs a frame,
/// giving up renders every row.
const PROBE_ATTEMPTS: usize = 8;

/// The row a `Virtualize` keeps out of its window and the slot its box sits at,
/// beside the window: for a row that translates itself to a slot (todo 1408).
#[derive(Clone, Copy)]
pub(crate) struct KeptSlot(pub Signal<Option<(usize, usize)>>);

/// The nearest `Virtualize`'s kept row, read inside one of its rows.
pub(crate) fn use_kept_slot() -> Option<Signal<Option<(usize, usize)>>> {
    try_use_context::<KeptSlot>().map(|kept| kept.0)
}

/// Rendered beside a windowed table's rows: the `ScrollArea` around the table then
/// reserves the skipped rows inside its body (todo 2201).
#[component]
pub(crate) fn RowsInTable() -> Element {
    let table = try_use_context::<ScrollViewport>().map(|viewport| viewport.table);
    use_effect(move || {
        if let Some(mut table) = table
            && !*table.peek()
        {
            table.set(true);
        }
    });
    use_drop(move || {
        if let Some(mut table) = table
            && let Ok(mut table) = table.try_write()
        {
            *table = false;
        }
    });
    VNode::empty()
}

/// How the row pitch is being arrived at.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Probe {
    /// One row, for its bare height.
    Single { attempt: usize },
    /// `PROBE_ROWS` rows: the height over the single row is `rows - 1` pitches.
    Batch { single: f64 },
    /// Given, measured, or given up on - `None` renders every row.
    Settled(Option<f64>),
    /// A measured `pitch` checked again in place after a resize (rows rewrap):
    /// the window's height first, then with rows added (todo 2496).
    Revise {
        pitch: f64,
        base: Option<(f64, (usize, usize))>,
    },
}

impl Probe {
    /// Rows to render while probing, or `None` once there is a pitch to use.
    fn rows(self, count: usize) -> Option<usize> {
        match self {
            Self::Single { .. } => Some(1.min(count)),
            Self::Batch { .. } => Some(PROBE_ROWS.min(count)),
            Self::Settled(_) | Self::Revise { .. } => None,
        }
    }

    /// The pitch the window uses, while settled or revising.
    fn pitch(self) -> Option<f64> {
        match self {
            Self::Settled(pitch) => pitch,
            Self::Revise { pitch, .. } => Some(pitch),
            _ => None,
        }
    }

    /// Still measuring, so the measuring effect runs again.
    fn measuring(self, count: usize) -> bool {
        self.rows(count).is_some() || matches!(self, Self::Revise { .. })
    }

    /// The rows rendered for the second revise measurement, while the window
    /// is still `window`; else the window. Growing never clamps the offset.
    fn revise_rows(self, window: Range<usize>, count: usize) -> Range<usize> {
        let Self::Revise {
            base: Some((_, at)),
            ..
        } = self
        else {
            return window;
        };
        if at != (window.start, window.end) {
            return window;
        }
        let more = PROBE_ROWS - 1;
        match (count - window.end, window.start) {
            (after, _) if after > 0 => window.start..window.end + more.min(after),
            (_, before) if before > 0 => window.start - more.min(before)..window.end,
            // The whole list is in the window: weigh it against fewer rows.
            _ => window.start..window.end - more.min(window.len().saturating_sub(1)),
        }
    }

    /// What a revise measurement of `height` makes of this stage, `window`
    /// being the window it was rendered for.
    fn revised(self, height: f64, window: (usize, usize), count: usize) -> Self {
        let Self::Revise { pitch, base } = self else {
            return self;
        };
        if height <= 0.0 {
            return Self::Settled(Some(pitch));
        }
        let Some((before, at)) = base else {
            return Self::Revise {
                pitch,
                base: Some((height, window)),
            };
        };
        // Scrolled in between: the padding moved, so start over.
        if at != window {
            return Self::Revise { pitch, base: None };
        }
        let rows = self.revise_rows(at.0..at.1, count).len();
        let base_rows = at.1 - at.0;
        let revised = match rows.cmp(&base_rows) {
            std::cmp::Ordering::Greater => probed_pitch(before, height, rows - base_rows + 1),
            std::cmp::Ordering::Less => probed_pitch(height, before, base_rows - rows + 1),
            std::cmp::Ordering::Equal => None,
        };
        Self::Settled(revised.or(Some(pitch)))
    }

    /// The stage after the area's width or `item_size` changed, if it restarts.
    fn resized(self, item_size: Option<f64>) -> Option<Self> {
        match (item_size, self) {
            (Some(size), _) => Some(Self::Settled(Some(size))),
            (None, Self::Settled(Some(pitch)) | Self::Revise { pitch, .. }) => {
                Some(Self::Revise { pitch, base: None })
            }
            _ => None,
        }
    }

    /// What `height` makes of this stage. A zero (not laid out yet) reruns the
    /// stage; believing it inflated the pitch by a whole row.
    fn advance(self, height: f64, count: usize) -> Self {
        if height <= 0.0 {
            return match self {
                Self::Single { attempt } if attempt + 1 < PROBE_ATTEMPTS => Self::Single {
                    attempt: attempt + 1,
                },
                Self::Settled(pitch) => Self::Settled(pitch),
                Self::Revise { .. } => self,
                _ => Self::Settled(None),
            };
        }

        match self {
            Self::Single { .. } if count >= 2 => Self::Batch { single: height },
            Self::Single { .. } => Self::Settled(probed_pitch(height, height, 1)),
            Self::Batch { single } => {
                Self::Settled(probed_pitch(single, height, PROBE_ROWS.min(count)))
            }
            Self::Settled(pitch) => Self::Settled(pitch),
            Self::Revise { .. } => self,
        }
    }
}

/// Renders only the rows of a long, uniform-height list that the
/// [`ScrollArea`] above it can show.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{List, ListItem, ScrollArea, Virtualize};
/// # fn app() -> Element {
/// let items = use_signal(Vec::<String>::new);
/// rsx! {
///     ScrollArea {
///         List {
///             Virtualize {
///                 count: items.len(),
///                 item: move |i| rsx! { ListItem { "{items.read()[i]}" } },
///             }
///         }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/layout/scroll-area>
#[component]
pub fn Virtualize(
    /// Rows in the whole list, not just the rendered ones.
    count: usize,
    /// Renders one row; called only for the rows in view.
    item: Callback<usize, Element>,
    /// Row pitch in px, a row plus its gap. Measured when unset, and again
    /// when the area's width changes.
    #[props(default)]
    item_size: Option<f64>,
    /// Rows kept beyond each edge; `theme.scroll_area.overscan` by default.
    #[props(default)]
    overscan: Option<usize>,
    /// An index rendered even out of view, e.g. the row holding focus.
    #[props(default)]
    keep_rendered: Option<usize>,
    /// The row's identity, e.g. its data's id; the index when unset. A row's
    /// state (focus, input, open details) follows it, so set it when rows can move.
    #[props(default)]
    item_key: Option<Callback<usize, String>>,
) -> Element {
    let theme = use_theme();
    let viewport = try_use_context::<ScrollViewport>();
    // Only one owns the offsets; the rest fall back to every row, and try again
    // each render, as the owner may be the instance this one replaces.
    let mut owner = use_hook(|| CopyValue::new(false));
    if !*owner.peek() && viewport.as_ref().is_some_and(ScrollViewport::claim) {
        owner.set(true);
    }
    let owned = *owner.peek();
    // A replaced owner lets go only after this first render: no rows until the retry,
    // as every row cost 10k mounts on a phone. Still unclaimed then, a second list renders all (2584).
    let mut waited = use_signal(|| false);
    use_effect(move || {
        if !*waited.peek() && !*owner.peek() {
            waited.set(true);
        }
    });
    let waiting = viewport.is_some() && !owned && !waited();
    let releasing = viewport.clone();
    use_drop(move || {
        if let (Some(viewport), Ok(true)) = (&releasing, owner.try_peek().map(|owned| *owned)) {
            viewport.release();
        }
    });

    let mut probe = use_signal(|| match item_size {
        Some(size) => Probe::Settled(Some(size)),
        None => Probe::Single { attempt: 0 },
    });

    // Asks the ScrollArea to make the content box measurable (not `display: contents`).
    let mut virtualized = viewport.as_ref().map(|viewport| viewport.virtualized);
    use_effect(use_reactive!(|owned| {
        if let (Some(virtualized), true) = (virtualized.as_mut(), owned)
            && !*virtualized.peek()
        {
            virtualized.set(true);
        }
    }));

    let geometry = viewport
        .as_ref()
        .and_then(|viewport| *viewport.geometry.read());
    // A rewrap after a resize, rotation or zoom, or a new `item_size`, changes the pitch.
    let width = geometry.map(|geometry| geometry.width);
    let mut seen = use_hook(|| CopyValue::new((item_size, width)));
    use_effect(use_reactive!(|(item_size, width)| {
        let (seen_size, seen_width) = *seen.peek();
        seen.set((item_size, width));
        let rewrapped = seen_width.is_some_and(|seen| seen > 0.0) && seen_width != width;
        let next = probe.peek().resized(item_size);
        if (seen_size != item_size || rewrapped)
            && let Some(next) = next
        {
            probe.set(next);
        }
    }));

    // The window the rows render for, read by the measuring effect after the render.
    let mut rendered = use_hook(|| CopyValue::new((0, 0)));
    let content = viewport.as_ref().map(|viewport| viewport.content);
    use_effect(use_reactive!(|(count, owned)| {
        let (Some(content), Some(virtualized), true) = (content, virtualized, owned) else {
            return;
        };
        let stage = probe();
        if !stage.measuring(count) || !virtualized() || !content.is_mounted() {
            return;
        }
        // The content box, not the container: its scroll height floors at the
        // viewport and would read the same for both probes.
        let measured = content.dimensions();
        spawn(async move {
            let Ok(size) = measured.await else {
                probe.set(Probe::Settled(None));
                return;
            };
            probe.set(match stage {
                Probe::Revise { .. } => stage.revised(size.height, *rendered.peek(), count),
                _ => stage.advance(size.height, count),
            });
        });
    }));

    let stage = probe();
    // A new pitch moves every row: the window and the scroll follow the row at the top (2538).
    let pitch = stage.pitch();
    let mut used = use_hook(|| CopyValue::new(pitch));
    let place = owned
        .then(|| kept_place(*used.peek(), pitch, geometry))
        .flatten();
    if *used.peek() != pitch {
        used.set(pitch);
    }
    let geometry = place.or(geometry);
    let area = viewport
        .as_ref()
        .map(|viewport| (viewport.area, viewport.geometry));
    use_effect(use_reactive!(|place| {
        let (Some(place), Some((area, mut geometry))) = (place, area) else {
            return;
        };
        geometry.set(Some(place));
        let offset = area.scroll_offset();
        spawn(async move {
            if let Ok((x, _)) = offset.await {
                let _ = area.scroll_to(x, place.offset);
            }
        });
    }));
    let mut kept = None;
    let spec = match (owned, pitch) {
        (true, Some(pitch)) => Some(WindowSpec {
            count,
            pitch,
            overscan: overscan.unwrap_or(theme.scroll_area.overscan),
            keep: keep_rendered,
        }),
        _ => None,
    };
    let visible = match (owned, stage, spec) {
        // Don't wait for a measurement: server renders never get one, and
        // waiting rendered every row.
        (_, _, Some(spec)) => {
            let (visible, beside) = spec.at(geometry);
            kept = beside;
            visible
        }
        // Still probing: render just enough to measure, for one frame.
        (true, stage, _) => match stage.rows(count) {
            Some(rows) => Window {
                range: 0..rows,
                offsets: ContentOffsets::default(),
            },
            None => Window::all(count),
        },
        _ if waiting => Window {
            range: 0..0,
            offsets: ContentOffsets::default(),
        },
        _ => Window::all(count),
    };

    // Handed up rather than drawn here: spacer elements would have to guess a
    // tag legal inside whatever list wraps these rows.
    let mut handed = viewport.as_ref().map(|viewport| viewport.spec);
    use_effect(use_reactive!(|(spec, owned)| {
        if let (Some(handed), true) = (handed.as_mut(), owned)
            && *handed.peek() != spec
        {
            handed.set(spec);
        }
    }));

    use_hook(|| {
        if viewport.is_none() {
            warn("Virtualize: no ScrollArea above it, so every row renders.");
        }
    });

    let laid = kept.map(|(index, before)| match before {
        true => (index, visible.range.start.saturating_sub(1)),
        false => (index, visible.range.end),
    });
    let mut kept_slot = use_context_provider(|| KeptSlot(Signal::new(None))).0;
    use_effect(use_reactive!(|laid| {
        if *kept_slot.peek() != laid {
            kept_slot.set(laid);
        }
    }));

    let (before, after) = match kept {
        Some((index, true)) => (Some(index), None),
        Some((index, false)) => (None, Some(index)),
        None => (None, None),
    };
    // Keyed: unkeyed, a row's node would pass to the next row as the window shifts.
    rendered.set((visible.range.start, visible.range.end));
    let rows = before
        .into_iter()
        .chain(stage.revise_rows(visible.range, count))
        .chain(after)
        .map(|index| {
            let key = match item_key {
                Some(item_key) => item_key.call(index),
                None => index.to_string(),
            };
            item.call(index).map(|row| keyed(row, key))
        });
    rsx! {
        {rows}
    }
}

/// The geometry keeping the row at the top in place when the pitch went from `old` to
/// `new`, `None` when it did not change or nothing is scrolled.
fn kept_place(
    old: Option<f64>,
    new: Option<f64>,
    at: Option<ScrollGeometry>,
) -> Option<ScrollGeometry> {
    let (old, new, at) = (old?, new?, at?);
    (old > 0.0 && old != new && at.offset > 0.0).then(|| ScrollGeometry {
        offset: at.offset * new / old,
        step: 0.0,
        ..at
    })
}

/// `row` under `key`; a `Fragment` wrapper would cost a component render per row.
fn keyed(row: VNode, key: String) -> VNode {
    VNode::new(
        *row.template(),
        DynamicValues::from_parts(
            Some(key),
            row.dynamic_node_values().into(),
            row.dynamic_attr_values().into(),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pitch(probe: Probe) -> Option<f64> {
        match probe {
            Probe::Settled(pitch) => pitch,
            _ => panic!("still probing"),
        }
    }

    /// Eight 40px rows with a 12px gap between them: 40 + 7 * 52.
    const BATCH: f64 = 40.0 + 7.0 * 52.0;

    #[test]
    fn two_probes_give_the_row_plus_its_gap() {
        let single = Probe::Single { attempt: 0 }.advance(40.0, 1000);
        assert_eq!(single, Probe::Batch { single: 40.0 });
        assert_eq!(pitch(single.advance(BATCH, 1000)), Some(52.0));
    }

    /// A `display: contents` box measures zero; believed, it made a 52px pitch 57.7px.
    #[test]
    fn a_zero_first_measurement_is_retried_not_believed() {
        let retried = Probe::Single { attempt: 0 }.advance(0.0, 1000);

        assert_eq!(retried, Probe::Single { attempt: 1 });
        assert_eq!(
            pitch(retried.advance(40.0, 1000).advance(BATCH, 1000)),
            Some(52.0)
        );
    }

    #[test]
    fn a_box_that_never_lays_out_gives_up() {
        let mut probe = Probe::Single { attempt: 0 };
        for _ in 0..PROBE_ATTEMPTS {
            probe = probe.advance(0.0, 1000);
        }

        assert_eq!(pitch(probe), None);
    }

    /// One row has no gap to measure, so it is its own pitch.
    #[test]
    fn a_single_row_list_skips_the_batch() {
        assert_eq!(
            pitch(Probe::Single { attempt: 0 }.advance(40.0, 1)),
            Some(40.0)
        );
    }

    #[test]
    fn a_settled_pitch_ignores_a_probe_measurement() {
        let settled = Probe::Settled(Some(52.0));

        assert_eq!(pitch(settled.advance(999.0, 1000)), Some(52.0));
        assert_eq!(pitch(settled.advance(0.0, 1000)), Some(52.0));
    }

    /// Rows rewrapped from 52px to 80px: the window, then 7 rows more below it.
    #[test]
    fn a_resize_revises_the_pitch_in_place() {
        let revise = Probe::Settled(Some(52.0)).resized(None).unwrap();
        assert_eq!(revise.revise_rows(10..30, 1000), 10..30);

        let based = revise.revised(20.0 * 80.0, (10, 30), 1000);
        assert_eq!(based.revise_rows(10..30, 1000), 10..37);
        assert_eq!(based.pitch(), Some(52.0));
        assert_eq!(
            pitch(based.revised(27.0 * 80.0, (10, 30), 1000)),
            Some(80.0)
        );
    }

    /// At the list's end the added rows go above; a list that fits drops rows.
    #[test]
    fn a_revise_weighs_rows_where_the_list_has_them() {
        let at_end = Probe::Revise {
            pitch: 52.0,
            base: Some((20.0 * 80.0, (980, 1000))),
        };
        assert_eq!(at_end.revise_rows(980..1000, 1000), 973..1000);
        assert_eq!(
            pitch(at_end.revised(27.0 * 80.0, (980, 1000), 1000)),
            Some(80.0)
        );

        let fits = Probe::Revise {
            pitch: 52.0,
            base: Some((10.0 * 80.0, (0, 10))),
        };
        assert_eq!(fits.revise_rows(0..10, 10), 0..3);
        assert_eq!(pitch(fits.revised(3.0 * 80.0, (0, 10), 10)), Some(80.0));
    }

    #[test]
    fn a_scroll_during_a_revise_starts_it_over() {
        let based = Probe::Revise {
            pitch: 52.0,
            base: Some((1600.0, (10, 30))),
        };

        assert_eq!(based.revise_rows(12..32, 1000), 12..32);
        assert_eq!(
            based.revised(1600.0, (12, 32), 1000),
            Probe::Revise {
                pitch: 52.0,
                base: None
            }
        );
    }

    /// Todo 2538: row 10 at the top under a 52px pitch stays there under 80px, a third in.
    #[test]
    fn a_new_pitch_keeps_the_row_at_the_top() {
        let at = ScrollGeometry {
            offset: 10.0 * 52.0 + 52.0 / 3.0,
            viewport: 400.0,
            step: 40.0,
            width: 300.0,
        };
        let place = kept_place(Some(52.0), Some(80.0), Some(at)).unwrap();
        assert!((place.offset - (10.0 * 80.0 + 80.0 / 3.0)).abs() < 1e-9);
        assert_eq!(
            (place.step, place.viewport, place.width),
            (0.0, 400.0, 300.0)
        );
        // The first pitch, an unchanged one, or no scroll keeps the offset.
        assert_eq!(kept_place(None, Some(80.0), Some(at)), None);
        assert_eq!(kept_place(Some(52.0), Some(52.0), Some(at)), None);
        let top = ScrollGeometry { offset: 0.0, ..at };
        assert_eq!(kept_place(Some(52.0), Some(80.0), Some(top)), None);
        assert_eq!(kept_place(Some(52.0), None, Some(at)), None);
    }

    /// A new `item_size` is used at once; without one, a probe in progress goes on.
    #[test]
    fn a_new_item_size_replaces_the_pitch() {
        assert_eq!(
            Probe::Settled(Some(52.0)).resized(Some(80.0)),
            Some(Probe::Settled(Some(80.0)))
        );
        assert_eq!(Probe::Single { attempt: 0 }.resized(None), None);
        assert_eq!(
            pitch(
                Probe::Revise {
                    pitch: 52.0,
                    base: None
                }
                .revised(0.0, (0, 20), 1000)
            ),
            Some(52.0)
        );
    }
}
