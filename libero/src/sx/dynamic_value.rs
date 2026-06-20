use crate::sx::static_value::{IntoStaticValue, StaticValue};
use std::{fmt, rc::Rc};

#[derive(Clone)]
pub enum DynamicValue {
    StaticValue(StaticValue),
    Closure(Rc<dyn Fn() -> StaticValue>),
}

pub trait IntoDynamicValue {
    fn into_dynamic_value(self) -> DynamicValue;
}

impl IntoDynamicValue for DynamicValue {
    fn into_dynamic_value(self) -> DynamicValue {
        self
    }
}

impl IntoDynamicValue for StaticValue {
    fn into_dynamic_value(self) -> DynamicValue {
        DynamicValue::StaticValue(self)
    }
}

impl IntoDynamicValue for i64 {
    fn into_dynamic_value(self) -> DynamicValue {
        self.into_static_value().into_dynamic_value()
    }
}

impl<F, T> IntoDynamicValue for F
where
    F: Fn() -> T + 'static,
    T: IntoStaticValue + 'static,
{
    fn into_dynamic_value(self) -> DynamicValue {
        DynamicValue::Closure(Rc::new(move || self().into_static_value()))
    }
}

impl PartialEq for DynamicValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StaticValue(left), Self::StaticValue(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for DynamicValue {}

impl fmt::Display for DynamicValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaticValue(value) => write!(f, "{}", value),
            Self::Closure(resolve) => write!(f, "{}", resolve()),
        }
    }
}
