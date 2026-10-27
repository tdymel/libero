use std::collections::HashMap;

use super::core::HeaderSpec;

/// The columns of a [`Table`](super::Table) held at its edges while the rest
/// scroll, by header text. Logical sides: `start` is the left in a
/// left-to-right page, the right in a right-to-left one.
///
/// ```rust
/// # use libero::components::PinnedColumns;
/// let pinned = PinnedColumns::default().start(["Name"]).end(["Actions"]);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PinnedColumns {
    pub start: Vec<String>,
    pub end: Vec<String>,
}

impl PinnedColumns {
    pub fn start<S: Into<String>>(mut self, headers: impl IntoIterator<Item = S>) -> Self {
        self.start = headers.into_iter().map(Into::into).collect();
        self
    }

    pub fn end<S: Into<String>>(mut self, headers: impl IntoIterator<Item = S>) -> Self {
        self.end = headers.into_iter().map(Into::into).collect();
        self
    }

    /// The side `header` is pinned to, if any.
    pub(super) fn side(&self, header: &str) -> Option<PinSide> {
        match (
            self.start.iter().any(|h| h == header),
            self.end.iter().any(|h| h == header),
        ) {
            (true, _) => Some(PinSide::Start),
            (false, true) => Some(PinSide::End),
            _ => None,
        }
    }

    /// `header` moved to `side`, last there, or unpinned with `None`.
    pub(super) fn with(&self, header: &str, side: Option<PinSide>) -> Self {
        let without = |list: &[String]| -> Vec<String> {
            list.iter().filter(|h| *h != header).cloned().collect()
        };
        let mut next = Self {
            start: without(&self.start),
            end: without(&self.end),
        };
        match side {
            Some(PinSide::Start) => next.start.push(header.to_string()),
            // The newest end column goes innermost, as the newest start one does.
            Some(PinSide::End) => next.end.insert(0, header.to_string()),
            None => {}
        }
        next
    }
}

/// An edge of a table, logical: `Start` is the left in a left-to-right page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinSide {
    Start,
    End,
}

impl PinSide {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

/// A pinned cell: its edge, how far in from it, and whether it borders the
/// scrolled columns.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct CellPin {
    pub side: PinSide,
    pub inset: String,
    pub edge: bool,
}

impl CellPin {
    /// The inline style holding it at its inset, logical so a right-to-left page flips it.
    pub fn style(&self) -> String {
        pin_insets(Some(self))
    }
}

/// Both insets of a cell, the one it isn't held by `auto`: a cell moved to the other
/// edge or unpinned would keep its old one otherwise (todos 3132, 3149).
pub(super) fn pin_insets(pin: Option<&CellPin>) -> String {
    let inset = |side: PinSide| match pin {
        Some(pin) if pin.side == side => pin.inset.as_str(),
        _ => "auto",
    };
    format!(
        "inset-inline-start:{};inset-inline-end:{};",
        inset(PinSide::Start),
        inset(PinSide::End)
    )
}

/// Whether the column at `position` of `order` starts a new run of one pin side.
pub(super) fn pin_edge_at(headers: &[HeaderSpec], order: &[usize], position: usize) -> bool {
    let side = |at: usize| headers[order[at]].pin.as_ref().map(|pin| pin.side);
    position > 0 && side(position - 1) != side(position)
}

/// `order` cut at its pin edges: groups and spans don't cross one.
pub(super) fn pin_runs<'a>(
    headers: &'a [HeaderSpec],
    order: &'a [usize],
) -> impl Iterator<Item = &'a [usize]> {
    let side = |index: usize| headers[index].pin.as_ref().map(|pin| pin.side);
    order.chunk_by(move |&a, &b| side(a) == side(b))
}

/// The pin of a cell over `covered`, columns of one run: held at its outer
/// column's inset, an edge when any column is.
pub(super) fn span_pin(headers: &[HeaderSpec], covered: &[usize]) -> Option<CellPin> {
    let first = headers[*covered.first()?].pin.as_ref()?;
    let outer = match first.side {
        PinSide::Start => first,
        PinSide::End => headers[*covered.last()?].pin.as_ref()?,
    };
    Some(CellPin {
        side: first.side,
        inset: outer.inset.clone(),
        edge: covered
            .iter()
            .any(|&index| headers[index].pin.as_ref().is_some_and(|pin| pin.edge)),
    })
}

