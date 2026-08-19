use super::{Property, SxEntry, SxModifierKey, SxPropertyKey, ThemeAwareValue};
use crate::css::canonical_condition;
use crate::tokens::{CssVar, Size};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Default, Hash)]
pub struct Sx {
    entries: Vec<SxEntry>,
}

impl Sx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(self, property: impl Into<String>, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_declaration(SxPropertyKey::parse(property), value.into())
    }

    /// Declares a CSS custom property inside this rule - the in-stylesheet
    /// counterpart to [`Variables`](crate::components::Variables), which
    /// sets one per instance on the `style` attribute.
    pub fn var(self, name: CssVar, value: impl Into<ThemeAwareValue>) -> Self {
        self.with(name.name().to_string(), value)
    }

    pub(super) fn with_known_property(
        self,
        property: Property,
        value: impl Into<ThemeAwareValue>,
    ) -> Self {
        self.with_declaration(SxPropertyKey::Known(property), value.into())
    }

    pub(super) fn modifier(self, modifier: SxModifierKey, nested: Sx) -> Self {
        self.with_modifier(modifier, nested)
    }

    pub fn hover(self, nested: Sx) -> Self {
        self.selector(":hover", nested)
    }

    pub fn focus(self, nested: Sx) -> Self {
        self.selector(":focus", nested)
    }

    pub fn focus_visible(self, nested: Sx) -> Self {
        self.selector(":focus-visible", nested)
    }

    pub fn when(self, condition: impl Into<String>, nested: Sx) -> Self {
        self.modifier(
            SxModifierKey::Condition(canonical_condition(&condition.into())),
            nested,
        )
    }

    /// One `when(size.state_name(), ..)` block per `Size` - the shape every
    /// size-aware `*Defaults::theme_vars()` needs.
    pub fn per_size(self, size_sx: impl Fn(Size) -> Sx) -> Self {
        Size::ALL.into_iter().fold(self, |base, size| {
            base.when(size.state_name(), size_sx(size))
        })
    }

    /// Same, keyed by `radius-{size}` so a component's radius can be set
    /// independently of its `size`.
    pub fn per_radius(self, radius_sx: impl Fn(Size) -> Sx) -> Self {
        Size::ALL.into_iter().fold(self, |base, radius| {
            base.when(radius.radius_state_name(), radius_sx(radius))
        })
    }

    pub fn selector(self, selector: impl Into<String>, nested: Sx) -> Self {
        self.modifier(SxModifierKey::Selector(selector.into()), nested)
    }

    pub fn breakpoint(self, breakpoint: Size, nested: Sx) -> Self {
        self.modifier(SxModifierKey::Breakpoint(breakpoint), nested)
    }

    pub fn apply_if<T>(self, value: Option<T>, f: impl FnOnce(Self, T) -> Self) -> Self {
        match value {
            Some(value) => f(self, value),
            None => self,
        }
    }

    fn with_declaration(mut self, property: SxPropertyKey, value: ThemeAwareValue) -> Self {
        // At most one declaration per property, so stop at the first hit.
        let existing = self.entries.iter().position(|entry| {
            matches!(entry, SxEntry::Declaration { property: existing, .. } if existing == &property)
        });

        if let Some(index) = existing {
            self.entries.remove(index);
        }

        self.entries.push(SxEntry::Declaration { property, value });
        self
    }

    /// At most one block per modifier: a second `:hover`/`when(..)`/breakpoint
    /// merges into the first (in place, so the block keeps the position it was
    /// first declared at) instead of emitting a duplicate scope.
    fn with_modifier(mut self, modifier: SxModifierKey, nested: Sx) -> Self {
        let existing = self.entries.iter().position(|entry| {
            matches!(entry, SxEntry::Nested { modifier: existing, .. } if existing == &modifier)
        });

        match existing {
            Some(index) => {
                let SxEntry::Nested { sx, .. } = &mut self.entries[index] else {
                    unreachable!("position() matched a nested entry")
                };
                *sx = std::mem::take(sx).and(nested);
            }
            None => self.entries.push(SxEntry::Nested {
                modifier,
                sx: nested,
            }),
        }

        self
    }

    pub fn and(mut self, other: Sx) -> Self {
        for entry in other.entries {
            match entry {
                SxEntry::Declaration { property, value } => {
                    self = self.with_declaration(property, value);
                }
                SxEntry::Nested { modifier, sx } => {
                    self = self.with_modifier(modifier, sx);
                }
            }
        }

        self
    }

    pub(crate) fn entries(&self) -> &[SxEntry] {
        &self.entries
    }

    /// Content hash of this `Sx`'s entries - not `Hash::hash`, which `Sx`
    /// also derives.
    pub(crate) fn content_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash_into(&mut hasher);
        hasher.finish()
    }

    fn hash_into(&self, hasher: &mut impl Hasher) {
        self.entries.hash(hasher);
    }

    /// This `Sx`'s class name - base62 of the hash of the CSS it renders to,
    /// which is also its key in the stylesheet registry. Building the CSS is
    /// the only way to get it, so at runtime it comes from `use_css`, which
    /// has the rendered sheet in hand; this spelling is for tests.
    #[cfg(test)]
    pub(crate) fn class_name(&self) -> String {
        class_name_from_hash(crate::css::Stylesheet::from(self).hash())
    }
}

/// The one place a `.lsx-*` class name is minted.
pub(crate) fn class_name_from_hash(hash: u64) -> String {
    format!("lsx-{}", encode_base62(hash))
}

const BASE62_ALPHABET: &[u8; 62] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Shorter than hex (base 16) for the same hash - up to 11 chars instead of
/// 16 for a `u64`.
fn encode_base62(mut value: u64) -> String {
    if value == 0 {
        return "0".to_string();
    }

    let mut chars = Vec::new();
    while value > 0 {
        chars.push(BASE62_ALPHABET[(value % 62) as usize]);
        value /= 62;
    }
    chars.reverse();

    String::from_utf8(chars).expect("base62 alphabet is ASCII")
}

pub fn sx() -> Sx {
    Sx::new()
}

#[cfg(test)]
mod tests {
    use super::encode_base62;

    #[test]
    fn encode_base62_roundtrips_edge_values() {
        assert_eq!(encode_base62(0), "0");
        assert_eq!(encode_base62(61), "z");
        assert_eq!(encode_base62(62), "10");
        assert_eq!(encode_base62(u64::MAX), "LygHa16AHYF");
    }

    #[test]
    fn per_size_and_per_radius_match_the_folds_they_replaced() {
        use crate::theme::Size;

        let helpers = super::sx()
            .per_size(|size| super::sx().padding(size.state_name()))
            .per_radius(|radius| super::sx().border_radius(radius.state_name()));
        let base = Size::ALL.into_iter().fold(super::sx(), |base, size| {
            base.when(size.state_name(), super::sx().padding(size.state_name()))
        });
        let folded = Size::ALL.into_iter().fold(base, |base, radius| {
            base.when(
                radius.radius_state_name(),
                super::sx().border_radius(radius.state_name()),
            )
        });

        assert_eq!(helpers.class_name(), folded.class_name());
    }

    #[test]
    fn class_name_is_shorter_than_previous_hex_encoding() {
        let name = super::sx().padding("lg").class_name();
        assert!(name.starts_with("lsx-"));
        assert!(name.len() <= "lsx-".len() + 11);
    }
}
