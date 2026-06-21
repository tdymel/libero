use crate::sx::{
    declaration::{DynamicDeclaration, StaticDeclaration},
    dynamic_value::IntoDynamicValue,
    static_value::IntoStaticValue,
};
use paste::paste;
use std::fmt;

macro_rules! css_declaration_methods {
    ($name:ident, $css_name:literal) => {
        pub const fn $name(self, value: impl const IntoStaticValue) -> Sx<{ N + 1 }> {
            self.other($css_name, value)
        }

        paste! {
            pub fn [<$name _dyn>]<V>(self, value: V) -> Self
            where
                V: IntoDynamicValue,
            {
                self.other_dyn($css_name, value)
            }
        }
    };
}

#[derive(Clone, PartialEq, Eq)]
pub struct Sx<const N: usize> {
    pub(crate) declarations: [StaticDeclaration; N],
    pub(crate) dynamic_declarations: Option<Vec<DynamicDeclaration>>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SxDyn {
    pub(crate) declarations: Vec<StaticDeclaration>,
    pub(crate) dynamic_declarations: Option<Vec<DynamicDeclaration>>,
}

pub const fn sx() -> Sx<0> {
    Sx {
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
            declarations,
            dynamic_declarations,
        }
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

    css_declaration_methods!(width, "width");
    css_declaration_methods!(height, "height");
    css_declaration_methods!(opacity, "opacity");
    css_declaration_methods!(background_color, "background-color");
}

impl<const N: usize> From<Sx<N>> for SxDyn {
    fn from(sx: Sx<N>) -> Self {
        Self {
            declarations: sx.declarations.into_iter().collect(),
            dynamic_declarations: sx.dynamic_declarations,
        }
    }
}

impl<const N: usize> From<&Sx<N>> for SxDyn {
    fn from(sx: &Sx<N>) -> Self {
        Self {
            declarations: sx.declarations.iter().cloned().collect(),
            dynamic_declarations: sx.dynamic_declarations.clone(),
        }
    }
}

impl<const N: usize> From<Sx<N>> for Option<SxDyn> {
    fn from(sx: Sx<N>) -> Self {
        Some(sx.into())
    }
}

impl fmt::Display for SxDyn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for declaration in &self.declarations {
            write!(f, "{} ", declaration)?;
        }

        if let Some(dynamic_declarations) = &self.dynamic_declarations {
            for declaration in dynamic_declarations {
                write!(f, "{} ", declaration)?;
            }
        }

        Ok(())
    }
}