/// The least share of the scroll area's width the unpinned columns keep: pins that
/// leave less are released (todo 3256).
pub(super) const MIN_UNPINNED_SHARE: f64 = 1.0 / 3.0;

/// The indices of the shown columns pinned to each side, in `pinned` order; one
/// pinned to both stays at the start.
fn pin_groups(headers: &[HeaderSpec], pinned: &PinnedColumns) -> (Vec<usize>, Vec<usize>) {
    let shown = |header: &String| {
        headers
            .iter()
            .position(|spec| !spec.hidden && spec.header == *header)
    };
    let mut start: Vec<usize> = Vec::new();
    for index in pinned.start.iter().filter_map(shown) {
        if !start.contains(&index) {
            start.push(index);
        }
    }
    let mut end: Vec<usize> = Vec::new();
    for index in pinned.end.iter().filter_map(shown) {
        if !start.contains(&index) && !end.contains(&index) {
            end.push(index);
        }
    }
    (start, end)
}

/// What the pinned columns are measured against.
pub(super) struct PinRoom<'a> {
    /// The scroll area's width; `None` before it is measured.
    pub area: Option<f64>,
    /// The leading columns' width, held with a start group.
    pub lead: f64,
    /// Each pinned header's rendered width.
    pub widths: &'a HashMap<String, f64>,
}

/// Whether `pinned` has a shown column at its start.
pub(super) fn has_start_pins(headers: &[HeaderSpec], pinned: &PinnedColumns) -> bool {
    !pin_groups(headers, pinned).0.is_empty()
}

/// `pinned` less the columns that crowd the unpinned ones below
/// [`MIN_UNPINNED_SHARE`] of the area: the end side's innermost first, then the start
/// side's. A released column's width is its last measured one, so a regrown area re-pins.
pub(super) fn fit_pins(
    headers: &[HeaderSpec],
    pinned: &PinnedColumns,
    room: &PinRoom,
) -> PinnedColumns {
    let Some(area) = room.area else {
        return pinned.clone();
    };
    let (start, end) = pin_groups(headers, pinned);
    let width = |index: usize| {
        let spec = &headers[index];
        room.widths.get(&spec.header).copied().unwrap_or_else(|| {
            spec.width
                .as_deref()
                .and_then(|width| width.strip_suffix("px")?.parse().ok())
                .unwrap_or(0.0)
        })
    };
    let lead = if start.is_empty() { 0.0 } else { room.lead };
    let mut total = lead
        + start
            .iter()
            .chain(&end)
            .map(|&index| width(index))
            .sum::<f64>();
    let limit = area * (1.0 - MIN_UNPINNED_SHARE);
    let mut fitted = pinned.clone();
    for &index in end.iter().chain(start.iter().rev()) {
        if total <= limit {
            break;
        }
        total -= width(index);
        let header = &headers[index].header;
        fitted.start.retain(|pinned| pinned != header);
        fitted.end.retain(|pinned| pinned != header);
    }
    fitted
}

/// The shown columns in display order: start-pinned ones in `pinned` order,
/// the rest in column order, then end-pinned ones. Sets each pinned header's
/// [`CellPin`]; `lead_width` is the toggle and checkbox columns', held with a
/// start group; `measured` holds the pinned headers' rendered widths. Returns the order and
/// the pinned headers whose inset is unknown, as a column further out has no `width`.
pub(super) fn pin_columns(
    headers: &mut [HeaderSpec],
    pinned: &PinnedColumns,
    lead_width: Option<&str>,
    measured: &HashMap<String, f64>,
) -> (Vec<usize>, Vec<String>) {
    let (start, mut end) = pin_groups(headers, pinned);
    let middle = (0..headers.len())
        .filter(|index| !headers[*index].hidden && !start.contains(index) && !end.contains(index));
    let order: Vec<usize> = start
        .iter()
        .copied()
        .chain(middle)
        .chain(end.iter().copied())
        .collect();

    let mut unknown = Vec::new();
    let outer = lead_width.filter(|_| !start.is_empty()).map(str::to_string);
    let mut place = |group: &[usize], side: PinSide, headers: &mut [HeaderSpec]| {
        let mut widths: Vec<String> = outer
            .iter()
            .filter(|_| side == PinSide::Start)
            .cloned()
            .collect();
        let mut known = true;
        for (at, &index) in group.iter().enumerate() {
            if !known {
                unknown.push(headers[index].header.clone());
            }
            headers[index].pin = Some(CellPin {
                side,
                inset: sum(&widths),
                edge: at + 1 == group.len(),
            });
            match sized_width(&headers[index], measured) {
                Some(width) => widths.push(width),
                None => known = false,
            }
        }
    };
    place(&start, PinSide::Start, headers);
    // Counted from the end edge inwards.
    end.reverse();
    place(&end, PinSide::End, headers);
    (order, unknown)
}

