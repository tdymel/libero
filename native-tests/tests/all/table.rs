//! `Table`'s sort arrow turns by a state-driven `transform` when the sort
//! flips to descending.

use dioxus::prelude::*;
use libero::components::{Table, column};
use native_tests::mount;

#[derive(Clone, PartialEq)]
struct Person {
    name: &'static str,
    age: u32,
}

const HEADER: &str = "th[data-sortable]";
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

#[test]
fn the_arrow_turns_when_the_sort_flips() {
    let mut page = mount(app);
    page.click(BUTTON);
    page.advance(1.0);
    assert_eq!(
        page.attr(HEADER, "aria-sort").as_deref(),
        Some("ascending"),
        "{}",
        page.tree()
    );
    let ascending = page.computed(ARROW, "transform");
    page.click(BUTTON);
    page.advance(1.0);
    assert_eq!(
        page.attr(HEADER, "aria-sort").as_deref(),
        Some("descending")
    );
    let descending = page.computed(ARROW, "transform");
    assert_ne!(ascending, descending, "the arrow stayed at {ascending}");
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
