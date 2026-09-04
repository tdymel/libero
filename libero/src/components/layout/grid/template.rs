use std::any::TypeId;
use std::fmt;
use std::sync::Arc;

/// A user-defined enum naming the zones of a [`GridTemplate`].
///
/// ```ignore
/// impl GridArea for PageArea {
///     fn name(&self) -> &'static str {
///         match self { Self::Header => "header", Self::Content => "content" }
///     }
/// }
/// ```
pub trait GridArea: Copy + PartialEq + 'static {
    /// A CSS ident: ASCII alphanumeric, `-` or `_`, not starting with a digit.
    fn name(&self) -> &'static str;
}

/// A zone name with its enum erased, so `Grid` is not generic - a component
/// monomorphized per user enum is what the `Tree<T>` refactor removed.
///
/// `source` is only used to tell "wrong enum" apart from "not in the template"
/// in the warning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AreaName {
    pub(crate) name: &'static str,
    pub(crate) source: TypeId,
}

impl AreaName {
    /// A zone with no area, i.e. one used outside a `Grid`. `Option<AreaName>`
    /// cannot be a prop: `#[props(into)]` would need
    /// `From<A> for Option<AreaName>`, which overlaps `core`'s
    /// `From<T> for Option<T>`.
    pub(crate) const NONE: Self = Self {
        name: "",
        source: TypeId::of::<()>(),
    };

    pub(crate) const fn is_set(&self) -> bool {
        !self.name.is_empty()
    }
}

impl Default for AreaName {
    fn default() -> Self {
        Self::NONE
    }
}

// Legal only while `AreaName` does not itself implement `GridArea`, which the
// orphan rule keeps downstream crates from adding - but this crate could.
impl<A: GridArea> From<A> for AreaName {
    fn from(area: A) -> Self {
        Self {
            name: area.name(),
            source: TypeId::of::<A>(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum GridTemplateError {
    Empty,
    EmptyRow {
        row: usize,
    },
    /// Rows are reconciled to the LCM of their cell counts; 12 is the cap.
    TooManyColumns {
        columns: usize,
        rows: Vec<usize>,
    },
    /// CSS requires every named area to be a solid rectangle.
    NonRectangular {
        area: &'static str,
    },
    InvalidName {
        name: &'static str,
    },
}

impl fmt::Display for GridTemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "a grid template needs at least one row"),
            Self::EmptyRow { row } => write!(f, "row {row} has no cells"),
            Self::TooManyColumns { columns, rows } => write!(
                f,
                "rows of {rows:?} cells need {columns} columns to reconcile, over the limit of 12"
            ),
            Self::NonRectangular { area } => {
                write!(f, "area `{area}` does not form a rectangle")
            }
            Self::InvalidName { name } => write!(f, "`{name}` is not a valid CSS ident"),
        }
    }
}

impl std::error::Error for GridTemplateError {}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TemplateInner {
    /// The `grid-template-areas` value, rows already quoted.
    pub areas: String,
    pub columns: usize,
    pub names: Vec<&'static str>,
    pub source: TypeId,
}

/// The shape of a [`Grid`](super::Grid): named zones laid out in a matrix.
///
/// Build it once, outside a component body - `build` returns a `Result`, which
/// `?` cannot carry through `fn() -> Element`, and a template is a constant of
/// the layout.
///
/// ```ignore
/// static PAGE: StaticGridTemplate<PageArea> = StaticGridTemplate::new(|template| {
///     template
///         .row(|row| row.cell(PageArea::Header))
///         .row(|row| row.cell(PageArea::Sidebar).cells(PageArea::Content, 3))
/// });
/// ```
#[derive(Clone, Debug)]
pub struct GridTemplate(pub(crate) Arc<TemplateInner>);

// `Props` are cloned and compared every render, so the pointer check comes
// first - a template hoisted out of the render always hits it.
impl PartialEq for GridTemplate {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0 == other.0
    }
}

impl GridTemplate {
    // `new` starts a builder, and its `build()` validates and returns the template.
    #[allow(clippy::new_ret_no_self)]
    pub fn new<A: GridArea>() -> GridTemplateBuilder<A> {
        GridTemplateBuilder { rows: Vec::new() }
    }

