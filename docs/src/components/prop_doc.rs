use dioxus::prelude::*;
use libero::{
    components::{Code, Flex, Part, Table, Text, Title, column},
    sx::sx,
};

use super::SectionLink;
use crate::Route;

/// One row of a docs page's Properties table.
#[derive(Clone, PartialEq)]
pub struct PropDoc {
    name: String,
    ty: String,
    default: String,
    description: String,
}

/// Starts a property row. `default` and `doc` fill the rest.
pub fn prop(name: impl Into<String>, ty: impl Into<String>) -> PropDoc {
    PropDoc {
        name: name.into(),
        ty: ty.into(),
        default: String::new(),
        description: String::new(),
    }
}

impl PropDoc {
    pub fn default(mut self, default: impl Into<String>) -> Self {
        self.default = default.into();
        self
    }

    pub fn doc(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}

/// One row of a component's Style API table: a part, its `data-slot`, what it is.
#[derive(Clone, PartialEq)]
struct PartDoc {
    name: String,
    slot: &'static str,
    description: String,
}

/// One component's props: a page documents its subcomponents as further groups.
#[derive(Clone, PartialEq)]
pub struct PropGroup {
    component: String,
    props: Vec<PropDoc>,
    base: bool,
    extends: String,
    parts: Vec<PartDoc>,
}

/// Starts a Properties block for one component.
pub fn props(component: impl Into<String>, props: Vec<PropDoc>) -> PropGroup {
    PropGroup {
        component: component.into(),
        props,
        base: true,
        extends: String::new(),
        parts: Vec::new(),
    }
}

impl PropGroup {
    #[cfg(test)]
    pub fn component(&self) -> &str {
        &self.component
    }

    #[cfg(test)]
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.props.iter().map(|prop| prop.name.as_str())
    }

    /// `(part, data-slot)` per row of the Style API table.
    #[cfg(test)]
    pub fn part_slots(&self) -> impl Iterator<Item = (&str, &str)> {
        self.parts
            .iter()
            .map(|part| (part.name.as_str(), part.slot))
    }

    pub fn has_parts(&self) -> bool {
        !self.parts.is_empty()
    }

    /// For a type that is not a `base_props!` component - a builder, say.
    pub fn without_base_props(mut self) -> Self {
        self.base = false;
        self
    }

    /// The tags this component's `base_props!` extends, as written there -
    /// `attributes` then takes their own attributes, not just the global ones.
    pub fn extends(mut self, tags: impl Into<String>) -> Self {
        self.extends = tags.into();
        self
    }

    /// Fills the Style API tab: every part of the component's part enum, with what it is.
    /// The slot comes from the enum, so the table cannot drift from the markup.
    pub fn parts<P: Part + std::fmt::Debug>(
        mut self,
        enum_name: &str,
        parts: Vec<(P, &str)>,
    ) -> Self {
        self.parts = parts
            .into_iter()
            .map(|(part, description)| PartDoc {
                name: format!("{enum_name}::{part:?}"),
                slot: part.slot(),
                description: description.to_string(),
            })
            .collect();
        self
    }
}

/// Appended to every component's own list: what `base_props!` gives all of them.
fn base_props(extends: &str) -> Vec<PropDoc> {
    let attributes = match extends.is_empty() {
        true => "Any global HTML attribute or DOM event handler, passed through to the root."
            .to_string(),
        false => format!(
            "Any global HTML attribute or DOM event handler, plus the ones specific to {extends}, passed through to the root."
        ),
    };

    vec![
        prop("class", "ClassList").doc("Extra class names on the root element."),
        prop("sx", "Sx").doc("Style overrides, applied after the theme's."),
        prop("states", "States").doc("`data-state` flags on the root, for styling and CSS hooks."),
        prop("attributes", "Vec<Attribute>").doc(attributes),
    ]
}

/// The Properties tab: one table per component, its own props then the shared ones.
/// Two columns (signature with default, description) so it fits a phone.
#[component]
pub fn PropertyTable(properties: Vec<PropGroup>) -> Element {
    let named = properties.len() > 1;

    rsx! {
        Flex {
            direction: "column",
            gap: "xl",
            for group in properties {
                Flex {
                    direction: "column",
                    gap: "sm",
                    if named {
                        // `lg` is an h3 by default; pinned to h2 under the page's h1.
                        Title { size: "lg", component: "h2", "{group.component}" }
                    }
                    PropRows {
                        name: format!("{} properties", group.component),
                        properties: group.props,
                        base: group.base,
                        extends: group.extends,
                    }
                }
            }
        }
    }
}

/// The Style API tab: one parts table per component that has parts.
#[component]
pub fn PartsPanel(properties: Vec<PropGroup>) -> Element {
    let groups: Vec<PropGroup> = properties
        .into_iter()
        .filter(|group| !group.parts.is_empty())
        .collect();
    let named = groups.len() > 1;

    rsx! {
        Flex {
            direction: "column",
            gap: "lg",
            Text {
                "Style a part with the "
                Code { source: "parts" }
                " prop, or address it as "
                Code { source: "[data-slot='…']" }
                " in your own CSS. The names are stable. "
                SectionLink { to: Route::StylingPage {}, section: "style-api", "Style API in Styling" }
                " explains how parts work."
            }
            for group in groups {
                Flex {
                    direction: "column",
                    gap: "sm",
                    if named {
                        // h2 directly under the page's h1, like the Properties tab.
                        Title { size: "lg", component: "h2", "{group.component}" }
                    }
                    PartRows { name: format!("{} parts", group.component), parts: group.parts }
                }
            }
        }
    }
}

