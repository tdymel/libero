use std::fmt::{self, Display};

/// A CSS class list. `Display` space-joins it, skipping empty entries.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ClassList(Vec<String>);

impl ClassList {
    pub fn new() -> Self {
        Self::default()
    }

    /// Skips `None`/empty, so `use_css`'s output chains directly.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_classes_in_the_order_they_were_added() {
        let classes = class_list()
            .with("lsx-a".to_string())
            .with("lsx-b".to_string());

        assert_eq!(classes.to_string(), "lsx-a lsx-b");
    }

    #[test]
    fn absent_and_empty_classes_are_skipped() {
        let classes = class_list()
            .with(None)
            .with(String::new())
            .with("lsx-a".to_string());

        assert_eq!(classes.to_string(), "lsx-a");
    }

    /// No deduping - a repeated class is harmless in a `class` attribute.
    #[test]
    fn a_repeated_class_is_kept_twice() {
        let classes = class_list()
            .with("lsx-a".to_string())
            .with("lsx-a".to_string());

        assert_eq!(classes.to_string(), "lsx-a lsx-a");
    }

    #[test]
    fn from_a_string_starts_a_list_with_it() {
        assert_eq!(ClassList::from("lsx-a").to_string(), "lsx-a");
        assert_eq!(ClassList::from(String::new()).to_string(), "");
    }
}
