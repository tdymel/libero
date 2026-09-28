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
        format!("inset-inline-{}:{};", self.side.as_str(), self.inset)
    }
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

/// The shown columns in display order: start-pinned ones in `pinned` order,
/// the rest in column order, then end-pinned ones. Sets each pinned header's
/// [`CellPin`]; `lead_width` is the toggle and checkbox columns', held with a
/// start group. Returns the order and the pinned headers whose inset is unknown, as a
/// column further out has no `width`.
pub(super) fn pin_columns(
    headers: &mut [HeaderSpec],
    pinned: &PinnedColumns,
    lead_width: Option<&str>,
) -> (Vec<usize>, Vec<String>) {
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
            match &headers[index].width {
                Some(width) => widths.push(width.clone()),
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
                align: CellAlign::Start,
                sortable: false,
                row_header: false,
                hideable: true,
                hidden: false,
                width: width.map(str::to_string),
                style: None,
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

        let (order, unknown) = pin_columns(&mut specs, &pinned, None);

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

        pin_columns(&mut specs, &pinned, Some("40px"));

        assert_eq!(specs[0].pin.as_ref().unwrap().inset, "40px");
        assert_eq!(specs[1].pin.as_ref().unwrap().inset, "calc(40px + 4rem)");

        let mut specs = headers(&[("A", Some("4rem")), ("B", Some("6rem"))]);
        pin_columns(
            &mut specs,
            &PinnedColumns::default().end(["A"]),
            Some("40px"),
        );
        assert_eq!(specs[0].pin.as_ref().unwrap().inset, "0");
    }

    #[test]
    fn a_column_past_one_without_width_is_reported() {
        let mut specs = headers(&[("A", None), ("B", Some("6rem")), ("C", None)]);
        let pinned = PinnedColumns::default().start(["A", "B"]).end(["C"]);

        let (_, unknown) = pin_columns(&mut specs, &pinned, None);

        assert_eq!(unknown, vec!["B"]);
    }

    #[test]
    fn hidden_unknown_and_twice_named_columns_are_skipped() {
        let mut specs = headers(&[("A", Some("1px")), ("B", Some("2px")), ("C", Some("3px"))]);
        specs[1].hidden = true;
        let pinned = PinnedColumns::default()
            .start(["B", "X", "C", "C"])
            .end(["C", "A"]);

        let (order, _) = pin_columns(&mut specs, &pinned, None);

        assert_eq!(order, vec![2, 0]);
        assert_eq!(specs[1].pin, None);
        assert_eq!(specs[2].pin.as_ref().unwrap().side, PinSide::Start);
        assert_eq!(specs[0].pin.as_ref().unwrap().side, PinSide::End);
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
        let (order, _) = pin_columns(&mut specs, &pinned, None);

        let runs: Vec<&[usize]> = pin_runs(&specs, &order).collect();
        assert_eq!(runs, vec![&[0, 1][..], &[2], &[3, 4]]);
        assert!(pin_edge_at(&specs, &order, 2) && !pin_edge_at(&specs, &order, 1));

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
