use super::{Property, SxEntry, SxModifierKey, SxPropertyKey, ThemeAwareValue};
use crate::tokens::Size;
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

    pub(super) fn with_known_property(
        self,
        property: Property,
        value: impl Into<ThemeAwareValue>,
    ) -> Self {
        self.with_declaration(SxPropertyKey::Known(property), value.into())
    }

    pub(super) fn modifier(mut self, modifier: SxModifierKey, nested: Sx) -> Self {
        self.entries.push(SxEntry::Nested {
            modifier,
            sx: nested,
        });
        self
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
        self.modifier(SxModifierKey::Condition(condition.into()), nested)
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
        self.entries.retain(|entry| {
            !matches!(
                entry,
                SxEntry::Declaration {
                    property: existing,
                    ..
                } if existing == &property
            )
        });

        self.entries.push(SxEntry::Declaration { property, value });
        self
    }

    pub fn and(mut self, other: Sx) -> Self {
        for entry in other.entries {
            match entry {
                SxEntry::Declaration { property, value } => {
                    self = self.with_declaration(property, value);
                }
                SxEntry::Nested { modifier, sx } => {
                    self.entries.push(SxEntry::Nested { modifier, sx });
                }
            }
        }

        self
    }

    pub(crate) fn entries(&self) -> &[SxEntry] {
        &self.entries
    }

    pub(crate) fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash_into(&mut hasher);
        hasher.finish()
    }

    fn hash_into(&self, hasher: &mut impl Hasher) {
        self.entries.hash(hasher);
    }

    pub(crate) fn class_name(&self) -> String {
        format!("lsx-{}", encode_base62(self.hash()))
    }
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
    fn class_name_is_shorter_than_previous_hex_encoding() {
        let name = super::sx().padding("lg").class_name();
        assert!(name.starts_with("lsx-"));
        assert!(name.len() <= "lsx-".len() + 11);
    }
}
