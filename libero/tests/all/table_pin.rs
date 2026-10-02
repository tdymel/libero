//! Pinned columns: their order, their sticky insets, and how a pin meets the
//! checkbox and detail columns and a column group.

use crate::common::{body, render, style_of, tags_with};
use crate::table_fixture::{Person, cell_of, head, people, table_rule, table_state};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{PinnedColumns, Table, column},
};

#[test]
fn pinned_columns_move_to_their_edges_and_stick_at_logical_insets() {
    #[derive(Clone, PartialEq)]
    struct Row {
        id: u32,
        name: &'static str,
        city: &'static str,
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    scroll: true,
                    default_pinned_columns: PinnedColumns::default().start(["Name", "Id"]).end(["City"]),
                    data: vec![Row { id: 1, name: "Ada", city: "London" }],
                    columns: vec![
                        column("Id").value(|row: &Row| row.id).width("4rem"),
                        column("City").value(|row: &Row| row.city.to_string()),
                        column("Name").value(|row: &Row| row.name.to_string()).width("8rem"),
                        column("Note").value(|_: &Row| "-".to_string()),
                    ],
                }
            }
        }
    }

    let html = render(app);
    let body = &body(&html);
    let at = |text: &str| body.find(text).unwrap();

    // Start ones in pinned order, the rest, then end ones.
    assert!(at(">Name<") < at(">Id<") && at(">Id<") < at(">Note<") && at(">Note<") < at(">City<"));
    assert!(at(">Ada<") < at(">1<") && at(">1<") < at(">-<") && at(">-<") < at(">London<"));
    assert!(table_state(&html).contains(&"pinned".to_string()), "{html}");
    let name = cell_of(body, "th", "Name");
    assert_eq!(name["data-pin"], "start");
    assert_eq!(style_of(&name)["inset-inline-start"], "0", "{name:?}");
    assert!(!name.contains_key("data-pin-edge"), "{name:?}");
    let id = cell_of(body, "td", "1");
    assert_eq!(style_of(&id)["inset-inline-start"], "8rem", "{id:?}");
    assert!(id.contains_key("data-pin-edge"), "{id:?}");
    assert!(!cell_of(body, "td", "-").contains_key("data-pin"));
    let city = cell_of(body, "td", "London");
    assert_eq!(city["data-pin"], "end");
    assert_eq!(style_of(&city)["inset-inline-end"], "0", "{city:?}");
    let pin = table_rule(&html, r#"[data-state~="pinned"] [data-pin]"#);
    assert_eq!(pin["position"], "sticky", "{pin:?}");
}

#[test]
fn a_start_pin_holds_the_checkbox_column_and_insets_past_it() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    scroll: true,
                    selectable: true,
                    row_key: |row: &Person| row.name.to_string(),
                    default_pinned_columns: PinnedColumns::default().start(["Name"]),
                    data: people(),
                    columns: vec![column("Name").value(|row: &Person| row.name.to_string())],
                }
            }
        }
    }

    let html = render(app);

    assert!(
        table_state(&html).contains(&"pin-select".to_string()),
        "{html}"
    );
    let name = cell_of(&body(&html), "td", "Ada");
    assert!(
        style_of(&name)["inset-inline-start"].starts_with("calc("),
        "{name:?}"
    );
}

#[test]
fn a_start_pin_holds_the_toggles_and_insets_past_them_and_the_checkboxes() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    scroll: true,
                    data: people(),
                    columns: vec![column("Name").value(|row: &Person| row.name.to_string())],
                    row_key: |row: &Person| row.name.to_string(),
                    row_detail: |row: &Person| Some(rsx! { p { "{row.age} years" } }),
                    selectable: true,
                    default_pinned_columns: PinnedColumns::default().start(["Name"]),
                }
            }
        }
    }

    let html = render(app);
    assert!(
        table_state(&html).contains(&"pin-detail".to_string()),
        "{html}"
    );
    let cell = cell_of(&body(&html), "td", "Ada");
    // The toggle column's 24px, then the checkbox column's.
    let inset = &style_of(&cell)["inset-inline-start"];
    assert!(inset.starts_with("calc(calc(24px"), "{cell:?}");
}

#[test]
fn a_pin_edge_splits_a_group_and_stops_a_span() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    default_pinned_columns: PinnedColumns::default().start(["First"]),
                    data: people(),
                    columns: vec![
                        column("First")
                            .value(|row: &Person| row.name)
                            .group("Name")
                            .width("6rem")
                            .col_span(|_| 2),
                        column("Last").value(|row: &Person| row.age).group("Name"),
                        column("Note").value(|_: &Person| "-"),
                    ],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let head = head(&body);

    // The pinned half of the group sticks with its column (todo 1359), the other scrolls.
    let groups = tags_with(head, r#"scope="colgroup""#);
    assert_eq!(groups.len(), 2, "{head}");
    assert_eq!(head.matches(">Name</th>").count(), 2, "{head}");
    let (pinned, scrolled) = (&groups[0], &groups[1]);
    assert_eq!(pinned["data-pin"], "start", "{pinned:?}");
    assert!(pinned.contains_key("data-pin-edge"), "{pinned:?}");
    assert_eq!(style_of(pinned)["inset-inline-start"], "0", "{pinned:?}");
    assert!(!scrolled.contains_key("data-pin"), "{scrolled:?}");
    assert!(!scrolled.contains_key("style"), "{scrolled:?}");
    for group in &groups {
        assert!(!group.contains_key("colspan"), "{group:?}");
    }
    assert_eq!(tags_with(head, r#"data-pin="start""#).len(), 2, "{head}");

    // The span of 2 stops at the edge: Ada's age keeps its own unpinned cell.
    let rows = &body[body.find("<tbody").unwrap()..];
    let ada = cell_of(rows, "td", "Ada");
    assert_eq!(ada["data-pin"], "start", "{ada:?}");
    assert!(ada.contains_key("data-pin-edge"), "{ada:?}");
    assert!(!ada.contains_key("colspan"), "{ada:?}");
    assert_eq!(style_of(&ada)["inset-inline-start"], "0", "{ada:?}");
    let age = cell_of(rows, "td", "36");
    assert!(
        !age.contains_key("data-pin") && !age.contains_key("style"),
        "{age:?}"
    );
}
