//! `Table` as Blitz lays it out and paints it: the row line, the div caption,
//! the painted arrow. The computed arrow turn is e2e's (`table::`).

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{PinnedColumns, Table, column};

#[derive(Clone, PartialEq)]
struct Person {
    name: &'static str,
    age: u32,
}

const BUTTON: &str = "th[data-sortable] button";
const ARROW: &str = "th[data-sortable] svg";

fn app() -> Element {
    let data = vec![
        Person {
            name: "Ada",
            age: 36,
        },
        Person {
            name: "Grace",
            age: 85,
        },
    ];
    rsx! {
        Table {
            data,
            columns: vec![
                column("Name").value(|p: &Person| p.name.to_string()),
                column("Age").value(|p: &Person| p.age).sortable(),
            ],
        }
    }
}

/// Todo 772: Blitz's collapsing model painted a 3px black grid from the first cell's border.
/// The row line is the theme's 1px; the cell's side edge carries none.
#[test]
fn a_row_line_is_one_thin_theme_line() {
    let page = mount(app);
    let (x, y, w, h) = page.rect("tbody td");
    let line = page.computed("tbody td", "border-bottom-color");
    let column: Vec<(u32, u32)> = (0..6)
        .map(|i| ((x + w / 2.0) as u32, (y + h - 3.0) as u32 + i))
        .collect();
    let px = page.painted_pixels(&column);
    let lined = px.iter().filter(|p| **p != [255, 255, 255, 255]).count();
    assert_eq!(lined, 1, "the row line is {lined}px, {line}: {px:?}");
    assert!(
        px.iter().all(|p| *p != [0, 0, 0, 255]),
        "a black line: {px:?}"
    );
    let side = page.painted_pixels(&[(x as u32, (y + h / 2.0) as u32)])[0];
    assert_eq!(
        side,
        [255, 255, 255, 255],
        "the cell's side edge is painted"
    );
}

/// Todo 850: Blitz lays out no `<caption>`, so natively it is a div before the
/// table that names it, in the web caption's look.
#[test]
fn the_caption_shows_above_the_table_and_names_it() {
    fn app() -> Element {
        // In a flex row, as on a docs demo stage: the caption still sits above.
        rsx! {
            div { display: "flex",
                Table {
                    caption: "People",
                    data: vec![Person { name: "Ada", age: 36 }],
                    columns: vec![column("Name").value(|p: &Person| p.name.to_string())],
                }
            }
        }
    }
    let page = mount(app);
    let id = page
        .attr("table", "aria-labelledby")
        .unwrap_or_else(|| panic!("the table is unnamed: {}", page.tree()));
    let selector = format!("div#{id}");
    assert_eq!(page.text(&selector), "People");
    let (_, top, _, height) = page.rect(&selector);
    assert!(height > 0.0, "the caption has no height");
    let header = page.rect("thead th").1;
    assert!(
        top + height <= header,
        "the caption ({top} + {height}) is not above the header ({header})"
    );
    assert_eq!(page.computed(&selector, "font-weight"), "600");
    assert_eq!(page.computed(&selector, "text-align"), "start");
    assert_eq!(page.computed(&selector, "padding-top"), "10px");
    assert_eq!(page.computed(&selector, "padding-left"), "12px");
    assert_eq!(
        page.computed(&selector, "font-size"),
        page.computed("table", "font-size")
    );
    assert_eq!(
        page.computed(&selector, "color"),
        page.computed("table", "color")
    );
}

/// Todo 734: Blitz held a `width: 100%` table at its region's width, so the
/// cells past it overflowed where no scroll reached them.
#[test]
fn a_wide_scroll_table_scrolls_to_its_last_column() {
    fn app() -> Element {
        let columns = (0..6)
            .map(|i| column(format!("Column_heading_{i}")).value(|p: &Person| p.name.to_string()))
            .collect();
        rsx! {
            div { id: "frame", width: "300px",
                Table {
                    aria_label: "Wide",
                    scroll: true,
                    data: vec![Person { name: "Ada", age: 36 }],
                    columns,
                }
            }
        }
    }
    let mut page = mount(app);
    let region = page.rect("[data-table-scroll]");
    let before = page.rect("thead th:last-child");
    assert!(
        before.0 > region.0 + region.2,
        "the last column starts at {} inside the region {region:?}",
        before.0
    );
    let gap = |page: &Page| {
        let (x, _, width, _) = page.rect("thead th:last-child");
        x + width - (region.0 + region.2)
    };
    page.hover("[data-table-scroll]");
    page.wheel_x("[data-table-scroll]", 2000.0);
    page.wait_for(|page| gap(page).abs() <= 1.0);
    let gap = gap(&page);
    assert!(
        gap.abs() <= 1.0,
        "the last column ends {gap}px past the region"
    );
}

