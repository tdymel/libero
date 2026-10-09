//! `Table` column groups and spans as Blitz lays them out (1156-2e): `colspan`
//! and `rowspan` size the header and body cells; a capped table sticks only its
//! last header row, as Blitz's sticky needs a box a `<thead>` lacks.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{Aggregate, Table, column};

#[derive(Clone, PartialEq)]
struct Sale {
    city: &'static str,
    q1: u32,
    q2: u32,
}

fn sales(rows: u32) -> Vec<Sale> {
    let mut data: Vec<Sale> = (1..=rows)
        .map(|n| Sale {
            city: "Lyon",
            q1: n,
            q2: n * 2,
        })
        .collect();
    data.push(Sale {
        city: "Total",
        q1: 0,
        q2: 0,
    });
    data
}

fn columns() -> Vec<libero::components::Column<Sale>> {
    vec![
        column("City")
            .value(|s: &Sale| s.city)
            .col_span(|s: &Sale| if s.city == "Total" { 2 } else { 1 }),
        column("Q1").value(|s: &Sale| s.q1).group("Revenue"),
        column("Q2").value(|s: &Sale| s.q2).group("Revenue"),
    ]
}

#[test]
fn group_and_span_cells_cover_their_columns() {
    fn app() -> Element {
        rsx! {
            div { width: "600px",
                Table { aria_label: "Sales", data: sales(1), columns: columns() }
            }
        }
    }
    let page = mount(app);
    let group = page.rect("th[data-group]");
    let q1 = page.rect("thead tr:nth-child(2) th:nth-child(1)");
    let q2 = page.rect("thead tr:nth-child(2) th:nth-child(2)");
    assert!(
        (group.2 - (q1.2 + q2.2)).abs() <= 1.0,
        "the group is {}px wide over {}px of columns",
        group.2,
        q1.2 + q2.2
    );
    // City spans both header rows; the total row's cell spans City and Q1.
    let city = page.rect("thead tr:nth-child(1) th:nth-child(1)");
    assert!(
        (city.3 - (group.3 + q1.3)).abs() <= 1.0,
        "City is {}px high over {}px of header rows",
        city.3,
        group.3 + q1.3
    );
    let total = page.rect("tbody tr:last-child td").2;
    assert!(
        (total - (city.2 + q1.2)).abs() <= 1.0,
        "the total cell is {total}px wide over {}px",
        city.2 + q1.2
    );
}

#[test]
fn a_footer_lines_up_under_its_columns_after_the_rows() {
    fn app() -> Element {
        rsx! {
            div { width: "600px",
                Table {
                    aria_label: "Sales",
                    selectable: true,
                    data: sales(3),
                    columns: vec![
                        column("City").value(|s: &Sale| s.city),
                        column("Q1").value(|s: &Sale| s.q1).aggregate(Aggregate::Sum),
                        column("Q2").value(|s: &Sale| s.q2),
                    ],
                }
            }
        }
    }
    let page = mount(app);
    let head = page.rect("thead th:nth-child(3)");
    let foot = page.rect("tfoot td:nth-child(3)");
    assert!(
        (foot.0 - head.0).abs() <= 1.0 && (foot.2 - head.2).abs() <= 1.0,
        "the footer cell is at {} and {}px wide under a header at {} and {}px",
        foot.0,
        foot.2,
        head.0,
        head.2
    );
    let select = page.rect("thead th:nth-child(1)").2;
    let lead = page.rect("tfoot td[data-footer-lead]").2;
    assert!(
        (lead - select).abs() <= 1.0,
        "the lead footer cell is {lead}px wide over a {select}px checkbox column"
    );
    let last = page.rect("tbody tr:last-child td");
    assert!(
        foot.1 > last.1,
        "the footer at {} is not under the last row at {}",
        foot.1,
        last.1
    );
}

#[test]
fn a_capped_grouped_table_keeps_its_last_header_row() {
    fn app() -> Element {
        rsx! {
            Table {
                aria_label: "Sales",
                max_height: "200px",
                striped: true,
                data: sales(40),
                columns: columns(),
            }
        }
    }
    let mut page = mount(app);
    let area = page.rect("[data-table-scroll]");
    let row = page.rect("tbody td").1;
    page.hover("tbody tr:nth-child(5) td");
    page.wheel("tbody tr:nth-child(5) td", 300.0);
    let held = |page: &Page| page.rect("tbody td").1 < row - 100.0;
    page.wait_for(held);
    assert!(held(&page), "the rows did not scroll: {}", page.tree());
    let (x, top, _, h) = page.rect("thead tr:nth-child(2) th:nth-child(1)");
    assert!(
        (top - area.1).abs() <= 1.0,
        "the last header row sits at {top}, the area at {}",
        area.1
    );
    let header = page.painted_pixels(&[((x + 2.0) as u32, (top + h / 2.0) as u32)])[0];
    assert_eq!(
        header,
        [255, 255, 255, 255],
        "the header is not painted over the rows"
    );
}
