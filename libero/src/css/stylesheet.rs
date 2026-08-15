use std::collections::hash_map::DefaultHasher;
use std::fmt::Write;
use std::hash::{Hash, Hasher};

use super::css_scope::CssScope;

/// A block of CSS text, optionally scoped to a single class its declarations
/// were rendered against (e.g. an [`Sx`](crate::sx::Sx) conversion). Raw text
/// built via `From<&str>`/`From<String>` is an escape hatch for CSS that
/// isn't produced by a scoped conversion, so it carries no class name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stylesheet {
    css: String,
    class_name: Option<String>,
}

impl Stylesheet {
    pub fn as_str(&self) -> &str {
        &self.css
    }

    pub(crate) fn class_name(&self) -> Option<&str> {
        self.class_name.as_deref()
    }

    pub(crate) fn with_class_name(mut self, class_name: impl Into<String>) -> Self {
        self.class_name = Some(class_name.into());
        self
    }

    pub(crate) fn new(scopes: Vec<CssScope>) -> Self {
        let mut css = String::new();
        for scope in scopes {
            write!(&mut css, "{scope}").expect("writing CSS scope into String cannot fail");
        }
        Self {
            css,
            class_name: None,
        }
    }

    pub(crate) fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.css.hash(&mut hasher);
        hasher.finish()
    }
}

impl From<String> for Stylesheet {
    fn from(value: String) -> Self {
        Self {
            css: value,
            class_name: None,
        }
    }
}

impl From<&str> for Stylesheet {
    fn from(value: &str) -> Self {
        Self {
            css: value.to_string(),
            class_name: None,
        }
    }
}
