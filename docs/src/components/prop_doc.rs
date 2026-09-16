use dioxus::prelude::*;
use libero::{
    components::{Code, Flex, Table, Text, Title, column},
    sx::sx,
};

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

/// One component's props: a page documents its subcomponents as further groups.
#[derive(Clone, PartialEq)]
pub struct PropGroup {
    component: String,
    props: Vec<PropDoc>,
    base: bool,
    extends: String,
}

/// Starts a Properties block for one component.
pub fn props(component: impl Into<String>, props: Vec<PropDoc>) -> PropGroup {
    PropGroup {
        component: component.into(),
        props,
        base: true,
        extends: String::new(),
    }
}

impl PropGroup {
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
///
/// Two columns, not four - the first reads as a signature (`size: Size`) with
/// the default beside it, so the table survives a phone without a horizontal
/// scrollbar and without a tall cell per row.
#[component]
pub fn PropertyTable(properties: Vec<PropGroup>) -> Element {
    // One table needs no heading - the tab already says what it lists, and the
    // component is the page's own title.
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
                        // `lg` is an h3, and the page's own title is the h1 -
                        // so the tag is pinned to h2, as `DocSection`'s xl is.
                        Title { size: "lg", component: "h2", "{group.component}" }
                    }
                    PropRows {
                        properties: group.props,
                        base: group.base,
                        extends: group.extends,
                    }
                }
            }
        }
    }
}

#[component]
fn PropRows(properties: Vec<PropDoc>, base: bool, extends: String) -> Element {
    let mut rows = properties;
    if base {
        rows.extend(base_props(&extends));
    }

    rsx! {
        Table {
            // A wrapped name cell would float a middle-aligned description
            // away from the prop it describes.
            //
            // The table lays out automatically, so it grows to the widest
            // unbreakable run in any cell - a type like
            // `Option<Callback<WindowRect>>` in the Name column, or a
            // `min_width`/`max_width`/`min_height` in a description. At 390px
            // that pushed the table past the page on 18 pages, so every cell
            // may break inside a word when it has to.
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
                    .render(|p: &PropDoc| rsx! {
                        for (code, run) in code_spans(&p.description) {
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
                    }),
            ],
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
