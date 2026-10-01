use dioxus::{core::DynamicValues, prelude::*};

use super::viewport::{
    ContentOffsets, ScrollGeometry, ScrollViewport, Window, keep_beside, probed_pitch, window,
};
use crate::{hooks::use_theme, platform::ElementApi, utils::warn};

/// Rows in the second probe render: enough to weigh the gap, cheap on a short list.
const PROBE_ROWS: usize = 8;

/// A zero came too early (still `display: contents`); each retry costs a frame,
/// giving up renders every row.
const PROBE_ATTEMPTS: usize = 8;

/// Assumed before the `ScrollArea` measured itself (first and server renders):
/// a 1080p screen, so no blank rows on first paint at any common height.
const UNMEASURED: ScrollGeometry = ScrollGeometry {
    offset: 0.0,
    viewport: 1080.0,
};

/// The row a `Virtualize` keeps out of its window and the slot its box sits at,
/// beside the window: for a row that translates itself to a slot (todo 1408).
#[derive(Clone, Copy)]
pub(crate) struct KeptSlot(pub Signal<Option<(usize, usize)>>);

/// The nearest `Virtualize`'s kept row, read inside one of its rows.
pub(crate) fn use_kept_slot() -> Option<Signal<Option<(usize, usize)>>> {
    try_use_context::<KeptSlot>().map(|kept| kept.0)
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
}

impl Probe {
    /// Rows to render while probing, or `None` once there is a pitch to use.
    fn rows(self, count: usize) -> Option<usize> {
        match self {
            Self::Single { .. } => Some(1.min(count)),
            Self::Batch { .. } => Some(PROBE_ROWS.min(count)),
            Self::Settled(_) => None,
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
    /// Row pitch in px, a row plus its gap. Measured when unset.
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
    // Only the first one owns the offsets; the rest fall back to every row.
    let owned = use_hook(|| viewport.as_ref().is_some_and(ScrollViewport::claim));

    let mut probe = use_signal(|| match item_size {
        Some(size) => Probe::Settled(Some(size)),
        None => Probe::Single { attempt: 0 },
    });

    // Asks the ScrollArea to make the content box measurable (not `display: contents`).
    let mut virtualized = viewport.as_ref().map(|viewport| viewport.virtualized);
    use_effect(move || {
        if let (Some(virtualized), true) = (virtualized.as_mut(), owned)
            && !*virtualized.peek()
        {
            virtualized.set(true);
        }
    });

    let content = viewport.as_ref().map(|viewport| viewport.content);
    use_effect(use_reactive!(|count| {
        let (Some(content), Some(virtualized), true) = (content, virtualized, owned) else {
            return;
        };
        let stage = probe();
        if stage.rows(count).is_none() || !virtualized() || !content.is_mounted() {
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
            probe.set(stage.advance(size.height, count));
        });
    }));

    let stage = probe();
    let geometry = viewport
        .as_ref()
        .and_then(|viewport| *viewport.geometry.read());
    let mut kept = None;
    let visible = match (owned, stage, geometry) {
        // Don't wait for a measurement: server renders never get one, and
        // waiting rendered every row.
        (true, Probe::Settled(Some(pitch)), geometry) => {
            let mut visible = window(
                count,
                pitch,
                geometry.unwrap_or(UNMEASURED),
                overscan.unwrap_or(theme.scroll_area.overscan),
            );
            kept = keep_beside(&mut visible, keep_rendered, count, pitch);
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
        _ => Window::all(count),
    };

    // Handed up rather than drawn here: spacer elements would have to guess a
    // tag legal inside whatever list wraps these rows.
    let offsets = visible.offsets;
    let mut reserved = viewport.as_ref().map(|viewport| viewport.offsets);
    use_effect(use_reactive!(|offsets| {
        if let (Some(reserved), true) = (reserved.as_mut(), owned)
            && *reserved.peek() != offsets
        {
            reserved.set(offsets);
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
    let rows = before
        .into_iter()
        .chain(visible.range)
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
    fn a_settled_pitch_is_never_revised() {
        let settled = Probe::Settled(Some(52.0));

        assert_eq!(pitch(settled.advance(999.0, 1000)), Some(52.0));
        assert_eq!(pitch(settled.advance(0.0, 1000)), Some(52.0));
    }
}
