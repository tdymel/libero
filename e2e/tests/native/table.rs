//! `Table` as Blitz lays it out and paints it: the row line, the div caption,
//! the painted arrow. The computed arrow turn is e2e's (`table::`).

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{Table, column};

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

/// Todo 772: Blitz's collapsing model painted a 3px black grid from the first
/// cell's top border. The row line is the theme's 1px, and the cell's side
/// edge carries none.
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
    let region = page.rect("[role=region]");
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
    page.hover("[role=region]");
    page.wheel_x("[role=region]", 2000.0);
    page.wait_for(|page| gap(page).abs() <= 1.0);
    let gap = gap(&page);
    assert!(
        gap.abs() <= 1.0,
        "the last column ends {gap}px past the region"
    );
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
