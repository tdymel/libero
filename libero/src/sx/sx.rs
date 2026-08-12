use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::declaration::{Property, SxModifierKey, SxPropertyKey};

#[derive(Debug, Clone, PartialEq, Default, Hash)]
pub struct Sx {
    entries: Vec<SxEntry>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum SxEntry {
    Declaration {
        property: SxPropertyKey,
        value: String,
    },
    Nested {
        modifier: SxModifierKey,
        sx: Sx,
    },
}

impl Sx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, property: impl Into<String>, value: impl Into<String>) -> Self {
        self.entries.push(SxEntry::Declaration {
            property: SxPropertyKey::parse(property),
            value: value.into(),
        });
        self
    }

    pub(super) fn with_known_property(
        mut self,
        property: Property,
        value: impl Into<String>,
    ) -> Self {
        self.entries.push(SxEntry::Declaration {
            property: SxPropertyKey::Known(property),
            value: value.into(),
        });
        self
    }

    pub(super) fn modifier(mut self, modifier: SxModifierKey, nested: Sx) -> Self {
        self.entries.push(SxEntry::Nested {
            modifier,
            sx: nested,
        });
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
        format!("lsx-{:016x}", self.hash())
    }
}

pub fn sx() -> Sx {
    Sx::new()
}
