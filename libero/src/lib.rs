// UNDER NO CIRCUMSTANCES REMOVE THESE FEATURES!
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![feature(const_convert)]

pub struct Sx<const N: usize> {
    declarations: [Declaration<StaticValue>; N],
    dynamic_declarations: Option<Vec<Declaration<DynamicValue>>>,
}

pub const fn sx() -> Sx<0> {
    Sx {
        declarations: [],
        dynamic_declarations: None,
    }
}

impl<const N: usize> Sx<N> {
    pub const fn push_decl(self, value: Declaration<StaticValue>) -> Sx<{ N + 1 }> {
        use std::{
            mem::{ManuallyDrop, MaybeUninit},
            ptr,
        };

        let this = ManuallyDrop::new(self);
        let declarations = unsafe { ptr::read(&this.declarations) };
        let dynamic_declarations = unsafe { ptr::read(&this.dynamic_declarations) };
        let declarations = ManuallyDrop::new(declarations);
        let value = ManuallyDrop::new(value);

        let mut result = MaybeUninit::<[Declaration<StaticValue>; N + 1]>::uninit();
        let ptr = result.as_mut_ptr() as *mut Declaration<StaticValue>;

        let declarations = unsafe {
            ptr::copy_nonoverlapping(declarations.as_ptr(), ptr, N);
            ptr::copy_nonoverlapping(&*value as *const Declaration<StaticValue>, ptr.add(N), 1);
            result.assume_init()
        };

        Sx {
            declarations,
            dynamic_declarations,
        }
    }

    pub const fn opacity(self, value: impl const IntoStaticValue) -> Sx<{ N + 1 }> {
        let value = value.into_static_value();
        self.push_decl(Declaration::<StaticValue>::new("opacity", value))
    }

    pub fn opacity_dyn<V>(mut self, value: V) -> Self
    where
        V: IntoDynamicValue,
    {
        let value = value.into_dynamic_value();

        match &mut self.dynamic_declarations {
            Some(dynamic_declarations) => {
                dynamic_declarations.push(Declaration::<DynamicValue>::new("opacity", value));
            }
            None => {
                self.dynamic_declarations =
                    Some(vec![Declaration::<DynamicValue>::new("opacity", value)]);
            }
        }

        self
    }
}

pub struct Declaration<Value> {
    key: &'static str,
    value: Value,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StaticValue {
    Integer(i64),
}

pub enum DynamicValue {
    StaticValue(StaticValue),
    Closure(Box<dyn Fn() -> StaticValue>),
}

pub const trait IntoStaticValue {
    fn into_static_value(self) -> StaticValue;
}

pub trait IntoDynamicValue {
    fn into_dynamic_value(self) -> DynamicValue;
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

// impl<T: IntoStaticValue> IntoDynamicValue for T {
//     fn into_dynamic_value(self) -> DynamicValue {
//         DynamicValue::StaticValue(self.into_static_value())
//     }
// }

impl<F, T> IntoDynamicValue for F
where
    F: Fn() -> T + 'static,
    T: IntoStaticValue + 'static,
{
    fn into_dynamic_value(self) -> DynamicValue {
        DynamicValue::Closure(Box::new(move || self().into_static_value()))
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

impl std::fmt::Debug for DynamicValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StaticValue(value) => f.debug_tuple("StaticValue").field(value).finish(),
            Self::Closure(_) => f.write_str("Closure(<dyn Fn() -> StaticValue>)"),
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    impl StaticValue {
        pub const fn integer(value: i64) -> Self {
            value.into_static_value()
        }
    }

    impl DynamicValue {
        pub fn integer(value: i64) -> Self {
            value.into_static_value().into_dynamic_value()
        }
    }

    #[test]
    fn push_decl_appends_in_const_context() {
        const SX: Sx<1> = sx().opacity(50);

        assert_eq!(SX.declarations.len(), 1);
        assert_eq!(SX.declarations[0].key, "opacity");
        assert_eq!(SX.declarations[0].value, StaticValue::integer(50));
        assert!(SX.dynamic_declarations.is_none());
    }

    #[test]
    fn opacity_dyn_appends_runtime_declaration() {
        let value = 73;
        const BASE_SX: Sx<1> = sx().opacity(42);
        let sx = BASE_SX.opacity_dyn(value).opacity(value);

        assert_eq!(sx.declarations.len(), 1);
        assert_eq!(sx.dynamic_declarations.as_ref().unwrap().len(), 1);
        assert_eq!(sx.dynamic_declarations.as_ref().unwrap()[0].key, "opacity");
        assert_eq!(
            sx.dynamic_declarations.as_ref().unwrap()[0].value,
            DynamicValue::integer(73)
        );
    }

    #[test]
    fn closure_can_be_added_dynamically() {
        let sx = sx().opacity_dyn(|| 99);

        assert_eq!(sx.declarations.len(), 0);
        assert_eq!(sx.dynamic_declarations.as_ref().unwrap().len(), 1);
        assert_eq!(sx.dynamic_declarations.as_ref().unwrap()[0].key, "opacity");

        match &sx.dynamic_declarations.as_ref().unwrap()[0].value {
            DynamicValue::Closure(resolve) => {
                assert_eq!(resolve(), StaticValue::Integer(99));
            }
            DynamicValue::StaticValue(_) => panic!("expected closure value"),
        }
    }
}
