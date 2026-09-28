use super::{
    cell_value::{FilterKind, SortKey},
    column::Column,
};

/// One column's filter: rows stay whose cell passes `operator` with `value`.
/// Keyed by header text, like [`TableSort`](super::TableSort).
///
/// ```rust
/// # use libero::components::{ColumnFilter, FilterOperator};
/// let adults = ColumnFilter::new("Age", FilterOperator::GreaterOrEqual, "18");
/// ```
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct ColumnFilter {
    /// The column's header text.
    pub column: String,
    pub operator: FilterOperator,
    /// What the cell is compared with. Empty, the filter keeps every row
    /// unless `operator` takes no value.
    pub value: String,
}

impl ColumnFilter {
    pub fn new(
        column: impl Into<String>,
        operator: FilterOperator,
        value: impl Into<String>,
    ) -> Self {
        Self {
            column: column.into(),
            operator,
            value: value.into(),
        }
    }
}

// Defined with the labels, which name each operator: localization sits below components.
pub use crate::localization::FilterOperator;

impl FilterOperator {
    /// The operators a column of `kind` offers, its default first.
    ///
    /// ```rust
    /// # use libero::components::{FilterKind, FilterOperator};
    /// assert_eq!(FilterOperator::of(FilterKind::Number)[0], FilterOperator::Equals);
    /// ```
    pub fn of(kind: FilterKind) -> &'static [Self] {
        use FilterOperator::*;
        match kind {
            FilterKind::Text => &[
                Contains,
                DoesNotContain,
                Equals,
                NotEquals,
                StartsWith,
                EndsWith,
                IsEmpty,
                IsNotEmpty,
            ],
            FilterKind::Number => &[
                Equals,
                NotEquals,
                GreaterThan,
                GreaterOrEqual,
                LessThan,
                LessOrEqual,
                IsEmpty,
                IsNotEmpty,
            ],
            FilterKind::Boolean => &[Is],
        }
    }

    /// `false` for the operators that test emptiness alone.
    pub fn takes_value(self) -> bool {
        !matches!(self, Self::IsEmpty | Self::IsNotEmpty)
    }
}

/// A number as typed: a lone comma is the decimal point, so `1,5` is 1.5.
pub(super) fn parse_number(text: &str) -> Option<f64> {
    let text = text.trim();
    let text = match (text.matches(',').count(), text.contains('.')) {
        (1, false) => text.replace(',', "."),
        _ => text.to_string(),
    };
    text.parse::<f64>().ok().filter(|value| value.is_finite())
}

/// A filter ready to run against cells.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum CellTest {
    Text(FilterOperator, String),
    Number(FilterOperator, f64),
    Boolean(bool),
    Empty(bool),
}

impl CellTest {
    /// `None` when the filter keeps every row: no value yet, a number that
    /// does not parse, or an operator `kind` does not offer.
    pub fn new(filter: &ColumnFilter, kind: FilterKind) -> Option<Self> {
        use FilterOperator::*;
        let operator = filter.operator;
        if !FilterOperator::of(kind).contains(&operator) {
            return None;
        }
        if !operator.takes_value() {
            return Some(Self::Empty(operator == IsEmpty));
        }
        let value = filter.value.trim();
        if value.is_empty() {
            return None;
        }
        match kind {
            FilterKind::Number => parse_number(value).map(|number| Self::Number(operator, number)),
            FilterKind::Boolean => match value.to_lowercase().as_str() {
                "true" => Some(Self::Boolean(true)),
                "false" => Some(Self::Boolean(false)),
                _ => None,
            },
            _ => Some(Self::Text(operator, value.to_lowercase())),
        }
    }

    /// Whether a cell with this `text` and sort `key` passes.
    pub fn passes(&self, text: &str, key: &SortKey) -> bool {
        use FilterOperator::*;
        match self {
            Self::Empty(empty) => (key.is_empty() || text.trim().is_empty()) == *empty,
            Self::Text(operator, value) => {
                let text = text.to_lowercase();
                match operator {
                    Contains => text.contains(value.as_str()),
                    DoesNotContain => !text.contains(value.as_str()),
                    Equals => text == *value,
                    NotEquals => text != *value,
                    StartsWith => text.starts_with(value.as_str()),
                    EndsWith => text.ends_with(value.as_str()),
                    _ => true,
                }
            }
            Self::Number(operator, value) => {
                let SortKey::Num(cell) = key else {
                    return *operator == NotEquals;
                };
                match operator {
                    Equals => cell == value,
                    NotEquals => cell != value,
                    GreaterThan => cell > value,
                    GreaterOrEqual => cell >= value,
                    LessThan => cell < value,
                    LessOrEqual => cell <= value,
                    _ => true,
                }
            }
            Self::Boolean(value) => *key == SortKey::text(value.to_string()),
        }
    }
}

/// The filters that test something, by column index. Items naming no
/// filterable column are skipped.
pub(super) fn cell_tests<T>(
    columns: &[Column<T>],
    filters: &[ColumnFilter],
) -> Vec<(usize, CellTest)> {
    filters
        .iter()
        .filter_map(|filter| {
            let index = columns
                .iter()
                .position(|column| column.filterable && column.header == filter.column)?;
            CellTest::new(filter, columns[index].filter_kind).map(|test| (index, test))
        })
        .collect()
}

/// Whether `row` passes every test.
pub(super) fn passes_all<T>(row: &T, columns: &[Column<T>], tests: &[(usize, CellTest)]) -> bool {
    tests.iter().all(|(index, test)| {
        let column = &columns[*index];
        test.passes(&(column.text)(row), &(column.sort_key)(row))
    })
}