/// How far `side`'s pinned columns reach in, lead columns included: a focus scroll
/// stops past them. The innermost one without a `width` counts as `measured`, its
/// rendered width (todo 1973); `None` without a pin there or before that is known.
pub(super) fn pinned_extent(
    headers: &[HeaderSpec],
    side: PinSide,
    measured: Option<f64>,
    widths: &HashMap<String, f64>,
) -> Option<String> {
    headers.iter().find_map(|spec| {
        let pin = spec
            .pin
            .as_ref()
            .filter(|pin| pin.side == side && pin.edge)?;
        let width =
            sized_width(spec, widths).or_else(|| measured.map(|width| format!("{width}px")))?;
        Some(match pin.inset.as_str() {
            "0" => width,
            inset => format!("calc({inset} + {width})"),
        })
    })
}

/// A sized column's rendered width once measured, else its `width`: an unbreakable cell
/// text grows it past the `width`, and the insets and the focus scroll must clear that.
fn sized_width(spec: &HeaderSpec, measured: &HashMap<String, f64>) -> Option<String> {
    let width = spec.width.as_ref()?;
    Some(match measured.get(&spec.header) {
        Some(px) => format!("{px}px"),
        None => width.clone(),
    })
}

fn sum(widths: &[String]) -> String {
    match widths {
        [] => "0".to_string(),
        [one] => one.clone(),
        many => format!("calc({})", many.join(" + ")),
    }
}

#[cfg(test)]
mod tests {
    use super::{super::cell_value::CellAlign, *};

    fn headers(columns: &[(&str, Option<&str>)]) -> Vec<HeaderSpec> {
        columns
            .iter()
            .map(|(header, width)| HeaderSpec {
                header: header.to_string(),
                key: header.to_string(),
                align: CellAlign::Start,
                sortable: false,
                row_header: false,
                hideable: true,
                hidden: false,
                width: width.map(str::to_string),
                min_width: None,
                body: None,
                groups: Vec::new(),
                pin: None,
                resize: None,
            })
            .collect()
    }

    fn insets(headers: &[HeaderSpec]) -> Vec<Option<(PinSide, String, bool)>> {
        headers
            .iter()
            .map(|spec| spec.pin.clone().map(|pin| (pin.side, pin.inset, pin.edge)))
            .collect()
    }

    #[test]
    fn pinned_columns_move_to_their_edges_and_stack_their_widths() {
        let mut specs = headers(&[
            ("A", Some("4rem")),
            ("B", Some("6rem")),
            ("C", None),
            ("D", Some("5rem")),
            ("E", Some("3rem")),
        ]);
        let pinned = PinnedColumns::default().start(["D", "B"]).end(["A", "E"]);

        let (order, unknown) = pin_columns(&mut specs, &pinned, None, &HashMap::new());

        assert_eq!(order, vec![3, 1, 2, 0, 4]);
        assert!(unknown.is_empty());
        assert_eq!(
            insets(&specs),
            vec![
                Some((PinSide::End, "3rem".into(), true)),
                Some((PinSide::Start, "5rem".into(), true)),
                None,
                Some((PinSide::Start, "0".into(), false)),
                Some((PinSide::End, "0".into(), false)),
            ]
        );
    }

    #[test]
    fn the_checkbox_column_counts_only_with_a_start_group() {
        let mut specs = headers(&[("A", Some("4rem")), ("B", Some("6rem"))]);
        let pinned = PinnedColumns::default().start(["A", "B"]);

        pin_columns(&mut specs, &pinned, Some("40px"), &HashMap::new());

        assert_eq!(specs[0].pin.as_ref().unwrap().inset, "40px");
        assert_eq!(specs[1].pin.as_ref().unwrap().inset, "calc(40px + 4rem)");

        let mut specs = headers(&[("A", Some("4rem")), ("B", Some("6rem"))]);
        pin_columns(
            &mut specs,
            &PinnedColumns::default().end(["A"]),
            Some("40px"),
            &HashMap::new(),
        );
        assert_eq!(specs[0].pin.as_ref().unwrap().inset, "0");
    }

