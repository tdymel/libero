//! The rows, columns and pickers the `table*.rs` tests share.

use std::collections::BTreeMap;

use libero::components::{Column, column};

use crate::common::{attributes_of, css_rules_for, tag_with};

#[derive(Clone, PartialEq)]
pub struct Person {
    pub name: &'static str,
    pub age: u32,
}

/// Ada 36, Grace 45, Linus 28: in name order, not in age order.
pub fn people() -> Vec<Person> {
    vec![
        Person {
            name: "Ada",
            age: 36,
        },
        Person {
            name: "Grace",
            age: 45,
        },
        Person {
            name: "Linus",
            age: 28,
        },
    ]
}

/// Name, a sortable row header, then Age, sortable.
pub fn person_columns() -> Vec<Column<Person>> {
    vec![
        column("Name")
            .value(|row: &Person| row.name.to_string())
            .sortable()
            .row_header(),
        column("Age").value(|row: &Person| row.age).sortable(),
    ]
}

#[derive(Clone, PartialEq)]
pub struct Item {
    pub name: &'static str,
}

#[derive(Clone, PartialEq)]
pub struct Stock {
    pub id: u32,
    pub name: &'static str,
    pub cents: u32,
}

/// Those of `texts` that `body` shows as a whole element text, in document order.
pub fn order_of<'a>(body: &str, texts: &[&'a str]) -> Vec<&'a str> {
    let mut found: Vec<(usize, &str)> = texts
        .iter()
        .filter_map(|text| body.find(&format!(">{text}<")).map(|at| (at, *text)))
        .collect();
    found.sort();
    found.into_iter().map(|(_, text)| text).collect()
}

/// The `<thead>`, open tag to close tag.
pub fn head(body: &str) -> &str {
    let start = body.find("<thead").expect("a <thead>");
    &body[start..start + body[start..].find("</thead>").expect("a </thead>")]
}

/// The attributes of the cell, `tag`, whose text is `text`.
pub fn cell_of(body: &str, tag: &str, text: &str) -> BTreeMap<String, String> {
    let at = body
        .find(&format!(">{text}<"))
        .unwrap_or_else(|| panic!("no {text} in {body}"));
    let start = body[..at].rfind(&format!("<{tag}")).unwrap();
    attributes_of(&body[start..], tag)
}

/// The tokens of the `<table>`'s `data-state`.
pub fn table_state(html: &str) -> Vec<String> {
    attributes_of(html, "table")
        .get("data-state")
        .map(|state| state.split_whitespace().map(str::to_string).collect())
        .unwrap_or_default()
}

/// The declarations of the `<table>`'s rule whose selector ends with `tail`.
pub fn table_rule(html: &str, tail: &str) -> BTreeMap<String, String> {
    css_rules_for(html, &attributes_of(html, "table"))
        .into_iter()
        .find(|rule| rule.selector.ends_with(tail))
        .map(|rule| rule.declarations)
        .unwrap_or_else(|| panic!("no table rule ending `{tail}` in:\n{html}"))
}

/// The attributes of the table's `ScrollArea` root.
pub fn region(html: &str) -> BTreeMap<String, String> {
    tag_with(html, "data-table-scroll")
}
