use crate::sx::{declaration_methods::StaticPropertyId, static_value::StaticValue};
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct StaticDeclaration {
    pub(crate) key: StaticPropertyId,
    pub(crate) value: StaticValue,
}

impl StaticDeclaration {
    pub const fn new(key: StaticPropertyId, value: StaticValue) -> Self {
        Self { key, value }
    }
}

impl fmt::Display for StaticDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.key.as_str())?;
        self.value.fmt_for_prop(self.key.as_str(), f)?;
        write!(f, ";")
    }
}