    pub(crate) fn contains(&self, area: &AreaName) -> bool {
        self.0.names.contains(&area.name)
    }
}

pub struct GridTemplateBuilder<A: GridArea> {
    rows: Vec<Vec<A>>,
}

/// The cells of one row. Cells share the row equally, so what matters is each
/// area's share of the total - `cell(A)` alone and `cells(A, 2)` alone are the
/// same shape.
pub struct RowBuilder<A: GridArea> {
    cells: Vec<A>,
}

impl<A: GridArea> RowBuilder<A> {
    /// One cell.
    pub fn cell(self, area: A) -> Self {
        self.cells(area, 1)
    }

    /// `count` adjacent cells of one area - its share of the row.
    pub fn cells(mut self, area: A, count: usize) -> Self {
        self.cells.extend(std::iter::repeat_n(area, count));
        self
    }
}

impl<A: GridArea> GridTemplateBuilder<A> {
    /// One row of the matrix.
    pub fn row(mut self, cells: impl FnOnce(RowBuilder<A>) -> RowBuilder<A>) -> Self {
        self.rows
            .push(cells(RowBuilder { cells: Vec::new() }).cells);
        self
    }

    pub fn build(self) -> Result<GridTemplate, GridTemplateError> {
        if self.rows.is_empty() {
            return Err(GridTemplateError::Empty);
        }
        if let Some(row) = self.rows.iter().position(Vec::is_empty) {
            return Err(GridTemplateError::EmptyRow { row });
        }

        let lengths: Vec<usize> = self.rows.iter().map(Vec::len).collect();
        let columns = lengths.iter().copied().fold(1, lcm);
        if columns > 12 {
            return Err(GridTemplateError::TooManyColumns {
                columns,
                rows: lengths,
            });
        }

        // Each row is divided equally by its own cell count, so a 2-cell row
        // above a 4-cell row makes the first row's cells two columns wide.
        let expanded: Vec<Vec<&'static str>> = self
            .rows
            .iter()
            .map(|row| {
                let repeat = columns / row.len();
                row.iter()
                    .flat_map(|area| std::iter::repeat_n(area.name(), repeat))
                    .collect()
            })
            .collect();

        let mut names: Vec<&'static str> = Vec::new();
        for name in expanded.iter().flatten() {
            if !names.contains(name) {
                names.push(name);
            }
        }

        for name in &names {
            if !is_ident(name) {
                return Err(GridTemplateError::InvalidName { name });
            }
            if !is_rectangular(&expanded, name) {
                return Err(GridTemplateError::NonRectangular { area: name });
            }
        }

        let areas = expanded
            .iter()
            .map(|row| format!("\"{}\"", row.join(" ")))
            .collect::<Vec<_>>()
            .join(" ");

        Ok(GridTemplate(Arc::new(TemplateInner {
            areas,
            columns,
            names,
            source: TypeId::of::<A>(),
        })))
    }
}

