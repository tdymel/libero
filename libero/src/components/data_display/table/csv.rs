use super::column::Column;

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
/// Text users typed can run as a formula in a spreadsheet (CSV injection,
/// OWASP). Neutralise such cells before writing them:
///
/// ```rust
/// let mut rows = vec![vec!["=HYPERLINK(\"x\")".to_string()]];
/// for cell in rows.iter_mut().flatten() {
///     if cell.starts_with(['=', '+', '-', '@', '\t', '\r']) {
///         cell.insert(0, '\'');
///     }
/// }
/// assert_eq!(rows[0][0], "'=HYPERLINK(\"x\")");
/// ```
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
/// ```rust
/// # use libero::components::{column, table_csv};
/// # struct User { name: String, age: u32 }
/// let users = vec![User { name: "Lovelace, Ada".into(), age: 36 }];
/// let columns = vec![
///     column("Name").value(|u: &User| u.name.clone()),
///     column("Age").value(|u: &User| u.age),
/// ];
/// assert_eq!(table_csv(&columns, &users), "Name,Age\r\n\"Lovelace, Ada\",36\r\n");
/// ```
pub fn table_csv<T>(columns: &[Column<T>], data: &[T]) -> String {
    let mut csv = String::new();
    for row in table_text(columns, data) {
        for (index, field) in row.iter().enumerate() {
            if index > 0 {
                csv.push(',');
            }
            push_field(&mut csv, field);
        }
        csv.push_str("\r\n");
    }
    csv
}

fn push_field(csv: &mut String, field: &str) {
    if field.contains([',', '"', '\r', '\n']) {
        csv.push('"');
        csv.push_str(&field.replace('"', "\"\""));
        csv.push('"');
    } else {
        csv.push_str(field);
    }
}

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
    fn no_rows_leave_the_header_line() {
        assert_eq!(table_csv(&columns(), &[]), "Name,Price\r\n");
    }
}