    /// Todo 1791: the scroll padding that keeps focus clear of each pinned side.
    #[test]
    fn each_side_reaches_past_its_innermost_pinned_column() {
        let mut specs = headers(&[
            ("A", Some("4rem")),
            ("B", Some("6rem")),
            ("C", None),
            ("D", Some("3rem")),
        ]);
        let pinned = PinnedColumns::default().start(["A", "B"]).end(["D"]);
        pin_columns(&mut specs, &pinned, Some("40px"), &HashMap::new());

        let none = HashMap::new();
        assert_eq!(
            pinned_extent(&specs, PinSide::Start, Some(99.0), &none).as_deref(),
            Some("calc(calc(40px + 4rem) + 6rem)")
        );
        assert_eq!(
            pinned_extent(&specs, PinSide::End, None, &none).as_deref(),
            Some("3rem")
        );

        let mut specs = headers(&[("A", None), ("B", Some("6rem"))]);
        let start = PinnedColumns::default().start(["A"]);
        pin_columns(&mut specs, &start, None, &none);
        assert_eq!(pinned_extent(&specs, PinSide::Start, None, &none), None);
        assert_eq!(pinned_extent(&specs, PinSide::End, Some(80.0), &none), None);
    }

    /// A sized column grown past its `width` by its text insets and pads by the width it renders.
    #[test]
    fn a_sized_pin_grown_past_its_width_counts_as_rendered() {
        let mut specs = headers(&[("A", Some("240px")), ("B", Some("240px")), ("C", None)]);
        let measured = HashMap::from([("A".to_string(), 270.5)]);
        let pinned = PinnedColumns::default().start(["A", "B"]);
        pin_columns(&mut specs, &pinned, Some("40px"), &measured);

        assert_eq!(specs[1].pin.as_ref().unwrap().inset, "calc(40px + 270.5px)");
        assert_eq!(
            pinned_extent(&specs, PinSide::Start, None, &measured).as_deref(),
            Some("calc(calc(40px + 270.5px) + 240px)")
        );
    }

    /// 1973: an innermost pinned column without `width` reaches as far as it renders.
    #[test]
    fn an_unsized_innermost_pin_reaches_its_measured_width() {
        let mut specs = headers(&[("A", Some("4rem")), ("B", None), ("C", None)]);
        let pinned = PinnedColumns::default().start(["A", "B"]).end(["C"]);
        pin_columns(&mut specs, &pinned, None, &HashMap::new());

        let none = HashMap::new();
        assert_eq!(
            pinned_extent(&specs, PinSide::Start, Some(120.0), &none).as_deref(),
            Some("calc(4rem + 120px)")
        );
        assert_eq!(
            pinned_extent(&specs, PinSide::End, Some(80.0), &none).as_deref(),
            Some("80px")
        );
    }

    #[test]
    fn a_column_past_one_without_width_is_reported() {
        let mut specs = headers(&[("A", None), ("B", Some("6rem")), ("C", None)]);
        let pinned = PinnedColumns::default().start(["A", "B"]).end(["C"]);

        let (_, unknown) = pin_columns(&mut specs, &pinned, None, &HashMap::new());

        assert_eq!(unknown, vec!["B"]);
    }

    #[test]
    fn hidden_unknown_and_twice_named_columns_are_skipped() {
        let mut specs = headers(&[("A", Some("1px")), ("B", Some("2px")), ("C", Some("3px"))]);
        specs[1].hidden = true;
        let pinned = PinnedColumns::default()
            .start(["B", "X", "C", "C"])
            .end(["C", "A"]);

        let (order, _) = pin_columns(&mut specs, &pinned, None, &HashMap::new());

        assert_eq!(order, vec![2, 0]);
        assert_eq!(specs[1].pin, None);
        assert_eq!(specs[2].pin.as_ref().unwrap().side, PinSide::Start);
        assert_eq!(specs[0].pin.as_ref().unwrap().side, PinSide::End);
    }