fn lcm(a: usize, b: usize) -> usize {
    a / gcd(a, b) * b
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// CSS rejects a named area whose cells are not a solid rectangle, so this
/// mirrors the platform rather than inventing a rule.
fn is_rectangular(rows: &[Vec<&'static str>], name: &str) -> bool {
    let mut bounds: Option<(usize, usize, usize, usize)> = None;
    for (y, row) in rows.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if *cell != name {
                continue;
            }
            bounds = Some(match bounds {
                None => (x, y, x, y),
                Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
            });
        }
    }

    let Some((x0, y0, x1, y1)) = bounds else {
        return true;
    };

    (y0..=y1).all(|y| (x0..=x1).all(|x| rows[y][x] == name))
        && rows.iter().enumerate().all(|(y, row)| {
            row.iter()
                .enumerate()
                .all(|(x, cell)| *cell != name || (x0..=x1).contains(&x) && (y0..=y1).contains(&y))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, PartialEq)]
    enum Area {
        Header,
        Sidebar,
        Content,
        Footer,
    }

    impl GridArea for Area {
        fn name(&self) -> &'static str {
            match self {
                Self::Header => "header",
                Self::Sidebar => "sidebar",
                Self::Content => "content",
                Self::Footer => "footer",
            }
        }
    }

    fn areas(template: &GridTemplate) -> &str {
        &template.0.areas
    }

    #[test]
    fn rows_reconcile_to_the_lcm_of_their_cell_counts() {
        let template = GridTemplate::new()
            .row(|row| row.cells(Area::Header, 2))
            .row(|row| row.cell(Area::Sidebar).cells(Area::Content, 3))
            .build()
            .unwrap();

        assert_eq!(template.0.columns, 4);
        assert_eq!(
            areas(&template),
            "\"header header header header\" \"sidebar content content content\""
        );
    }

    #[test]
    fn a_single_cell_row_spans_the_whole_width() {
        let template = GridTemplate::new()
            .row(|row| row.cell(Area::Header))
            .row(|row| row.cell(Area::Sidebar).cell(Area::Content))
            .build()
            .unwrap();

        assert_eq!(areas(&template), "\"header header\" \"sidebar content\"");
    }

    #[test]
    fn three_beside_two_needs_six_columns() {
        let template = GridTemplate::new()
            .row(|row| {
                row.cell(Area::Header)
                    .cell(Area::Sidebar)
                    .cell(Area::Content)
            })
            .row(|row| row.cells(Area::Footer, 2))
            .build()
            .unwrap();

        assert_eq!(template.0.columns, 6);
    }

    #[test]
    fn an_empty_template_is_rejected() {
        assert_eq!(
            GridTemplate::new::<Area>().build().unwrap_err(),
            GridTemplateError::Empty
        );
    }

    #[test]
    fn an_empty_row_is_rejected() {
        assert_eq!(
            GridTemplate::new()
                .row(|row| row.cell(Area::Header))
                .row(|row| row)
                .build()
                .unwrap_err(),
            GridTemplateError::EmptyRow { row: 1 }
        );
    }

    #[test]
    fn five_beside_four_would_need_twenty_columns() {
        let err = GridTemplate::new()
            .row(|row| row.cells(Area::Header, 5))
            .row(|row| row.cells(Area::Content, 4))
            .build()
            .unwrap_err();

        assert_eq!(
            err,
            GridTemplateError::TooManyColumns {
                columns: 20,
                rows: vec![5, 4],
            }
        );
    }

    #[test]
    fn an_l_shaped_area_is_rejected() {
        let err = GridTemplate::new()
            .row(|row| row.cells(Area::Header, 2))
            .row(|row| row.cell(Area::Header).cell(Area::Content))
            .build()
            .unwrap_err();

        assert_eq!(err, GridTemplateError::NonRectangular { area: "header" });
    }

    #[test]
    fn a_split_area_is_rejected() {
        let err = GridTemplate::new()
            .row(|row| {
                row.cell(Area::Header)
                    .cell(Area::Content)
                    .cell(Area::Header)
            })
            .build()
            .unwrap_err();

        assert_eq!(err, GridTemplateError::NonRectangular { area: "header" });
    }

    #[test]
    fn a_name_that_is_not_a_css_ident_is_rejected() {
        #[derive(Clone, Copy, PartialEq)]
        struct Bad;
        impl GridArea for Bad {
            fn name(&self) -> &'static str {
                "2 cols"
            }
        }

        assert_eq!(
            GridTemplate::new()
                .row(|row| row.cell(Bad))
                .build()
                .unwrap_err(),
            GridTemplateError::InvalidName { name: "2 cols" }
        );
    }

    #[test]
    fn an_area_from_another_enum_is_distinguishable() {
        #[derive(Clone, Copy, PartialEq)]
        struct Other;
        impl GridArea for Other {
            fn name(&self) -> &'static str {
                "header"
            }
        }

        let template = GridTemplate::new()
            .row(|row| row.cell(Area::Header))
            .build()
            .unwrap();

        assert!(template.contains(&AreaName::from(Other)));
        assert_ne!(template.0.source, AreaName::from(Other).source);
        assert_eq!(template.0.source, AreaName::from(Area::Header).source);
    }
}
