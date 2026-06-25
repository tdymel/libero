use crate::{
    Size,
    sx::{declaration::StaticDeclaration, static_value::IntoStaticValue, sx_dyn::SxDyn},
};

#[derive(Clone, PartialEq, Eq)]
pub enum NestedRuleKind {
    Selector { fragment: &'static str },
    Media { query: &'static str },
    MediaUp { size: Size },
    MediaDown { size: Size },
    MediaBetween { min: Size, max: Size },
}

#[derive(Clone, PartialEq, Eq)]
pub struct NestedRuleMeta {
    pub(crate) kind: NestedRuleKind,
    pub(crate) decl_start: usize,
    pub(crate) decl_len: usize,
    pub(crate) child_rule_start: usize,
    pub(crate) child_rule_len: usize,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Sx<const N: usize, const R: usize> {
    pub(crate) declarations: [StaticDeclaration; N],
    pub(crate) nested_rules: [NestedRuleMeta; R],
    pub(crate) root_decl_len: usize,
    pub(crate) root_rule_len: usize,
}

#[derive(Clone, Copy)]
pub struct SxRef<'a> {
    pub(crate) declarations: &'a [StaticDeclaration],
    pub(crate) nested_rules: &'a [NestedRuleMeta],
    pub(crate) root_decl_len: usize,
    pub(crate) root_rule_len: usize,
}

impl<'a, const N: usize, const R: usize> From<&'a Sx<N, R>> for SxRef<'a> {
    fn from(sx: &'a Sx<N, R>) -> Self {
        Self {
            declarations: &sx.declarations,
            nested_rules: &sx.nested_rules,
            root_decl_len: sx.root_decl_len,
            root_rule_len: sx.root_rule_len,
        }
    }
}

pub const fn sx() -> Sx<0, 0> {
    Sx {
        declarations: [],
        nested_rules: [],
        root_decl_len: 0,
        root_rule_len: 0,
    }
}

impl<const N: usize, const R: usize> Sx<N, R> {
    pub const fn push_decl(self, value: StaticDeclaration) -> Sx<{ N + 1 }, R> {
        use std::{
            mem::{ManuallyDrop, MaybeUninit},
            ptr,
        };

        let this = ManuallyDrop::new(self);
        let declarations = unsafe { ptr::read(&this.declarations) };
        let nested_rules = unsafe { ptr::read(&this.nested_rules) };
        let root_decl_len = this.root_decl_len;
        let root_rule_len = this.root_rule_len;
        let declarations = ManuallyDrop::new(declarations);
        let nested_rules = ManuallyDrop::new(nested_rules);
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
            nested_rules: ManuallyDrop::into_inner(nested_rules),
            root_decl_len: root_decl_len + 1,
            root_rule_len,
        }
    }

    pub const fn other(
        self,
        key: &'static str,
        value: impl const IntoStaticValue,
    ) -> Sx<{ N + 1 }, R> {
        let value = value.into_static_value();
        self.push_decl(StaticDeclaration::new(key, value))
    }

    pub fn into_dyn(self) -> SxDyn {
        SxDyn::from(self.as_ref())
    }

