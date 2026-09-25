//! Inline marks. A mark may carry attributes: a link is a mark with an href.

use std::fmt;

use serde::{Deserialize, Serialize};

/// An href that passed the scheme allowlist: `http`, `https` or `mailto`.
/// Deserialising checks it too, so a stored doc cannot smuggle `javascript:` in.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Href(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsafeHref(pub String);

impl fmt::Display for UnsafeHref {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "link scheme not allowed: {:?}", self.0)
    }
}

impl std::error::Error for UnsafeHref {}

impl Href {
    pub const SCHEMES: [&str; 3] = ["http", "https", "mailto"];

    /// Strips what a browser strips (tab, newline, edge controls and spaces),
    /// then accepts only an absolute URL with an allowed scheme.
    pub fn parse(raw: &str) -> Result<Self, UnsafeHref> {
        let cleaned: String = raw
            .chars()
            .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
            .collect();
        let cleaned = cleaned.trim_matches(|c: char| c <= ' ');
        let rejected = || UnsafeHref(raw.to_string());
        let (scheme, rest) = cleaned.split_once(':').ok_or_else(rejected)?;
        let valid_scheme = scheme
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
        let scheme = scheme.to_ascii_lowercase();
        if !valid_scheme || !Self::SCHEMES.contains(&scheme.as_str()) || rest.is_empty() {
            return Err(rejected());
        }
        if cleaned.chars().any(char::is_control) {
            return Err(rejected());
        }
        Ok(Self(format!("{scheme}:{rest}")))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Href {
    type Error = UnsafeHref;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::parse(&raw)
    }
}

impl From<Href> for String {
    fn from(href: Href) -> Self {
        href.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Mark {
    Bold,
    Italic,
    Underline,
    Strike,
    Code,
    Link {
        href: Href,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
}

/// A mark's kind without its attributes: what a toggle or a toolbar asks about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MarkKind {
    Bold,
    Italic,
    Underline,
    Strike,
    Code,
    Link,
}

impl Mark {
    pub fn link(href: &str) -> Result<Self, UnsafeHref> {
        Ok(Self::Link {
            href: Href::parse(href)?,
            title: None,
        })
    }

    pub fn kind(&self) -> MarkKind {
        match self {
            Self::Bold => MarkKind::Bold,
            Self::Italic => MarkKind::Italic,
            Self::Underline => MarkKind::Underline,
            Self::Strike => MarkKind::Strike,
            Self::Code => MarkKind::Code,
            Self::Link { .. } => MarkKind::Link,
        }
    }

    /// Whether typing at the end of a marked run continues the mark.
    pub fn inclusive(&self) -> bool {
        !matches!(self, Self::Link { .. })
    }
}

/// At most one mark per kind, kept sorted, so equal sets compare equal.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(from = "Vec<Mark>", into = "Vec<Mark>")]
pub struct Marks(Vec<Mark>);

impl Marks {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Mark> {
        self.0.iter()
    }

    pub fn has(&self, kind: MarkKind) -> bool {
        self.get(kind).is_some()
    }

    pub fn get(&self, kind: MarkKind) -> Option<&Mark> {
        self.0.iter().find(|mark| mark.kind() == kind)
    }

    /// Adds `mark`, replacing one of the same kind (a second link replaces the first).
    pub fn add(&mut self, mark: Mark) {
        self.remove(mark.kind());
        let at = self.0.partition_point(|other| other < &mark);
        self.0.insert(at, mark);
    }

    pub fn remove(&mut self, kind: MarkKind) {
        self.0.retain(|mark| mark.kind() != kind);
    }

    pub fn with(mut self, mark: Mark) -> Self {
        self.add(mark);
        self
    }
}

impl From<Vec<Mark>> for Marks {
    fn from(marks: Vec<Mark>) -> Self {
        marks.into_iter().collect()
    }
}

impl From<Marks> for Vec<Mark> {
    fn from(marks: Marks) -> Self {
        marks.0
    }
}

impl FromIterator<Mark> for Marks {
    fn from_iter<I: IntoIterator<Item = Mark>>(marks: I) -> Self {
        let mut set = Self::new();
        for mark in marks {
            set.add(mark);
        }
        set
    }
}
