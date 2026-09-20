use crate::sx::ThemeAwareValue;

/// One member of an [`AvatarGroup`](super::AvatarGroup): `"Ada Lovelace".into()`,
/// or `AvatarSpec { name: .., ..Default::default() }`.
///
/// No `Element` field, so `Vec<AvatarSpec>` compares equal and the group memoizes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AvatarSpec {
    /// The accessible name, announced for this member.
    pub name: String,
    /// The picture. Falls back to `initials` once it fails to load.
    pub src: Option<String>,
    /// Drawn when there is no picture. Never derived from `name`: which
    /// graphemes abbreviate a name depends on the script.
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