/// `column`'s filter in `filters`, if any.
pub(super) fn filter_of<'a>(filters: &'a [ColumnFilter], column: &str) -> Option<&'a ColumnFilter> {
    filters.iter().find(|filter| filter.column == column)
}

/// `filters` with `column`'s item replaced by `next`, or dropped for `None`.
pub(super) fn with_filter(
    filters: &[ColumnFilter],
    column: &str,
    next: Option<ColumnFilter>,
) -> Vec<ColumnFilter> {
    let mut out: Vec<ColumnFilter> = Vec::with_capacity(filters.len() + 1);
    let mut next = next;
    for filter in filters {
        if filter.column != column {
            out.push(filter.clone());
        } else if let Some(next) = next.take() {
            out.push(next);
        }
    }
    out.extend(next);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_display::table::{cell_value::CellValue, column::column};
    use FilterOperator::*;

    fn test(kind: FilterKind, operator: FilterOperator, value: &str) -> Option<CellTest> {
        CellTest::new(&ColumnFilter::new("A", operator, value), kind)
    }

    fn text(operator: FilterOperator, value: &str, cell: &str) -> bool {
        test(FilterKind::Text, operator, value)
            .unwrap()
            .passes(cell, &SortKey::text(cell))
    }

    fn number(operator: FilterOperator, value: &str, cell: Option<f64>) -> bool {
        let key = cell.map_or(SortKey::Empty, SortKey::num);
        let shown = cell.map(|cell| cell.to_string()).unwrap_or_default();
        test(FilterKind::Number, operator, value)
            .unwrap()
            .passes(&shown, &key)
    }

    #[test]
    fn text_operators_ignore_case() {
        assert!(text(Contains, "LOVE", "Ada Lovelace"));
        assert!(!text(DoesNotContain, "love", "Ada Lovelace"));
        assert!(text(Equals, "ada", "ADA"));
        assert!(!text(NotEquals, "ada", "Ada"));
        assert!(text(StartsWith, "ad", "Ada"));
        assert!(text(EndsWith, "DA", "Ada"));
        assert!(text(IsEmpty, "", " "));
        assert!(text(IsNotEmpty, "", "x"));
    }

    #[test]
    fn numbers_compare_by_value_and_take_a_decimal_comma() {
        assert!(number(GreaterThan, "1,5", Some(2.0)));
        assert!(!number(GreaterThan, "1.5", Some(1.5)));
        assert!(number(GreaterOrEqual, " 1.5 ", Some(1.5)));
        assert!(number(LessThan, "10", Some(9.0)));
        assert!(number(LessOrEqual, "-1", Some(-1.0)));
        assert!(number(Equals, "3", Some(3.0)));
        assert!(!number(Equals, "3", None));
        assert!(number(NotEquals, "3", None));
        assert!(number(IsEmpty, "", None));
        assert!(!number(IsNotEmpty, "", None));
    }

    #[test]
    fn an_unfinished_filter_keeps_every_row() {
        assert_eq!(test(FilterKind::Text, Contains, "  "), None);
        assert_eq!(test(FilterKind::Number, Equals, "1,2,3"), None);
        assert_eq!(test(FilterKind::Number, Equals, "-"), None);
        assert_eq!(test(FilterKind::Number, Contains, "1"), None);
        assert_eq!(test(FilterKind::Boolean, Is, "maybe"), None);
    }

    #[test]
    fn booleans_match_their_value() {
        let yes = test(FilterKind::Boolean, Is, "true").unwrap();
        // A formatted cell text does not matter: the value decides.
        assert!(yes.passes("Yes", &true.sort_key()));
        assert!(!yes.passes("No", &false.sort_key()));
        assert!(!yes.passes("", &SortKey::Empty));
    }

    #[test]
    fn a_comma_is_the_decimal_point_only_alone() {
        assert_eq!(parse_number("1,5"), Some(1.5));
        assert_eq!(parse_number("1.5"), Some(1.5));
        assert_eq!(parse_number("1,000.5"), None);
        assert_eq!(parse_number("inf"), None);
    }

    #[test]
    fn tests_skip_unknown_and_unfilterable_columns() {
        struct Row {
            name: &'static str,
            id: u32,
        }
        let columns = vec![
            column("Name").value(|r: &Row| r.name),
            column("Id").value(|r: &Row| r.id).filterable(false),
        ];
        let tests = cell_tests(
            &columns,
            &[
                ColumnFilter::new("Name", Contains, "a"),
                ColumnFilter::new("Id", Equals, "1"),
                ColumnFilter::new("Gone", Contains, "a"),
            ],
        );
        assert_eq!(tests.len(), 1);
        assert!(passes_all(&Row { name: "Ada", id: 2 }, &columns, &tests));
        assert!(!passes_all(&Row { name: "Bob", id: 1 }, &columns, &tests));
    }

    #[test]
    fn a_column_keeps_one_item_in_place() {
        let filters = vec![
            ColumnFilter::new("A", Contains, "x"),
            ColumnFilter::new("B", Equals, "1"),
        ];
        let next = with_filter(&filters, "A", Some(ColumnFilter::new("A", Equals, "y")));
        assert_eq!(next[0], ColumnFilter::new("A", Equals, "y"));
        assert_eq!(next.len(), 2);
        assert_eq!(with_filter(&filters, "A", None), [filters[1].clone()]);
        assert_eq!(with_filter(&filters, "C", None), filters);
        assert_eq!(
            with_filter(&[], "C", Some(ColumnFilter::new("C", IsEmpty, ""))).len(),
            1
        );
        assert_eq!(filter_of(&filters, "B"), Some(&filters[1]));
    }
}