#[component]
fn PropRows(name: String, properties: Vec<PropDoc>, base: bool, extends: String) -> Element {
    let mut rows = properties;
    if base {
        rows.extend(base_props(&extends));
    }

    rsx! {
        Table {
            aria_label: name,
            // Top-aligned so a wrapped name keeps its description beside it. Cells may break
            // inside words: long types pushed the auto-layout table past a 390px page.
            sx: sx().selector(
                "& td",
                sx().vertical_align("top").with("overflow-wrap", "anywhere"),
            ),
            data: rows,
            columns: vec![
                column("Name")
                    .value(|p: &PropDoc| p.name.clone())
                    .render(|p: &PropDoc| rsx! {
                        Flex {
                            wrap: "wrap",
                            gap: "xs",
                            align: "baseline",
                            // Breaking anywhere let a phone squeeze the column to
                            // a few characters a line; a type now breaks only past this.
                            sx: sx().min_width("10rem"),
                            Code { source: "{p.name}: {p.ty}", language: "rust" }
                            if !p.default.is_empty() {
                                Text { size: "xs", color: "muted.6", "default: {p.default}" }
                            }
                        }
                    }),
                column("Description")
                    .value(|p: &PropDoc| p.description.clone())
                    .render(|p: &PropDoc| prose(&p.description)),
            ],
        }
    }
}

#[component]
fn PartRows(name: String, parts: Vec<PartDoc>) -> Element {
    rsx! {
        Table {
            aria_label: name,
            sx: sx().selector(
                "& td",
                sx().vertical_align("top").with("overflow-wrap", "anywhere"),
            ),
            data: parts,
            columns: vec![
                column("Part")
                    .value(|p: &PartDoc| p.name.clone())
                    .render(|p: &PartDoc| rsx! { Code { source: "{p.name}", language: "rust" } }),
                column("data-slot")
                    .value(|p: &PartDoc| p.slot.to_string())
                    .render(|p: &PartDoc| rsx! { Code { source: "{p.slot}" } }),
                column("Description")
                    .value(|p: &PartDoc| p.description.clone())
                    .render(|p: &PartDoc| prose(&p.description)),
            ],
        }
    }
}

/// Inline prose with `` `code` `` and `*emphasis*` runs, as the docs write them.
pub(super) fn prose(text: &str) -> Element {
    rsx! {
        for (code, run) in code_spans(text) {
            if code {
                Code { source: run }
            } else {
                for (em, text) in emphasis_spans(&run) {
                    if em {
                        em { "{text}" }
                    } else {
                        "{text}"
                    }
                }
            }
        }
    }
}

/// Splits a description on backticks into `(is_code, text)` runs; an unmatched
/// backtick stays literal.
fn code_spans(text: &str) -> Vec<(bool, String)> {
    let parts: Vec<&str> = text.split('`').collect();
    let closed = match parts.len() % 2 {
        1 => parts.len(),
        _ => parts.len() - 1,
    };
    let mut runs: Vec<(bool, String)> = parts[..closed]
        .iter()
        .enumerate()
        .filter(|(_, part)| !part.is_empty())
        .map(|(i, part)| (i % 2 == 1, part.to_string()))
        .collect();
    if closed < parts.len() {
        let tail = format!("`{}", parts[closed]);
        match runs.last_mut() {
            Some((false, last)) => last.push_str(&tail),
            _ => runs.push((false, tail)),
        }
    }
    runs
}

/// Splits prose on `*one*` into `(is_emphasis, text)` runs. A `*` opens only
/// before a non-space and closes only after one, so `2 * 3` stays literal.
fn emphasis_spans(text: &str) -> Vec<(bool, String)> {
    let mut runs = Vec::new();
    let mut plain = String::new();
    let mut rest = text;
    while let Some(open) = rest.find('*') {
        plain.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let close = after
            .char_indices()
            .find(|&(i, c)| c == '*' && i > 0 && !after[..i].ends_with(char::is_whitespace))
            .map(|(i, _)| i)
            .filter(|_| !after.starts_with(char::is_whitespace));
        match close {
            Some(close) => {
                if !plain.is_empty() {
                    runs.push((false, std::mem::take(&mut plain)));
                }
                runs.push((true, after[..close].to_string()));
                rest = &after[close + 1..];
            }
            None => {
                plain.push('*');
                rest = after;
            }
        }
    }
    plain.push_str(rest);
    if !plain.is_empty() {
        runs.push((false, plain));
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::{code_spans, emphasis_spans};

    #[test]
    fn stars_round_a_word_become_emphasis() {
        assert_eq!(
            emphasis_spans("the ratio of *one* cell"),
            vec![
                (false, "the ratio of ".into()),
                (true, "one".into()),
                (false, " cell".into()),
            ]
        );
        assert_eq!(
            emphasis_spans("2 * 3 * 4"),
            vec![(false, "2 * 3 * 4".into())]
        );
        assert_eq!(emphasis_spans("a *b"), vec![(false, "a *b".into())]);
        assert_eq!(
            emphasis_spans("2 * 3 and *four*"),
            vec![(false, "2 * 3 and ".into()), (true, "four".into())]
        );
    }

    #[test]
    fn backticks_become_code_runs() {
        assert_eq!(
            code_spans("Set `dismiss` to `false` here."),
            vec![
                (false, "Set ".into()),
                (true, "dismiss".into()),
                (false, " to ".into()),
                (true, "false".into()),
                (false, " here.".into()),
            ]
        );
        assert_eq!(
            code_spans("`data-state` flags"),
            vec![(true, "data-state".into()), (false, " flags".into())]
        );
    }

    #[test]
    fn an_unmatched_backtick_stays_literal() {
        assert_eq!(
            code_spans("a `b` c `d"),
            vec![
                (false, "a ".into()),
                (true, "b".into()),
                (false, " c `d".into())
            ]
        );
        assert_eq!(code_spans("plain"), vec![(false, "plain".into())]);
    }
}
