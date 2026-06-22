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
pub enum NestedRule {
    Selector {
        fragment: &'static str,
        sx: Box<SxDyn>,
    },
    Media {
        query: &'static str,
        sx: Box<SxDyn>,
    },
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
    pub(crate) nested_rules: Vec<NestedRule>,
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

    pub fn into_dyn(self) -> SxDyn {
        self.into()
    }

    pub fn selector<S>(self, fragment: &'static str, nested: S) -> SxDyn
    where
        S: Into<SxDyn>,
    {
        self.into_dyn().selector(fragment, nested)
    }

    pub fn hover<S>(self, nested: S) -> SxDyn
    where
        S: Into<SxDyn>,
    {
        self.selector("&:hover", nested)
    }

    pub fn focus<S>(self, nested: S) -> SxDyn
    where
        S: Into<SxDyn>,
    {
        self.selector("&:focus", nested)
    }

    pub fn active<S>(self, nested: S) -> SxDyn
    where
        S: Into<SxDyn>,
    {
        self.selector("&:active", nested)
    }

    pub fn media<S>(self, query: &'static str, nested: S) -> SxDyn
    where
        S: Into<SxDyn>,
    {
        self.into_dyn().media(query, nested)
    }

    css_declaration_methods!(width, "width");
    css_declaration_methods!(height, "height");
    css_declaration_methods!(font_size, "font-size");
    css_declaration_methods!(opacity, "opacity");
    css_declaration_methods!(color, "color");
    css_declaration_methods!(background_color, "background-color");
    css_declaration_methods!(box_shadow, "box-shadow");
    css_declaration_methods!(gap, "gap");
    css_declaration_methods!(padding, "padding");
    css_declaration_methods!(padding_top, "padding-top");
    css_declaration_methods!(padding_right, "padding-right");
    css_declaration_methods!(padding_bottom, "padding-bottom");
    css_declaration_methods!(padding_left, "padding-left");
    css_declaration_methods!(margin, "margin");
    css_declaration_methods!(margin_top, "margin-top");
    css_declaration_methods!(margin_right, "margin-right");
    css_declaration_methods!(margin_bottom, "margin-bottom");
    css_declaration_methods!(margin_left, "margin-left");
    css_declaration_methods!(border_radius, "border-radius");
    css_declaration_methods!(border_top_left_radius, "border-top-left-radius");
    css_declaration_methods!(border_top_right_radius, "border-top-right-radius");
    css_declaration_methods!(border_bottom_right_radius, "border-bottom-right-radius");
    css_declaration_methods!(border_bottom_left_radius, "border-bottom-left-radius");
    css_declaration_methods!(cursor, "cursor");
    css_declaration_methods!(user_select, "user-select");
    css_declaration_methods!(text_transform, "text-transform");
    css_declaration_methods!(text_align, "text-align");
}

impl SxDyn {
    pub fn selector<S>(mut self, fragment: &'static str, nested: S) -> Self
    where
        S: Into<SxDyn>,
    {
        self.nested_rules.push(NestedRule::Selector {
            fragment,
            sx: Box::new(nested.into()),
        });
        self
    }

    pub fn hover<S>(self, nested: S) -> Self
    where
        S: Into<SxDyn>,
    {
        self.selector("&:hover", nested)
    }

    pub fn focus<S>(self, nested: S) -> Self
    where
        S: Into<SxDyn>,
    {
        self.selector("&:focus", nested)
    }

    pub fn active<S>(self, nested: S) -> Self
    where
        S: Into<SxDyn>,
    {
        self.selector("&:active", nested)
    }

    pub fn media<S>(mut self, query: &'static str, nested: S) -> Self
    where
        S: Into<SxDyn>,
    {
        self.nested_rules.push(NestedRule::Media {
            query,
            sx: Box::new(nested.into()),
        });
        self
    }
}

impl<const N: usize> From<Sx<N>> for SxDyn {
    fn from(sx: Sx<N>) -> Self {
        Self {
            declarations: sx.declarations.into_iter().collect(),
            dynamic_declarations: sx.dynamic_declarations,
            nested_rules: Vec::new(),
        }
    }
}

impl<const N: usize> From<&Sx<N>> for SxDyn {
    fn from(sx: &Sx<N>) -> Self {
        Self {
            declarations: sx.declarations.iter().cloned().collect(),
            dynamic_declarations: sx.dynamic_declarations.clone(),
            nested_rules: Vec::new(),
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

        for nested_rule in &self.nested_rules {
            match nested_rule {
                NestedRule::Selector { fragment, sx } => {
                    write!(f, "[{} {{ {} }}] ", fragment, sx)?;
                }
                NestedRule::Media { query, sx } => {
                    write!(f, "[@media {} {{ {} }}] ", query, sx)?;
                }
            }
        }

        Ok(())
    }
}
