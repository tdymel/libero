use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::LazyLock;

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

    pub fn with(self, property: impl Into<String>, value: impl Into<String>) -> Self {
        self.with_declaration(SxPropertyKey::parse(property), value.into())
    }

    pub(super) fn with_known_property(self, property: Property, value: impl Into<String>) -> Self {
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

    fn with_declaration(mut self, property: SxPropertyKey, value: String) -> Self {
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
        format!("lsx-{:016x}", self.hash())
    }
}

pub fn sx() -> Sx {
    Sx::new()
}

pub struct StaticSx(LazyLock<Sx>);

impl std::fmt::Debug for StaticSx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StaticSx").finish()
    }
}

impl PartialEq for StaticSx {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl StaticSx {
    pub const fn new(init: fn() -> Sx) -> Self {
        Self(LazyLock::new(init))
    }
}

impl Deref for StaticSx {
    type Target = Sx;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SxInput {
    #[default]
    None,
    Owned(Sx),
    Static(&'static StaticSx),
}

impl SxInput {
    pub fn as_sx(&self) -> Option<&Sx> {
        match self {
            Self::None => None,
            Self::Owned(sx) => Some(sx),
            Self::Static(sx) => Some(sx),
        }
    }
}

impl From<Sx> for SxInput {
    fn from(value: Sx) -> Self {
        Self::Owned(value)
    }
}

impl From<&'static StaticSx> for SxInput {
    fn from(value: &'static StaticSx) -> Self {
        Self::Static(value)
    }
}
