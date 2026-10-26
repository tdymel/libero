use super::{column::Column, pipeline::VisibleRow};

/// The header row, then one row of cell text per row of `data`: each cell as
/// its column shows it as text, `format` applied and `render` ignored.
///
/// ```rust
/// # use libero::components::{column, table_text};
/// # struct User { name: String, age: u32 }
/// let users = vec![User { name: "Ada".into(), age: 36 }];
/// let columns = vec![
///     column("Name").value(|u: &User| u.name.clone()),
///     column("Age").value(|u: &User| u.age),
/// ];
/// assert_eq!(table_text(&columns, &users), vec![vec!["Name", "Age"], vec!["Ada", "36"]]);
/// ```
///
/// The text is as the cells show it: a cell users typed can still run as a
/// formula in a spreadsheet. [`table_csv`] neutralises such cells.
pub fn table_text<T>(columns: &[Column<T>], data: &[T]) -> Vec<Vec<String>> {
    let header = columns.iter().map(|column| column.header.clone()).collect();
    std::iter::once(header)
        .chain(
            data.iter()
                .map(|row| columns.iter().map(|column| (column.text)(row)).collect()),
        )
        .collect()
}

/// [`table_text`] as CSV (RFC 4180): comma separated, CRLF line ends, a field
/// quoted when it holds a comma, quote or line break. The caller saves it.
///
/// A field starting with `=`, `+`, `-`, `@`, Tab or CR, a plain number aside, gets a
/// leading `'`: a spreadsheet reads it as text, not a formula (CSV injection, OWASP).
/// [`table_csv_raw`] keeps every field as it is.
///
/// ```rust
/// # use libero::components::{column, table_csv};
/// # struct User { name: String, age: i32 }
/// let users = vec![
///     User { name: "Lovelace, Ada".into(), age: 36 },
///     User { name: "=1+1".into(), age: -1 },
/// ];
/// let columns = vec![
///     column("Name").value(|u: &User| u.name.clone()),
///     column("Age").value(|u: &User| u.age),
/// ];
/// assert_eq!(
///     table_csv(&columns, &users),
///     "Name,Age\r\n\"Lovelace, Ada\",36\r\n'=1+1,-1\r\n"
/// );
/// ```
pub fn table_csv<T>(columns: &[Column<T>], data: &[T]) -> String {
    csv_of(table_text(columns, data), true)
}

/// [`table_csv`] with every field as it is, formula starts included: for a reader
/// that is no spreadsheet, or that neutralises them itself.
///
/// ```rust
/// # use libero::components::{column, table_csv_raw};
/// let rows = vec!["=1+1"];
/// let columns = vec![column("Sum").value(|row: &&str| row.to_string())];
/// assert_eq!(table_csv_raw(&columns, &rows), "Sum\r\n=1+1\r\n");
/// ```
pub fn table_csv_raw<T>(columns: &[Column<T>], data: &[T]) -> String {
    csv_of(table_text(columns, data), false)
}

/// [`table_csv`] of the data rows at `order` in the columns at `shown`, group rows left
/// out; [`table_csv_raw`]'s fields unless `guard`.
pub(super) fn shown_csv<T>(
    columns: &[Column<T>],
    shown: &[usize],
    data: &[T],
    order: &[VisibleRow],
    guard: bool,
) -> String {
    let header = shown.iter().map(|&at| columns[at].header.clone()).collect();
    let rows = order.iter().filter_map(|row| row.data()).map(|row| {
        shown
            .iter()
            .map(|&at| (columns[at].text)(&data[row]))
            .collect()
    });
    csv_of(std::iter::once(header).chain(rows), guard)
}

fn csv_of(rows: impl IntoIterator<Item = Vec<String>>, guard: bool) -> String {
    let mut csv = String::new();
    for row in rows {
        for (index, field) in row.iter().enumerate() {
            if index > 0 {
                csv.push(',');
            }
            push_field(&mut csv, field, guard);
        }
        csv.push_str("\r\n");
    }
    csv
}

