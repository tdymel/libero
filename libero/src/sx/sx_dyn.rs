use crate::{Size, sx::declaration::StaticDeclaration};
use std::fmt;

use super::sx::Sx;

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

impl<const N: usize> From<Sx<N>> for SxDyn {
    fn from(sx: Sx<N>) -> Self {
        Self {
            declarations: sx.declarations.into_iter().collect(),
            nested_rules: Vec::new(),
        }
    }
}

impl<const N: usize> From<&Sx<N>> for SxDyn {
    fn from(sx: &Sx<N>) -> Self {
        Self {
            declarations: sx.declarations.iter().cloned().collect(),
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
