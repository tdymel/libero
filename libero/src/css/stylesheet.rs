use std::collections::hash_map::DefaultHasher;
use std::fmt::Write;
use std::hash::{Hash, Hasher};

use super::css_scope::CssScope;

/// A block of CSS text, scoped to one class when built from an [`Sx`](crate::sx::Sx).
/// Raw `&str`/`String` CSS has no class.
///
/// ```
/// # use libero::Stylesheet;
/// let sheet = Stylesheet::from(".banner{padding:1rem;}");
/// assert_eq!(sheet.as_str(), ".banner{padding:1rem;}");
/// ```
///
/// Docs: <https://libero-ui.dev/about/styling>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stylesheet {
    css: String,
    class_name: Option<String>,
    /// Of the CSS before [`with_root_class`](Self::with_root_class): the class name's source.
    hash: u64,
}

impl Stylesheet {
    /// The CSS text.
    pub fn as_str(&self) -> &str {
        &self.css
    }

    pub(crate) fn class_name(&self) -> Option<&str> {
        self.class_name.as_deref()
    }

    /// Swaps the placeholder root selector for the class name. [`hash`](Self::hash)
    /// stays pre-swap, so class name and registry key are one number.
    pub(crate) fn with_root_class(mut self, placeholder: &str, class_name: String) -> Self {
        self.css = self.css.replace(placeholder, &class_name);
        self.class_name = Some(class_name);
        self
    }

    pub(crate) fn new(scopes: Vec<CssScope>) -> Self {
        let mut css = String::new();
        for scope in scopes {
            write!(&mut css, "{scope}").expect("writing CSS scope into String cannot fail");
        }
        Self::from(css)
    }

    /// The registry's key, and an `Sx` sheet's class name. Never persist it:
    /// `DefaultHasher` isn't stable across std releases.
    pub(crate) fn hash(&self) -> u64 {
        self.hash
    }
}

impl From<String> for Stylesheet {
    fn from(value: String) -> Self {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        Self {
            css: value,
            class_name: None,
            hash: hasher.finish(),
        }
    }
}

impl From<&str> for Stylesheet {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}
