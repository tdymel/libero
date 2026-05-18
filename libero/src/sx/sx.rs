use crate::sx::{
    declaration::{DynamicDeclaration, StaticDeclaration},
    dynamic_value::IntoDynamicValue,
    static_value::IntoStaticValue,
};
use paste::paste;
use std::fmt;

macro_rules! css_declaration_methods {
    ($name:ident) => {
        pub const fn $name(self, value: impl const IntoStaticValue) -> Sx<{ N + 1 }> {
            self.other(stringify!($name), value)
        }

        paste! {
            pub fn [<$name _dyn>]<V>(self, value: V) -> Self
            where
                V: IntoDynamicValue,
            {
                self.other_dyn(stringify!($name), value)
            }
        }
    };
}

pub struct Sx<const N: usize> {
    name: Option<&'static str>,
    declarations: [StaticDeclaration; N],
    dynamic_declarations: Option<Vec<DynamicDeclaration>>,
}

pub const fn sx() -> Sx<0> {
    Sx {
        name: None,
        declarations: [],
        dynamic_declarations: None,
    }
}

impl<const N: usize> Sx<N> {
    pub const fn push_decl(self, value: StaticDeclaration) -> Sx<{ N + 1 }> {
        use std::{
            mem::{ManuallyDrop, MaybeUninit},
            ptr,
        };

        let this = ManuallyDrop::new(self);
        let name = unsafe { ptr::read(&this.name) };
        let declarations = unsafe { ptr::read(&this.declarations) };
        let dynamic_declarations = unsafe { ptr::read(&this.dynamic_declarations) };
        let declarations = ManuallyDrop::new(declarations);
        let value = ManuallyDrop::new(value);

        let mut result = MaybeUninit::<[StaticDeclaration; N + 1]>::uninit();
        let ptr = result.as_mut_ptr() as *mut StaticDeclaration;

        let declarations = unsafe {
            ptr::copy_nonoverlapping(declarations.as_ptr(), ptr, N);
            ptr::copy_nonoverlapping(&*value as *const StaticDeclaration, ptr.add(N), 1);
            result.assume_init()
        };

        Sx {
            name,
            declarations,
            dynamic_declarations,
        }
    }

    pub const fn name(mut self, name: &'static str) -> Self {
        self.name = Some(name);
        self
    }

    pub const fn other(
        self,
        key: &'static str,
        value: impl const IntoStaticValue,
    ) -> Sx<{ N + 1 }> {
        let value = value.into_static_value();
        self.push_decl(StaticDeclaration::new(key, value))
    }

    pub fn other_dyn<V>(mut self, key: &'static str, value: V) -> Self
    where
        V: IntoDynamicValue,
    {
        let value = value.into_dynamic_value();

        match &mut self.dynamic_declarations {
            Some(dynamic_declarations) => {
                dynamic_declarations.push(DynamicDeclaration::new(key, value));
            }
            None => {
                self.dynamic_declarations = Some(vec![DynamicDeclaration::new(key, value)]);
            }
        }

        self
    }

    css_declaration_methods!(width);
    css_declaration_methods!(height);
    css_declaration_methods!(opacity);
}

impl<const N: usize> fmt::Display for Sx<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = self.name {
            write!(f, ".{} {{ ", name)?;
        }

        for declaration in &self.declarations {
            write!(f, "{} ", declaration)?;
        }

        if let Some(dynamic_declarations) = &self.dynamic_declarations {
            for declaration in dynamic_declarations {
                write!(f, "{} ", declaration)?;
            }
        }

        if self.name.is_some() {
            write!(f, "}}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Sx, sx};

    #[test]
    fn display_formats_plain_css_declarations() {
        let sx = sx().width(10).height(20).opacity_dyn(|| 50);

        assert_eq!(sx.to_string(), "width: 10; height: 20; opacity: 50; ");
    }

    #[test]
    fn display_formats_named_css_rule() {
        const SX: Sx<2> = sx().name("card").width(10).height(20);

        assert_eq!(SX.to_string(), ".card { width: 10; height: 20; }");
    }
}