    fn room(area: Option<f64>, lead: f64, widths: &HashMap<String, f64>) -> PinRoom<'_> {
        PinRoom { area, lead, widths }
    }

    /// Todo 3256: the end side's innermost pin goes first, then the start side's.
    #[test]
    fn pins_leaving_the_rest_under_a_third_are_released_innermost_first() {
        let specs = headers(&[
            ("A", Some("100px")),
            ("B", Some("100px")),
            ("C", Some("100px")),
            ("D", Some("100px")),
            ("E", Some("100px")),
        ]);
        let pinned = PinnedColumns::default().start(["A", "B"]).end(["D", "E"]);
        let widths = HashMap::new();
        let fits = |area, lead| fit_pins(&specs, &pinned, &room(Some(area), lead, &widths));

        assert_eq!(fits(600.0, 0.0), pinned);
        assert_eq!(
            fits(450.0, 0.0),
            PinnedColumns::default().start(["A", "B"]).end(["E"])
        );
        assert_eq!(fits(300.0, 0.0), PinnedColumns::default().start(["A", "B"]));
        assert_eq!(fits(250.0, 40.0), PinnedColumns::default().start(["A"]));
        assert_eq!(fits(100.0, 40.0), PinnedColumns::default());
        assert_eq!(fit_pins(&specs, &pinned, &room(None, 0.0, &widths)), pinned);
    }

    #[test]
    fn a_measured_width_beats_the_declared_one_and_a_hidden_pin_counts_for_nothing() {
        let mut specs = headers(&[("A", Some("100px")), ("B", None), ("C", Some("100px"))]);
        specs[2].hidden = true;
        let pinned = PinnedColumns::default().start(["A", "B", "C"]);
        let wide = HashMap::from([("B".to_string(), 400.0)]);

        assert_eq!(
            fit_pins(&specs, &pinned, &room(Some(600.0), 0.0, &wide)),
            PinnedColumns::default().start(["A", "C"])
        );
        assert_eq!(
            fit_pins(&specs, &pinned, &room(Some(600.0), 0.0, &HashMap::new())),
            pinned
        );
    }

    #[test]
    fn a_pin_with_no_shown_start_column_ignores_the_lead_width() {
        let specs = headers(&[("A", Some("100px")), ("B", Some("100px"))]);
        let pinned = PinnedColumns::default().end(["A"]);
        let widths = HashMap::new();

        assert_eq!(
            fit_pins(&specs, &pinned, &room(Some(200.0), 90.0, &widths)),
            pinned
        );
        assert!(!has_start_pins(&specs, &pinned));
    }

    #[test]
    fn a_pin_moves_a_column_between_sides() {
        let pinned = PinnedColumns::default().start(["A"]).end(["B"]);

        assert_eq!(
            pinned.with("B", Some(PinSide::Start)),
            PinnedColumns::default().start(["A", "B"])
        );
        assert_eq!(
            pinned.with("C", Some(PinSide::End)),
            PinnedColumns::default().start(["A"]).end(["C", "B"])
        );
        assert_eq!(pinned.with("A", None), PinnedColumns::default().end(["B"]));
        assert_eq!(pinned.side("B"), Some(PinSide::End));
    }

    #[test]
    fn runs_cut_at_pin_edges_and_a_span_holds_at_its_outer_column() {
        let mut specs = headers(&[
            ("A", Some("1px")),
            ("B", Some("2px")),
            ("C", Some("3px")),
            ("D", Some("4px")),
            ("E", Some("5px")),
        ]);
        let pinned = PinnedColumns::default().start(["A", "B"]).end(["D", "E"]);
        let (order, _) = pin_columns(&mut specs, &pinned, None, &HashMap::new());

        let runs: Vec<&[usize]> = pin_runs(&specs, &order).collect();
        assert_eq!(runs, vec![&[0, 1][..], &[2], &[3, 4]]);
        assert!(pin_edge_at(&specs, &order, 2));
        assert!(!pin_edge_at(&specs, &order, 1));

        let start = span_pin(&specs, &[0, 1]).unwrap();
        assert_eq!((start.inset.as_str(), start.edge), ("0", true));
        let end = span_pin(&specs, &[3, 4]).unwrap();
        assert_eq!(
            (end.side, end.inset.as_str(), end.edge),
            (PinSide::End, "0", true)
        );
        assert_eq!(span_pin(&specs, &[2]), None);
    }
}