    pub fn as_ref(&self) -> SxRef<'_> {
        self.into()
    }

    pub const fn selector<const M: usize, const K: usize>(
        self,
        fragment: &'static str,
        nested: Sx<M, K>,
    ) -> Sx<{ N + M }, { R + K + 1 }> {
        self.push_nested_rule(NestedRuleKind::Selector { fragment }, nested)
    }

    pub const fn hover<const M: usize, const K: usize>(
        self,
        nested: Sx<M, K>,
    ) -> Sx<{ N + M }, { R + K + 1 }> {
        self.selector("&:hover", nested)
    }

    pub const fn focus<const M: usize, const K: usize>(
        self,
        nested: Sx<M, K>,
    ) -> Sx<{ N + M }, { R + K + 1 }> {
        self.selector("&:focus", nested)
    }

    pub const fn active<const M: usize, const K: usize>(
        self,
        nested: Sx<M, K>,
    ) -> Sx<{ N + M }, { R + K + 1 }> {
        self.selector("&:active", nested)
    }

    pub const fn media<const M: usize, const K: usize>(
        self,
        query: &'static str,
        nested: Sx<M, K>,
    ) -> Sx<{ N + M }, { R + K + 1 }> {
        self.push_nested_rule(NestedRuleKind::Media { query }, nested)
    }

    pub const fn media_up<const M: usize, const K: usize>(
        self,
        size: Size,
        nested: Sx<M, K>,
    ) -> Sx<{ N + M }, { R + K + 1 }> {
        self.push_nested_rule(NestedRuleKind::MediaUp { size }, nested)
    }

    pub const fn media_down<const M: usize, const K: usize>(
        self,
        size: Size,
        nested: Sx<M, K>,
    ) -> Sx<{ N + M }, { R + K + 1 }> {
        self.push_nested_rule(NestedRuleKind::MediaDown { size }, nested)
    }

    pub const fn media_between<const M: usize, const K: usize>(
        self,
        min: Size,
        max: Size,
        nested: Sx<M, K>,
    ) -> Sx<{ N + M }, { R + K + 1 }> {
        self.push_nested_rule(NestedRuleKind::MediaBetween { min, max }, nested)
    }

    const fn push_nested_rule<const M: usize, const K: usize>(
        self,
        kind: NestedRuleKind,
        nested: Sx<M, K>,
    ) -> Sx<{ N + M }, { R + K + 1 }> {
        use std::{
            mem::{ManuallyDrop, MaybeUninit},
            ptr,
        };

        let this = ManuallyDrop::new(self);
        let self_declarations = unsafe { ptr::read(&this.declarations) };
        let self_nested_rules = unsafe { ptr::read(&this.nested_rules) };
        let self_root_decl_len = this.root_decl_len;
        let self_root_rule_len = this.root_rule_len;

        let nested = ManuallyDrop::new(nested);
        let nested_declarations = unsafe { ptr::read(&nested.declarations) };
        let nested_rules = unsafe { ptr::read(&nested.nested_rules) };
        let nested_root_decl_len = nested.root_decl_len;
        let nested_root_rule_len = nested.root_rule_len;

        let self_declarations = ManuallyDrop::new(self_declarations);
        let self_nested_rules = ManuallyDrop::new(self_nested_rules);
        let nested_declarations = ManuallyDrop::new(nested_declarations);
        let nested_rules = ManuallyDrop::new(nested_rules);
        let meta = ManuallyDrop::new(NestedRuleMeta {
            kind,
            decl_start: N,
            decl_len: nested_root_decl_len,
            child_rule_start: R + K - nested_root_rule_len,
            child_rule_len: nested_root_rule_len,
        });

        let mut declarations_result = MaybeUninit::<[StaticDeclaration; N + M]>::uninit();
        let declarations_ptr = declarations_result.as_mut_ptr() as *mut StaticDeclaration;
        let declarations = unsafe {
            ptr::copy_nonoverlapping(self_declarations.as_ptr(), declarations_ptr, N);
            ptr::copy_nonoverlapping(nested_declarations.as_ptr(), declarations_ptr.add(N), M);
            declarations_result.assume_init()
        };

        let mut nested_rules_result = MaybeUninit::<[NestedRuleMeta; R + K + 1]>::uninit();
        let nested_rules_ptr = nested_rules_result.as_mut_ptr() as *mut NestedRuleMeta;
        let nested_rules = unsafe {
            ptr::copy_nonoverlapping(self_nested_rules.as_ptr(), nested_rules_ptr, R);
            ptr::copy_nonoverlapping(nested_rules.as_ptr(), nested_rules_ptr.add(R), K);
            ptr::copy_nonoverlapping(
                &*meta as *const NestedRuleMeta,
                nested_rules_ptr.add(R + K),
                1,
            );
            nested_rules_result.assume_init()
        };

        Sx {
            declarations,
            nested_rules,
            root_decl_len: self_root_decl_len,
            root_rule_len: self_root_rule_len + 1,
        }
    }
}
