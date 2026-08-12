#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AlignInput {
    Default,
    Start,
    End,
    Center,
    Stretch,
    Baseline,
    Other(String),
}

impl Default for AlignInput {
    fn default() -> Self {
        Self::Default
    }
}

impl AlignInput {
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Default => None,
            Self::Start => Some("flex-start"),
            Self::End => Some("flex-end"),
            Self::Center => Some("center"),
            Self::Stretch => Some("stretch"),
            Self::Baseline => Some("baseline"),
            Self::Other(value) => Some(value.as_str()),
        }
    }
}

impl From<&str> for AlignInput {
    fn from(value: &str) -> Self {
        match value {
            "flex-start" | "start" => Self::Start,
            "flex-end" | "end" => Self::End,
            "center" => Self::Center,
            "stretch" => Self::Stretch,
            "baseline" => Self::Baseline,
            _ => Self::Other(value.to_string()),
        }
    }
}

impl From<String> for AlignInput {
    fn from(value: String) -> Self {
        match value.as_str() {
            "flex-start" | "start" => Self::Start,
            "flex-end" | "end" => Self::End,
            "center" => Self::Center,
            "stretch" => Self::Stretch,
            "baseline" => Self::Baseline,
            _ => Self::Other(value),
        }
    }
}