/// With `guard`, a field a spreadsheet would run as a formula gets a leading `'`; a
/// plain number such as `-5` stays one.
fn push_field(csv: &mut String, field: &str, guard: bool) {
    let guarded;
    let number = field.parse::<f64>().is_ok_and(f64::is_finite);
    let runs = field.starts_with(FORMULA_STARTS) && !number;
    let field = match guard && runs {
        true => {
            guarded = format!("'{field}");
            guarded.as_str()
        }
        false => field,
    };
    if field.contains([',', '"', '\r', '\n']) {
        csv.push('"');
        csv.push_str(&field.replace('"', "\"\""));
        csv.push('"');
    } else {
        csv.push_str(field);
    }
}

const FORMULA_STARTS: [char; 6] = ['=', '+', '-', '@', '\t', '\r'];

#[cfg(test)]
mod tests {
    use super::{super::column::column, *};

    struct Row {
        name: &'static str,
        cents: u64,
    }

    fn columns() -> Vec<Column<Row>> {
        vec![
            column("Name").value(|row: &Row| row.name.to_string()),
            column("Price")
                .value(|row: &Row| row.cents)
                .format(|row: &Row| format!("${}.{:02}", row.cents / 100, row.cents % 100)),
        ]
    }

    #[test]
    fn cells_read_as_their_text_with_the_header_first() {
        let rows = [Row {
            name: "Tea",
            cents: 250,
        }];

        assert_eq!(
            table_text(&columns(), &rows),
            vec![vec!["Name", "Price"], vec!["Tea", "$2.50"]]
        );
    }

    #[test]
    fn fields_with_a_separator_quote_or_line_break_are_quoted() {
        let rows = [
            Row {
                name: "a,b",
                cents: 1,
            },
            Row {
                name: "say \"hi\"",
                cents: 2,
            },
            Row {
                name: "two\nlines",
                cents: 3,
            },
        ];

        assert_eq!(
            table_csv(&columns(), &rows),
            "Name,Price\r\n\"a,b\",$0.01\r\n\"say \"\"hi\"\"\",$0.02\r\n\"two\nlines\",$0.03\r\n"
        );
    }

    #[test]
    fn a_shown_csv_follows_the_order_and_the_shown_columns() {
        let rows = [
            Row {
                name: "Tea",
                cents: 250,
            },
            Row {
                name: "Jam",
                cents: 400,
            },
        ];

        assert_eq!(
            shown_csv(
                &columns(),
                &[1, 0],
                &rows,
                &[1, 0].map(VisibleRow::Data),
                true
            ),
            "Price,Name\r\n$4.00,Jam\r\n$2.50,Tea\r\n"
        );
    }

    #[test]
    fn formula_starts_get_a_quote_mark_unless_raw() {
        let rows = [
            "=SUM(A1)", "+1+1", "-2", "-A1", "@cmd", "\tx", "\rx", "=a,b", "a=b",
        ]
        .map(|name| Row { name, cents: 0 });

        assert_eq!(
            table_csv(&columns(), &rows),
            "Name,Price\r\n'=SUM(A1),$0.00\r\n'+1+1,$0.00\r\n-2,$0.00\r\n'-A1,$0.00\r\n\
             '@cmd,$0.00\r\n'\tx,$0.00\r\n\"'\rx\",$0.00\r\n\"'=a,b\",$0.00\r\na=b,$0.00\r\n"
        );
        assert!(table_csv_raw(&columns(), &rows).contains("\r\n=SUM(A1),"));
        let order = [0].map(VisibleRow::Data);
        assert_eq!(
            shown_csv(&columns(), &[0], &rows, &order, true),
            "Name\r\n'=SUM(A1)\r\n"
        );
        assert_eq!(
            shown_csv(&columns(), &[0], &rows, &order, false),
            "Name\r\n=SUM(A1)\r\n"
        );
    }

    #[test]
    fn words_rust_parses_as_numbers_are_still_guarded() {
        let rows = [
            "-Infinity",
            "+NaN",
            "-inf",
            "+infinity",
            "-nan",
            "+5",
            "-1.5e3",
        ]
        .map(|name| Row { name, cents: 0 });

        assert_eq!(
            table_csv(&columns(), &rows),
            "Name,Price\r\n'-Infinity,$0.00\r\n'+NaN,$0.00\r\n'-inf,$0.00\r\n\
             '+infinity,$0.00\r\n'-nan,$0.00\r\n+5,$0.00\r\n-1.5e3,$0.00\r\n"
        );
    }

    #[test]
    fn no_rows_leave_the_header_line() {
        assert_eq!(table_csv(&columns(), &[]), "Name,Price\r\n");
    }
}
