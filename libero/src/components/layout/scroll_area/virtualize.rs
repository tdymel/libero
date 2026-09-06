use dioxus::prelude::*;

use super::viewport::{
    ContentOffsets, ScrollGeometry, ScrollViewport, Window, probed_pitch, window,
};
use crate::{hooks::use_theme, platform::ElementApi, utils::warn};

/// Rows in the second probe render. Enough that a per-row gap is a real part
/// of the measured height, few enough to cost nothing if the list is short.
const PROBE_ROWS: usize = 8;

/// A measurement of nothing is a measurement that came too early - the content
/// box is still `display: contents`, which has no box to measure. Each retry
/// costs one frame, and giving up renders every row.
const PROBE_ATTEMPTS: usize = 8;

/// The geometry assumed while there is a pitch but the `ScrollArea` has not
/// measured itself yet: the first render, and every server render. Scrolled to
/// the top of a full 1080p screen, so a first paint at any common height has
/// no blank rows below the last one, while a 50,000-row list still renders a
/// few dozen.
const UNMEASURED: ScrollGeometry = ScrollGeometry {
    offset: 0.0,
    viewport: 1080.0,
};

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

    /// What `height` makes of this stage. A zero says the box was not laid out
    /// yet, so the same stage runs again rather than dividing by it - the first
    /// measurement landing as 0 is what inflated the pitch by a whole row.
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

/// Renders only the rows of a long list that its [`ScrollArea`] can show.
///
/// It draws no element of its own, so it goes wherever the rows go - inside a
/// [`List`](crate::components::List), a [`Table`](crate::components::Table)
/// body, or a plain stack - as long as a `ScrollArea` is somewhere above it.
/// Without one it warns and renders every row.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{List, ListItem, ScrollArea, Virtualize};
/// # fn app() -> Element {
/// # let items = use_signal(Vec::<String>::new);
/// # rsx! {
/// ScrollArea {
///     List {
///         Virtualize {
///             count: items.len(),
///             item: move |i| rsx! { ListItem { "{items.read()[i]}" } },
///         }
///     }
/// }
/// # } }
/// ```
///
/// Rows must be uniform height - it measures one and assumes the rest match.
#[component]
pub fn Virtualize(
    /// Rows in the whole list, not just the rendered ones.
    count: usize,
    /// Renders one row. Called only for the rows in view, so the list itself
    /// is never walked.
    item: Callback<usize, Element>,
    /// Row pitch in px - a row's height plus the gap below it. Measured from
    /// the first rows rendered when unset; set it to skip that, which off the
    /// web saves two round-trips.
    #[props(default)]
    item_size: Option<f64>,
    /// Rows kept beyond each edge, so a scroll has something to reveal before
    /// the next render lands. `theme.scroll_area.overscan` by default.
    #[props(default)]
    overscan: Option<usize>,
) -> Element {
    let theme = use_theme();
    let viewport = try_use_context::<ScrollViewport>();
    // Only the first one owns the offsets; the rest fall back to every row.
    let owned = use_hook(|| viewport.as_ref().is_some_and(ScrollViewport::claim));

    let mut probe = use_signal(|| match item_size {
        Some(size) => Probe::Settled(Some(size)),
        None => Probe::Single { attempt: 0 },
    });

    // The content box only becomes measurable once it stops being
    // `display: contents`, which is what this asks the ScrollArea for.
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
        // The probe render is committed by the time an effect runs, so this
        // measures what is actually on screen. The content box sizes to its
        // rows, unlike the container, whose scroll height floors at the
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
    let visible = match (owned, stage, geometry) {
        // A given `item_size` settles the probe before the ScrollArea has
        // measured anything, and waiting for that rendered every row - on the
        // first render, and on every server render, which never measures.
        (true, Probe::Settled(Some(pitch)), geometry) => window(
            count,
            pitch,
            geometry.unwrap_or(UNMEASURED),
            overscan.unwrap_or(theme.scroll_area.overscan),
        ),
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

    rsx! {
        for index in visible.range {
            {item.call(index)}
        }
    }
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

    /// The bug this guards: a content box still `display: contents` measures
    /// zero, and dividing the batch by one row fewer inflated every pitch by a
    /// whole row - 57.7px where the rows were laid out 52px apart.
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
