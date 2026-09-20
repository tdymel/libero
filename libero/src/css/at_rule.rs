use std::fmt::{self, Display, Formatter};

/// A conditional group rule around a [`CssScope`](super::CssScope). A scope holds a
/// list: `@media` and `@container` nest, and only two `@media` fold into one `and`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AtRule {
    Media(String),
    Container { name: String, condition: String },
}

impl AtRule {
    /// Folds `other` into this rule if both are `@media`: `@media a and b`.
    pub(crate) fn merged(&self, other: &AtRule) -> Option<AtRule> {
        match (self, other) {
            (AtRule::Media(existing), AtRule::Media(added)) => {
                Some(AtRule::Media(format!("{existing} and {added}")))
            }
            _ => None,
        }
    }
}

impl Display for AtRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AtRule::Media(query) => write!(f, "@media {query}"),
            AtRule::Container { name, condition } => write!(f, "@container {name} {condition}"),
        }
    }
}
