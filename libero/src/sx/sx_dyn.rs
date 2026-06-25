use crate::{
    Size,
    sx::{
        declaration::StaticDeclaration,
        sx::{NestedRuleKind, NestedRuleMeta, Sx},
    },
};
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub enum NestedRule {
    Selector {
        fragment: &'static str,
        sx: Box<SxDyn>,
    },
    Media {
        query: String,
        sx: Box<SxDyn>,
    },
    MediaUp {
        size: Size,
        sx: Box<SxDyn>,
    },
    MediaDown {
        size: Size,
        sx: Box<SxDyn>,
    },
    MediaBetween {
        min: Size,
        max: Size,
        sx: Box<SxDyn>,
    },
}

#[derive(Clone, PartialEq, Eq)]
pub struct SxDyn {
    pub(crate) declarations: Vec<StaticDeclaration>,
    pub(crate) nested_rules: Vec<NestedRule>,
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

    pub fn media<S>(mut self, query: impl Into<String>, nested: S) -> Self
    where
        S: Into<SxDyn>,
    {
        self.nested_rules.push(NestedRule::Media {
            query: query.into(),
            sx: Box::new(nested.into()),
        });
        self
    }

    pub fn media_up<S>(mut self, size: Size, nested: S) -> Self
    where
        S: Into<SxDyn>,
    {
        self.nested_rules.push(NestedRule::MediaUp {
            size,
            sx: Box::new(nested.into()),
        });
        self
    }

    pub fn media_down<S>(mut self, size: Size, nested: S) -> Self
    where
        S: Into<SxDyn>,
    {
        self.nested_rules.push(NestedRule::MediaDown {
            size,
            sx: Box::new(nested.into()),
        });
        self
    }

    pub fn media_between<S>(mut self, min: Size, max: Size, nested: S) -> Self
    where
        S: Into<SxDyn>,
    {
        self.nested_rules.push(NestedRule::MediaBetween {
            min,
            max,
            sx: Box::new(nested.into()),
        });
        self
    }

    pub fn fmt_declarations_only(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for declaration in &self.declarations {
            write!(f, "{} ", declaration)?;
        }

        Ok(())
    }
}

fn build_sx_dyn(
    all_declarations: &[StaticDeclaration],
    all_nested_rules: &[NestedRuleMeta],
    decl_start: usize,
    decl_len: usize,
    rule_start: usize,
    rule_len: usize,
) -> SxDyn {
    let declarations = all_declarations[decl_start..decl_start + decl_len].to_vec();
    let nested_rules = all_nested_rules[rule_start..rule_start + rule_len]
        .iter()
        .map(|meta| build_nested_rule(all_declarations, all_nested_rules, meta))
        .collect();

    SxDyn {
        declarations,
        nested_rules,
    }
}

fn build_nested_rule(
    all_declarations: &[StaticDeclaration],
    all_nested_rules: &[NestedRuleMeta],
    meta: &NestedRuleMeta,
) -> NestedRule {
    let sx = build_sx_dyn(
        all_declarations,
        all_nested_rules,
        meta.decl_start,
        meta.decl_len,
        meta.child_rule_start,
        meta.child_rule_len,
    );

    match &meta.kind {
        NestedRuleKind::Selector { fragment } => NestedRule::Selector {
            fragment,
            sx: Box::new(sx),
        },
        NestedRuleKind::Media { query } => NestedRule::Media {
            query: (*query).to_string(),
            sx: Box::new(sx),
        },
        NestedRuleKind::MediaUp { size } => NestedRule::MediaUp {
            size: *size,
            sx: Box::new(sx),
        },
        NestedRuleKind::MediaDown { size } => NestedRule::MediaDown {
            size: *size,
            sx: Box::new(sx),
        },
        NestedRuleKind::MediaBetween { min, max } => NestedRule::MediaBetween {
            min: *min,
            max: *max,
            sx: Box::new(sx),
        },
    }
}

impl<const N: usize, const R: usize> From<Sx<N, R>> for SxDyn {
    fn from(sx: Sx<N, R>) -> Self {
        build_sx_dyn(
            &sx.declarations,
            &sx.nested_rules,
            0,
            sx.root_decl_len,
            R - sx.root_rule_len,
            sx.root_rule_len,
        )
    }
}

impl<const N: usize, const R: usize> From<&Sx<N, R>> for SxDyn {
    fn from(sx: &Sx<N, R>) -> Self {
        build_sx_dyn(
            &sx.declarations,
            &sx.nested_rules,
            0,
            sx.root_decl_len,
            R - sx.root_rule_len,
            sx.root_rule_len,
        )
    }
}

impl<const N: usize, const R: usize> From<Sx<N, R>> for Option<SxDyn> {
    fn from(sx: Sx<N, R>) -> Self {
        Some(sx.into())
    }
}

impl fmt::Display for SxDyn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_declarations_only(f)?;

        for nested_rule in &self.nested_rules {
            match nested_rule {
                NestedRule::Selector { fragment, sx } => {
                    write!(f, "[{} {{ {} }}] ", fragment, sx)?;
                }
                NestedRule::Media { query, sx } => {
                    write!(f, "[@media {} {{ {} }}] ", query, sx)?;
                }
                NestedRule::MediaUp { size, sx } => {
                    write!(f, "[@media-up {:?} {{ {} }}] ", size, sx)?;
                }
                NestedRule::MediaDown { size, sx } => {
                    write!(f, "[@media-down {:?} {{ {} }}] ", size, sx)?;
                }
                NestedRule::MediaBetween { min, max, sx } => {
                    write!(f, "[@media-between {:?}-{:?} {{ {} }}] ", min, max, sx)?;
                }
            }
        }

        Ok(())
    }
}
