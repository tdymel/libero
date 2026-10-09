use std::rc::Rc;

use dioxus::prelude::*;

use super::{
    cell_value::SortKey,
    column::Column,
    core::{HeaderSpec, align_attr},
    pinning::CellPin,
    pipeline::VisibleRow,
};
use crate::localization::{Aggregate, TableLabels};

/// The footer's cells by header index: an aggregate's name and its text.
pub(super) type FooterCells = Vec<Option<(&'static str, String)>>;

/// One column's keys sized up for every [`Aggregate`].
struct Tally {
    filled: usize,
    numbers: usize,
    sum: f64,
    min: f64,
    max: f64,
}

impl Tally {
    fn of(keys: impl Iterator<Item = SortKey>) -> Self {
        let mut tally = Self {
            filled: 0,
            numbers: 0,
            sum: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        };
        for key in keys {
            tally.filled += usize::from(!key.is_empty());
            if let SortKey::Num(number) = key {
                tally.numbers += 1;
                tally.sum += number;
                tally.min = tally.min.min(number);
                tally.max = tally.max.max(number);
            }
        }
        tally
    }

    /// `None` for an average, minimum or maximum without a number.
    fn value(&self, aggregate: Aggregate) -> Option<f64> {
        let any = self.numbers > 0;
        match aggregate {
            Aggregate::Sum => Some(self.sum),
            Aggregate::Avg => any.then(|| self.sum / self.numbers as f64),
            Aggregate::Min => any.then_some(self.min),
            Aggregate::Max => any.then_some(self.max),
            Aggregate::Count => Some(self.filled as f64),
        }
    }
}

type FooterInput<T> = (Rc<Vec<T>>, Vec<Column<T>>, Vec<VisibleRow>);

/// The aggregates across renders: recomputed only when the rows, the columns or
/// the rows passing the filters change, not on a page, selection or sort.
pub(super) struct FooterRows<T> {
    input: Option<FooterInput<T>>,
    values: Vec<Option<f64>>,
}

impl<T> Default for FooterRows<T> {
    fn default() -> Self {
        Self {
            input: None,
            values: Vec::new(),
        }
    }
}

impl<T: PartialEq> FooterRows<T> {
    /// The footer of `columns` over the `data` rows at `rows`, in any order; `None`
    /// when no column aggregates.
    pub fn cells(
        &mut self,
        data: &Rc<Vec<T>>,
        columns: &[Column<T>],
        rows: &[VisibleRow],
        labels: &TableLabels,
    ) -> Option<FooterCells> {
        if columns.iter().all(|column| column.aggregate.is_none()) {
            *self = Self::default();
            return None;
        }
        let fresh = self.input.as_ref().is_some_and(|(known, cols, order)| {
            order == rows
                && cols.as_slice() == columns
                && (Rc::ptr_eq(known, data) || known == data)
        });
        if !fresh {
            self.values = columns
                .iter()
                .map(|column| {
                    let aggregate = column.aggregate?;
                    let keys = rows
                        .iter()
                        .filter_map(|row| row.data())
                        .map(|index| (column.sort_key)(&data[index]));
                    Tally::of(keys).value(aggregate)
                })
                .collect();
            self.input = Some((data.clone(), columns.to_vec(), rows.to_vec()));
        }
        Some(
            columns
                .iter()
                .zip(&self.values)
                .map(|(column, value)| {
                    let aggregate = column.aggregate?;
                    let value = (*value)?;
                    let text = match &column.aggregate_format {
                        Some(format) => format(value),
                        None => value.to_string(),
                    };
                    Some(((labels.aggregate_name)(aggregate), text))
                })
                .collect(),
        )
    }
}

/// The footer: one cell under each shown column, `leads` of them spanned by one
/// cell before them.
pub(super) fn footer_row(
    headers: &[HeaderSpec],
    shown: &[usize],
    mut cells: FooterCells,
    leads: usize,
) -> Element {
    let cells: Vec<Element> = shown
        .iter()
        .map(|&index| {
            let pin = headers[index].pin.as_ref();
            let cell = cells.get_mut(index).and_then(Option::take);
            rsx! {
                td {
                    key: "{index}",
                    "data-align": align_attr(headers[index].align),
                    "data-pin": pin.map(|pin| pin.side.as_str()),
                    "data-pin-edge": pin.filter(|pin| pin.edge).map(|_| true),
                    style: pin.map(CellPin::style),
                    if let Some((name, text)) = cell {
                        span { "data-footer-name": true, "{name}" }
                        "{text}"
                    }
                }
            }
        })
        .collect();
    rsx! {
        tfoot {
            tr {
                if leads > 0 {
                    td {
                        "data-footer-lead": true,
                        colspan: (leads > 1).then(|| leads.to_string()),
                    }
                }
                {cells.into_iter()}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::column::column, *};

    fn keys(values: &[Option<f64>]) -> Vec<SortKey> {
        values
            .iter()
            .map(|value| value.map_or(SortKey::Empty, SortKey::num))
            .collect()
    }

    fn value(aggregate: Aggregate, values: &[Option<f64>]) -> Option<f64> {
        Tally::of(keys(values).into_iter()).value(aggregate)
    }

    #[test]
    fn the_aggregates_read_the_numbers() {
        let values = [Some(4.0), None, Some(2.0), Some(9.0)];

        assert_eq!(value(Aggregate::Sum, &values), Some(15.0));
        assert_eq!(value(Aggregate::Avg, &values), Some(5.0));
        assert_eq!(value(Aggregate::Min, &values), Some(2.0));
        assert_eq!(value(Aggregate::Max, &values), Some(9.0));
        assert_eq!(value(Aggregate::Count, &values), Some(3.0));
    }

    #[test]
    fn no_numbers_leave_an_average_empty_and_a_sum_at_zero() {
        assert_eq!(value(Aggregate::Sum, &[]), Some(0.0));
        assert_eq!(value(Aggregate::Count, &[None]), Some(0.0));
        for aggregate in [Aggregate::Avg, Aggregate::Min, Aggregate::Max] {
            assert_eq!(value(aggregate, &[None]), None);
        }
    }

    #[test]
    fn text_counts_but_has_no_sum() {
        let text = [SortKey::text("a"), SortKey::text("b"), SortKey::Empty];

        assert_eq!(
            Tally::of(text.iter().cloned()).value(Aggregate::Count),
            Some(2.0)
        );
        assert_eq!(Tally::of(text.iter().cloned()).value(Aggregate::Max), None);
    }

    #[test]
    fn the_footer_aggregates_the_given_rows_and_is_reused() {
        use std::cell::Cell;

        let calls = Rc::new(Cell::new(0));
        let counted = {
            let calls = calls.clone();
            vec![
                column("N")
                    .value(move |n: &u32| {
                        calls.set(calls.get() + 1);
                        *n
                    })
                    .aggregate(Aggregate::Sum)
                    .aggregate_format(|sum| format!("{sum} in all")),
            ]
        };
        let data = Rc::new(vec![1, 2, 3, 4]);
        let mut footer = FooterRows::default();
        let cells = |footer: &mut FooterRows<u32>, rows: &[usize]| {
            let rows: Vec<VisibleRow> = rows.iter().copied().map(VisibleRow::Data).collect();
            footer.cells(&data, &counted, &rows, &TableLabels::ENGLISH)
        };

        assert_eq!(
            cells(&mut footer, &[0, 2, 3]),
            Some(vec![Some(("Sum", "8 in all".to_string()))])
        );
        assert_eq!(calls.get(), 3);
        cells(&mut footer, &[0, 2, 3]);
        assert_eq!(calls.get(), 3, "the same rows reuse the sums");
        assert_eq!(
            cells(&mut footer, &[1]),
            Some(vec![Some(("Sum", "2 in all".to_string()))])
        );
        assert_eq!(calls.get(), 4);
    }

    #[test]
    fn without_an_aggregating_column_there_is_no_footer() {
        let columns = vec![column("N").value(|n: &u32| *n)];
        let data = Rc::new(vec![1]);

        let rows = [VisibleRow::Data(0)];
        let cells = FooterRows::default().cells(&data, &columns, &rows, &TableLabels::ENGLISH);
        assert_eq!(cells, None);
    }
}
