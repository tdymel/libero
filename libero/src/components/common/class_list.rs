use std::fmt::{self, Display};

/// A CSS class list, built up incrementally - `Display`/`to_string()` is its
/// string representation (space-joined, empty entries skipped).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ClassList(Vec<String>);

impl ClassList {
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends `class`, skipping `None`/empty ones - safe to chain directly
    /// off whatever `use_css` or a caller's own `class` prop returns.
    pub fn with(mut self, class: impl Into<Option<String>>) -> Self {
        if let Some(class) = class.into().filter(|class| !class.is_empty()) {
            self.0.push(class);
        }
        self
    }
}

impl Display for ClassList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(" "))
    }
}

impl From<&str> for ClassList {
    fn from(value: &str) -> Self {
        Self::new().with(value.to_string())
    }
}

impl From<String> for ClassList {
    fn from(value: String) -> Self {
        Self::new().with(value)
    }
}

pub fn class_list() -> ClassList {
    ClassList::new()
}
