use std::fmt::{self, Display, Formatter};

/// A conditional group rule around a [`CssScope`](super::CssScope). A scope holds a
/// list: `@media` and `@container` nest, and only two `@media` fold into one `and`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AtRule {
    Media(String),
    Supports(String),
    Container { name: String, condition: String },
}

impl AtRule {
    /// Folds `other` into this rule if both are `@media` `and` chains: `@media a and b`.
    /// A media type, `not`, `or` or a comma list would change meaning, so those nest.
    pub(crate) fn merged(&self, other: &AtRule) -> Option<AtRule> {
        match (self, other) {
            (AtRule::Media(existing), AtRule::Media(added))
                if is_and_chain(existing) && is_and_chain(added) =>
            {
                Some(AtRule::Media(format!("{existing} and {added}")))
            }
            _ => None,
        }
    }
}

/// Whether `query` is `(…)` conditions joined only by `and` at its top level.
fn is_and_chain(query: &str) -> bool {
    let mut depth = 0u32;
    let mut word = String::new();
    for c in query.trim().chars().chain([' ']) {
        if depth == 0 && (c == '(' || c.is_whitespace()) {
            if !word.is_empty() && !word.eq_ignore_ascii_case("and") {
                return false;
            }
            word.clear();
        }
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return false,
            c if depth == 0 && !c.is_whitespace() => word.push(c),
            _ => {}
        }
    }
    query.trim_start().starts_with('(')
}

impl Display for AtRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AtRule::Media(query) => write!(f, "@media {query}"),
            AtRule::Supports(condition) => write!(f, "@supports {condition}"),
            AtRule::Container { name, condition } => write!(f, "@container {name} {condition}"),
        }
    }
}