fn pinned_app(rtl: bool) -> Element {
    let columns = (0..6)
        .map(|i| {
            column(format!("C{i}"))
                .value(|p: &Person| p.name.to_string())
                .width("120px")
        })
        .collect();
    rsx! {
        div { width: "300px", dir: if rtl { "rtl" } else { "ltr" },
            Table {
                aria_label: "Wide",
                scroll: true,
                striped: true,
                default_pinned_columns: PinnedColumns::default().start(["C0"]).end(["C5"]),
                data: vec![Person { name: "Ada", age: 36 }, Person { name: "Grace", age: 85 }],
                columns,
            }
        }
    }
}

/// Wheels the wide table to its far end and checks C0 at the start edge, C5 at
/// the end and C1 scrolled away under C0.
fn pins_hold(mut page: Page, rtl: bool) {
    const AREA: &str = "[data-table-scroll]";
    let area = page.rect(AREA);
    let c1 = page.rect("thead th:nth-child(2)").0;
    page.hover(AREA);
    let moved = |page: &Page| (page.rect("thead th:nth-child(2)").0 - c1).abs() > 100.0;
    page.wheel_x(AREA, 2000.0);
    page.wait_for(moved);
    if !moved(&page) {
        page.wheel_x(AREA, -2000.0);
        page.wait_for(moved);
    }
    let edges = |page: &Page, selector: &str| {
        let (x, _, width, _) = page.rect(selector);
        match rtl {
            true => (x + width, x),
            false => (x, x + width),
        }
    };
    let (start, end) = match rtl {
        true => (area.0 + area.2, area.0),
        false => (area.0, area.0 + area.2),
    };
    for cell in [
        "thead th:first-child",
        "tbody tr:nth-child(2) td:first-child",
    ] {
        let (at, _) = edges(&page, cell);
        assert!(
            (at - start).abs() <= 1.0,
            "{cell} starts at {at}, the area at {start}: {}",
            page.tree()
        );
    }
    let (_, at) = edges(&page, "thead th:last-child");
    assert!(
        (at - end).abs() <= 1.0,
        "C5 ends at {at}, the area at {end}"
    );
    let moved = page.rect("thead th:nth-child(2)").0;
    assert!((moved - c1).abs() > 100.0, "C1 stayed at {moved}");
}

/// 1156-2d: Blitz's sticky emulation holds start- and end-pinned cells, the
/// header and the body ones, at both edges.
#[test]
fn pinned_columns_hold_at_both_edges() {
    pins_hold(mount(|| pinned_app(false)), false);
}

/// 1156-2d: the same in a right-to-left page, where the start edge is the right.
/// Blitz scrolls the whole RTL table off the area instead (C0 and C5 at -240).
#[test]
#[ignore = "todo 707: Blitz cannot scroll an RTL ScrollArea's left overflow"]
fn pinned_columns_hold_at_both_edges_right_to_left() {
    pins_hold(mount(|| pinned_app(true)), true);
}

/// 1156-2a: a column's `width` sits on its header cell (Blitz ignores `<col>`)
/// and sizes its body cells too, padding included.
#[test]
fn a_column_width_sizes_the_whole_column() {
    fn app() -> Element {
        rsx! {
            div { width: "600px",
                Table {
                    aria_label: "People",
                    data: vec![Person { name: "Ada", age: 36 }],
                    columns: vec![
                        column("Name").value(|p: &Person| p.name.to_string()).width("200px"),
                        column("Age").value(|p: &Person| p.age),
                    ],
                }
            }
        }
    }
    let page = mount(app);
    let header = page.rect("thead th").2;
    let cell = page.rect("tbody td").2;
    assert!(
        (header - 200.0).abs() <= 1.0,
        "the header is {header}px wide"
    );
    assert!((cell - 200.0).abs() <= 1.0, "the cell is {cell}px wide");
}

