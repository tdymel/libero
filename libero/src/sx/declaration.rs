use crate::sx::{dynamic_value::DynamicValue, static_value::StaticValue};
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct Declaration<Value> {
    pub(crate) key: &'static str,
    pub(crate) value: Value,
}

pub type StaticDeclaration = Declaration<StaticValue>;
pub type DynamicDeclaration = Declaration<DynamicValue>;

impl DynamicDeclaration {
    pub const fn new(key: &'static str, value: DynamicValue) -> Self {
        Self { key, value }
    }
}

impl StaticDeclaration {
    pub const fn new(key: &'static str, value: StaticValue) -> Self {
        Self { key, value }
    }
}

impl fmt::Display for StaticDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {};", self.key, self.value)
    }
}

impl fmt::Display for DynamicDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {};", self.key, self.value)
    }
}
