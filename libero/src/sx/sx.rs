use super::{Property, SxEntry, SxModifierKey, SxPropertyKey, ThemeAwareValue};
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

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
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
        format!("lsx-{:016x}", self.hash())
    }
}

pub fn sx() -> Sx {
    Sx::new()
}
