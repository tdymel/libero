#[derive(Debug, PartialEq, Eq)]
pub enum StaticValue {
    Integer(i64),
}

pub const trait IntoStaticValue {
    fn into_static_value(self) -> StaticValue;
}

impl const IntoStaticValue for StaticValue {
    fn into_static_value(self) -> StaticValue {
        self
    }
}

impl const IntoStaticValue for i64 {
    fn into_static_value(self) -> StaticValue {
        StaticValue::Integer(self)
    }
}
