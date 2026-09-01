use crate::sx::ThemeAwareValue;

/// One member of an [`AvatarGroup`](super::AvatarGroup).
///
/// A plain struct with three defaulted fields rather than a builder: that is
/// what `component-api-design`'s rule says a fixed, small set of optional
/// parts is. `Default` makes `AvatarSpec { name: .., ..Default::default() }`
/// the long form, and `"Ada Lovelace".into()` the short one.
///
/// **No `Element` field, so `Vec<AvatarSpec>` compares equal** and the group's
/// children memoize - which is also why `initials` is a `String` the caller
/// supplies rather than anything derived from `name`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AvatarSpec {
    /// The accessible name, announced for this member.
    pub name: String,
    /// The picture. Falls back to `initials` once it fails to load.
    pub src: Option<String>,
    /// What is drawn when there is no picture. Nothing is derived from
    /// `name`: an initial is a first *grapheme cluster*, not a first `char`,
    /// and which one abbreviates a name is a property of the script, not of
    /// the string.
    pub initials: Option<String>,
    /// This member's own tint, overriding the group's.
    pub color: Option<ThemeAwareValue>,
}

impl From<&str> for AvatarSpec {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Self::default()
        }
    }
}

impl From<String> for AvatarSpec {
    fn from(name: String) -> Self {
        Self {
            name,
            ..Self::default()
        }
    }
}
