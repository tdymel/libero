use std::fmt::{self, Display, Formatter};

/// A conditional group rule wrapping a [`CssScope`](super::CssScope). Scopes
/// hold an ordered list of these because `@media` and `@container` *nest* -
/// only two `@media` can be folded into one `and`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AtRule {
    Media(String),
    Container { name: String, condition: String },
}

impl AtRule {
    /// Folds `other` into this rule if both are `@media`, since
    /// `@media a{@media b{..}}` and `@media a and b{..}` are the same rule.
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
