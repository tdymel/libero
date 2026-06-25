use crate::{
    Size,
    sx::{declaration::StaticDeclaration, static_value::IntoStaticValue, sx_dyn::SxDyn},
};

macro_rules! css_declaration_methods {
    ($name:ident, $css_name:literal) => {
        pub const fn $name(self, value: impl const IntoStaticValue) -> Sx<{ N + 1 }> {
            self.other($css_name, value)
        }
    };
}

#[derive(Clone, PartialEq, Eq)]
pub struct Sx<const N: usize> {
    pub(crate) declarations: [StaticDeclaration; N],
}

pub const fn sx() -> Sx<0> {
    Sx { declarations: [] }
}

impl<const N: usize> Sx<N> {
    pub const fn push_decl(self, value: StaticDeclaration) -> Sx<{ N + 1 }> {
        use std::{
            mem::{ManuallyDrop, MaybeUninit},
            ptr,
        };

        let this = ManuallyDrop::new(self);
        let declarations = unsafe { ptr::read(&this.declarations) };
        let declarations = ManuallyDrop::new(declarations);
        let value = ManuallyDrop::new(value);

        let mut result = MaybeUninit::<[StaticDeclaration; N + 1]>::uninit();
        let ptr = result.as_mut_ptr() as *mut StaticDeclaration;

        let declarations = unsafe {
            ptr::copy_nonoverlapping(declarations.as_ptr(), ptr, N);
            ptr::copy_nonoverlapping(&*value as *const StaticDeclaration, ptr.add(N), 1);
            result.assume_init()
        };

        Sx { declarations }
    }

    pub const fn other(
        self,
        key: &'static str,
        value: impl const IntoStaticValue,
    ) -> Sx<{ N + 1 }> {
        let value = value.into_static_value();
        self.push_decl(StaticDeclaration::new(key, value))
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

    pub fn media<S>(self, query: impl Into<String>, nested: S) -> SxDyn
    where
        S: Into<SxDyn>,
    {
        self.into_dyn().media(query, nested)
    }

    pub fn media_up<S>(self, size: Size, nested: S) -> SxDyn
    where
        S: Into<SxDyn>,
    {
        self.into_dyn().media_up(size, nested)
    }

    pub fn media_down<S>(self, size: Size, nested: S) -> SxDyn
    where
        S: Into<SxDyn>,
    {
        self.into_dyn().media_down(size, nested)
    }

    pub fn media_between<S>(self, min: Size, max: Size, nested: S) -> SxDyn
    where
        S: Into<SxDyn>,
    {
        self.into_dyn().media_between(min, max, nested)
    }

    css_declaration_methods!(width, "width");
    css_declaration_methods!(height, "height");
    css_declaration_methods!(font_size, "font-size");
    css_declaration_methods!(font_weight, "font-weight");
    css_declaration_methods!(font_family, "font-family");
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
    css_declaration_methods!(position, "position");
    css_declaration_methods!(overflow, "overflow");
    css_declaration_methods!(transition, "transition");
    css_declaration_methods!(content, "content");
    css_declaration_methods!(left, "left");
    css_declaration_methods!(top, "top");
    css_declaration_methods!(pointer_events, "pointer-events");
    css_declaration_methods!(background, "background");
    css_declaration_methods!(transform, "transform");
    css_declaration_methods!(transform_origin, "transform-origin");
}