/// Todo 734: Blitz's UA sheet centres a button's content, so a sortable header
/// sat mid-cell while its column's cells start at the edge.
#[test]
fn a_sortable_header_starts_at_its_cells_edge() {
    fn app() -> Element {
        rsx! {
            Table {
                data: vec![Person { name: "Ada", age: 36 }],
                columns: vec![column("Name").value(|p: &Person| p.name.to_string()).sortable()],
            }
        }
    }
    let page = mount(app);
    let (x, _, width, _) = page.rect(BUTTON);
    let arrow = page.rect(ARROW).0;
    assert!(
        arrow < x + width / 4.0,
        "the header's arrow is at {arrow}, its button spans {x}..{}",
        x + width
    );
}

#[test]
fn the_painted_arrow_turns_too() {
    let mut page = mount(app);
    page.click(BUTTON);
    page.advance(1.0);
    let ascending = page.painted_transform(ARROW);
    page.click(BUTTON);
    page.advance(1.0);
    let descending = page.painted_transform(ARROW);
    assert_ne!(
        ascending, descending,
        "the painted arrow stayed at {ascending:?}"
    );
}

/// 1156-2c: under `max_height` the header cells hold at the scroll area's top
/// while the rows wheel under them, painted over the rows.
#[test]
fn a_capped_table_keeps_its_header_over_the_scrolled_rows() {
    fn app() -> Element {
        let data = (0..40)
            .map(|age| Person { name: "Ada", age })
            .collect::<Vec<_>>();
        rsx! {
            Table {
                aria_label: "People",
                max_height: "200px",
                // Grey rows: a see-through header would show them.
                striped: true,
                data,
                columns: vec![
                    column("Name").value(|p: &Person| p.name.to_string()),
                    column("Age").value(|p: &Person| p.age).sortable(),
                ],
            }
        }
    }
    let mut page = mount(app);
    let area = page.rect("[data-table-scroll]");
    assert!(
        (area.3 - 200.0).abs() <= 1.0,
        "the area is {}px high",
        area.3
    );
    let row = page.rect("tbody td").1;
    page.hover("tbody tr:nth-child(5) td");
    page.wheel("tbody tr:nth-child(5) td", 300.0);
    let held = |page: &Page| page.rect("tbody td").1 < row - 100.0;
    page.wait_for(held);
    assert!(held(&page), "the rows did not scroll: {}", page.tree());
    let (x, top, _, h) = page.rect("thead th");
    assert!(
        (top - area.1).abs() <= 1.0,
        "the header sits at {top}, the area at {}",
        area.1
    );
    // The header's surface, not a row's text, at its middle-left.
    let header = page.painted_pixels(&[((x + 2.0) as u32, (top + h / 2.0) as u32)])[0];
    assert_eq!(
        header,
        [255, 255, 255, 255],
        "the header is not painted over the rows"
    );
}

fn windowed() -> Element {
    let data: Vec<Person> = (1..=10_000)
        .map(|age| Person { name: "Ada", age })
        .collect();
    rsx! {
        Table {
            aria_label: "People",
            max_height: "300px",
            virtual_row_height: 40.0,
            data,
            columns: vec![
                column("Name").value(|p: &Person| p.name.to_string()),
                column("Age").value(|p: &Person| p.age).sortable(),
            ],
        }
    }
}

fn row_indices(page: &Page) -> Vec<usize> {
    page.query_all("tbody tr")
        .into_iter()
        .filter_map(|id| page.attr_of(id, "aria-rowindex")?.parse().ok())
        .collect()
}

/// 1156-5a: a wheel renders the rows it scrolls to, under the stuck header.
#[test]
fn a_wheel_moves_the_window() {
    let mut page = mount(windowed);
    page.wait_for(|page| {
        row_indices(page)
            .iter()
            .max()
            .is_some_and(|&last| last < 40)
    });
    page.hover("tbody td");
    page.wheel("tbody td", 20_000.0);
    page.wait_for(|page| {
        row_indices(page)
            .iter()
            .min()
            .is_some_and(|&first| first > 400)
    });
    let (_, header, _, _) = page.rect("thead th");
    let (_, area, _, _) = page.rect("[data-table-scroll]");
    assert!(
        (header - area).abs() <= 1.0,
        "header at {header}, area at {area}"
    );
}
