use crate::sx::{dynamic_value::DynamicValue, static_value::StaticValue};

pub struct Declaration<Value> {
    key: &'static str,
    value: Value,
}

pub type StaticDeclaration = Declaration<StaticValue>;
pub type DynamicDeclaration = Declaration<DynamicValue>;

impl Declaration<DynamicValue> {
    pub const fn new(key: &'static str, value: DynamicValue) -> Self {
        Self { key, value }
    }
}

impl Declaration<StaticValue> {
    pub const fn new(key: &'static str, value: StaticValue) -> Self {
        Self { key, value }
    }
}
