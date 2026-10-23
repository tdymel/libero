use dioxus::prelude::*;

use super::Mark;
use crate::{
    components::{
        common::{HtmlTag, Input, States, base_props},
        layout::use_box,
    },
    sx::ThemeAwareValue,
    theme::Gradient,
};

/// The byte length of `term` matched case-insensitively at the start of `text`.
fn match_len(text: &str, term: &str) -> Option<usize> {
    let mut rest = text.char_indices();
    for wanted in term.chars() {
        let (_, found) = rest.next()?;
        if !found.to_lowercase().eq(wanted.to_lowercase()) {
            return None;
        }
    }
    Some(rest.next().map_or(text.len(), |(at, _)| at))
}

/// `text` cut into runs, each `true` when a term matched it; the longest term wins at a
/// position, and touching matches join into one run.
fn highlight_runs<'a>(text: &'a str, terms: &[String]) -> Vec<(&'a str, bool)> {
    let mut terms: Vec<&str> = terms
        .iter()
        .map(|term| term.trim())
        .filter(|term| !term.is_empty())
        .collect();
    terms.sort_by_key(|term| std::cmp::Reverse(term.chars().count()));

    let mut runs = Vec::new();
    let (mut start, mut at, mut marked) = (0, 0, false);
    while at < text.len() {
        let (step, hit) = match terms.iter().find_map(|term| match_len(&text[at..], term)) {
            Some(len) => (len, true),
            None => (text[at..].chars().next().map_or(1, char::len_utf8), false),
        };
        if hit != marked && at > start {
            runs.push((&text[start..at], marked));
            start = at;
        }
        marked = hit;
        at += step;
    }
    if at > start {
        runs.push((&text[start..], marked));
    }
    runs
}

base_props! {
    pub struct HighlightProps {
        /// The text searched; plain text, so its matches can be found.
        #[props(into)]
        text: String,
        /// Every case-insensitive match is marked; where terms overlap, the longest wins.
        /// Blank terms match nothing.
        #[props(default)]
        terms: Vec<String>,
        /// Each match's tint, as [`Mark`]'s `color`.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Each match's gradient, as [`Mark`]'s `gradient`.
        #[props(default, into)]
        gradient: Option<Gradient>,
    }
}

/// A `<span>` of `text` with every match of `terms` in a [`Mark`]; for hand-picked
/// highlights, use `Mark` itself.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Highlight;
/// # fn app() -> Element {
/// rsx! {
///     Highlight {
///         text: "Rust components for Dioxus, in Rust",
///         terms: vec!["rust".to_string()],
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/typography/mark>
#[component]
pub fn Highlight(props: HighlightProps) -> Element {
    let runs = highlight_runs(&props.text, &props.terms);
    let children = rsx! {
        for (index, (run, marked)) in runs.into_iter().enumerate() {
            if marked {
                Mark {
                    key: "{index}",
                    color: props.color.clone(),
                    gradient: props.gradient.clone(),
                    "{run}"
                }
            } else {
                "{run}"
            }
        }
    };

    let states: Input<States> = props.states.unwrap_or_default().into();

    use_box()
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .render(HtmlTag::Span, props.attributes, children)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terms(terms: &[&str]) -> Vec<String> {
        terms.iter().map(|term| term.to_string()).collect()
    }

    #[test]
    fn every_match_is_marked_whatever_its_case() {
        assert_eq!(
            highlight_runs("Rust for rust, RUST", &terms(&["rust"])),
            [
                ("Rust", true),
                (" for ", false),
                ("rust", true),
                (", ", false),
                ("RUST", true)
            ]
        );
    }

    #[test]
    fn the_longest_term_wins_and_touching_matches_join() {
        assert_eq!(
            highlight_runs("a dioxusapp", &terms(&["dio", "dioxus", "app"])),
            [("a ", false), ("dioxusapp", true)]
        );
    }

    #[test]
    fn blank_terms_or_none_leave_the_text_whole() {
        assert_eq!(
            highlight_runs("plain", &terms(&["", "  "])),
            [("plain", false)]
        );
        assert_eq!(highlight_runs("plain", &[]), [("plain", false)]);
        assert!(highlight_runs("", &terms(&["a"])).is_empty());
    }

    #[test]
    fn multibyte_text_cuts_on_char_boundaries() {
        assert_eq!(
            highlight_runs("Größe GRÖSSE größe", &terms(&["größe"])),
            [("Größe", true), (" GRÖSSE ", false), ("größe", true)]
        );
    }
}
